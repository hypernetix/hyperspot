use modkit_security::AccessScope;

use super::{
    paginate_odata, Column, DomainError, Expr, LimitCfg, NewUser, ODataQuery, OffsetDateTime, Page,
    SecurityContext, Service, Set, SortDir, User, UserAM, UserDomainEvent, UserEntity,
    UserFilterField, UserODataMapper, UserPatch, Uuid,
};

async fn audit_get_user_access_best_effort(svc: &Service, id: Uuid) {
    let audit_result = svc.audit.get_user_access(id).await;
    if let Err(e) = audit_result {
        tracing::debug!("Audit service call failed (continuing): {}", e);
    }
}

async fn ensure_user_id_available(
    svc: &Service,
    scope: &AccessScope,
    id: Uuid,
) -> Result<(), DomainError> {
    let found = svc
        .sec
        .find_by_id::<UserEntity>(scope, id)
        .map_err(|e| DomainError::database(e.to_string()))?
        .one(svc.sec.conn())
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    if found.is_some() {
        return Err(DomainError::validation(
            "id",
            "User with this ID already exists",
        ));
    }

    Ok(())
}

async fn ensure_email_unique(
    svc: &Service,
    scope: &AccessScope,
    email: &str,
) -> Result<(), DomainError> {
    let secure_query = svc
        .sec
        .find::<UserEntity>(scope)
        .filter(sea_orm::Condition::all().add(Expr::col(Column::Email).eq(email)));

    let count = secure_query
        .count(svc.sec.conn())
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    if count > 0 {
        return Err(DomainError::email_already_exists(email.to_owned()));
    }

    Ok(())
}

pub(super) async fn get_user(
    svc: &Service,
    ctx: &SecurityContext,
    id: Uuid,
) -> Result<User, DomainError> {
    tracing::debug!("Getting user by id");

    audit_get_user_access_best_effort(svc, id).await;

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let found = svc
        .sec
        .find_by_id::<UserEntity>(&scope, id)
        .map_err(|e| DomainError::database(e.to_string()))?
        .one(svc.sec.conn())
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    let user = found
        .map(Into::into)
        .ok_or_else(|| DomainError::user_not_found(id))?;

    tracing::debug!("Successfully retrieved user");
    Ok(user)
}

pub(super) async fn list_users_page(
    svc: &Service,
    ctx: &SecurityContext,
    query: &ODataQuery,
) -> Result<Page<User>, DomainError> {
    tracing::debug!("Listing users with cursor pagination");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let secure_query = svc.sec.find::<UserEntity>(&scope);
    let base_query = secure_query.into_inner();

    let page = paginate_odata::<UserFilterField, UserODataMapper, _, _, _, _>(
        base_query,
        svc.sec.conn(),
        query,
        ("id", SortDir::Desc),
        LimitCfg {
            default: u64::from(svc.config.default_page_size),
            max: u64::from(svc.config.max_page_size),
        },
        Into::into,
    )
    .await
    .map_err(|e| DomainError::database(e.to_string()))?;

    tracing::debug!("Successfully listed {} users in page", page.items.len());
    Ok(page)
}

#[allow(clippy::cognitive_complexity)]
pub(super) async fn create_user(
    svc: &Service,
    ctx: &SecurityContext,
    new_user: NewUser,
) -> Result<User, DomainError> {
    tracing::info!("Creating new user");

    svc.validate_new_user(&new_user)?;

    let NewUser {
        id: provided_id,
        tenant_id,
        email,
        display_name,
    } = new_user;

    let id = provided_id.unwrap_or_else(Uuid::now_v7);

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    if provided_id.is_some() {
        ensure_user_id_available(svc, &scope, id).await?;
    }

    ensure_email_unique(svc, &scope, &email).await?;

    let now = OffsetDateTime::now_utc();

    let user = User {
        id,
        tenant_id,
        email,
        display_name,
        created_at: now,
        updated_at: now,
    };

    let m = UserAM {
        id: Set(user.id),
        tenant_id: Set(user.tenant_id),
        email: Set(user.email.clone()),
        display_name: Set(user.display_name.clone()),
        created_at: Set(user.created_at),
        updated_at: Set(user.updated_at),
    };

    let _ = svc
        .sec
        .insert::<UserEntity>(&scope, m)
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    let notification_result = svc.audit.notify_user_created().await;
    if let Err(e) = notification_result {
        tracing::debug!("Notification service call failed (continuing): {}", e);
    }

    svc.events.publish(&UserDomainEvent::Created {
        id: user.id,
        at: user.created_at,
    });

    tracing::info!("Successfully created user with id={}", user.id);
    Ok(user)
}

pub(super) async fn update_user(
    svc: &Service,
    ctx: &SecurityContext,
    id: Uuid,
    patch: UserPatch,
) -> Result<User, DomainError> {
    tracing::info!("Updating user");

    svc.validate_user_patch(&patch)?;

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let found = svc
        .sec
        .find_by_id::<UserEntity>(&scope, id)
        .map_err(|e| DomainError::database(e.to_string()))?
        .one(svc.sec.conn())
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    let mut current: User = found.ok_or_else(|| DomainError::user_not_found(id))?.into();

    if let Some(ref new_email) = patch.email {
        if new_email != &current.email {
            let secure_query = svc
                .sec
                .find::<UserEntity>(&scope)
                .filter(sea_orm::Condition::all().add(Expr::col(Column::Email).eq(new_email)));

            let count = secure_query
                .count(svc.sec.conn())
                .await
                .map_err(|e| DomainError::database(e.to_string()))?;

            if count > 0 {
                return Err(DomainError::email_already_exists(new_email.clone()));
            }
        }
    }

    if let Some(email) = patch.email {
        current.email = email;
    }
    if let Some(display_name) = patch.display_name {
        current.display_name = display_name;
    }
    current.updated_at = OffsetDateTime::now_utc();

    let m = UserAM {
        id: Set(current.id),
        tenant_id: Set(current.tenant_id),
        email: Set(current.email.clone()),
        display_name: Set(current.display_name.clone()),
        created_at: Set(current.created_at),
        updated_at: Set(current.updated_at),
    };

    let _ = svc
        .sec
        .update_with_ctx::<UserEntity>(&scope, current.id, m)
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    svc.events.publish(&UserDomainEvent::Updated {
        id: current.id,
        at: current.updated_at,
    });

    tracing::info!("Successfully updated user");
    Ok(current)
}

pub(super) async fn delete_user(
    svc: &Service,
    ctx: &SecurityContext,
    id: Uuid,
) -> Result<(), DomainError> {
    tracing::info!("Deleting user");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let deleted = svc
        .sec
        .delete_by_id::<UserEntity>(&scope, id)
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    if !deleted {
        return Err(DomainError::user_not_found(id));
    }

    svc.events.publish(&UserDomainEvent::Deleted {
        id,
        at: OffsetDateTime::now_utc(),
    });

    tracing::info!("Successfully deleted user");
    Ok(())
}
