use super::{
    debug, info, DomainError, Expr, Language, LanguageColumn, LanguageEntity, OffsetDateTime,
    SecurityContext, Service, Set, UserEntity, UserLanguageAM, UserLanguageColumn,
    UserLanguageEntity, Uuid,
};

#[allow(clippy::cognitive_complexity)]
pub(super) async fn assign_language_to_user(
    svc: &Service,
    ctx: &SecurityContext,
    user_id: Uuid,
    language_id: Uuid,
) -> Result<(), DomainError> {
    info!("Assigning language to user (idempotent)");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let user = svc
        .sec
        .find_by_id::<UserEntity>(&scope, user_id)
        .map_err(|e| DomainError::database(e.to_string()))?
        .one(svc.sec.conn())
        .await
        .map_err(|e| DomainError::database(e.to_string()))?
        .ok_or_else(|| DomainError::user_not_found(user_id))?;

    let _language = svc
        .sec
        .find_by_id::<LanguageEntity>(&scope, language_id)
        .map_err(|e| DomainError::database(e.to_string()))?
        .one(svc.sec.conn())
        .await
        .map_err(|e| DomainError::database(e.to_string()))?
        .ok_or_else(|| DomainError::not_found("Language", language_id))?;

    let existing = svc
        .sec
        .find::<UserLanguageEntity>(&scope)
        .filter(
            sea_orm::Condition::all()
                .add(Expr::col(UserLanguageColumn::UserId).eq(user_id))
                .add(Expr::col(UserLanguageColumn::LanguageId).eq(language_id)),
        )
        .one(svc.sec.conn())
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    if existing.is_some() {
        debug!("Language already assigned to user, operation is idempotent");
        return Ok(());
    }

    let now = OffsetDateTime::now_utc();
    let id = Uuid::now_v7();

    let m = UserLanguageAM {
        id: Set(id),
        tenant_id: Set(user.tenant_id),
        user_id: Set(user_id),
        language_id: Set(language_id),
        created_at: Set(now),
    };

    let _ = svc
        .sec
        .insert::<UserLanguageEntity>(&scope, m)
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    info!("Successfully assigned language to user");
    Ok(())
}

pub(super) async fn remove_language_from_user(
    svc: &Service,
    ctx: &SecurityContext,
    user_id: Uuid,
    language_id: Uuid,
) -> Result<(), DomainError> {
    info!("Removing language from user (idempotent)");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let result = svc
        .sec
        .delete_many::<UserLanguageEntity>(&scope)
        .filter(
            sea_orm::Condition::all()
                .add(Expr::col(UserLanguageColumn::UserId).eq(user_id))
                .add(Expr::col(UserLanguageColumn::LanguageId).eq(language_id)),
        )
        .exec(svc.sec.conn())
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    if result.rows_affected == 0 {
        debug!("Language not assigned to user, operation is idempotent");
    } else {
        info!("Successfully removed language from user");
    }

    Ok(())
}

pub(super) async fn list_user_languages(
    svc: &Service,
    ctx: &SecurityContext,
    user_id: Uuid,
) -> Result<Vec<Language>, DomainError> {
    debug!("Listing languages for user");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let user_languages = svc
        .sec
        .find::<UserLanguageEntity>(&scope)
        .filter(sea_orm::Condition::all().add(Expr::col(UserLanguageColumn::UserId).eq(user_id)))
        .all(svc.sec.conn())
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    let language_ids: Vec<Uuid> = user_languages.iter().map(|ul| ul.language_id).collect();

    if language_ids.is_empty() {
        return Ok(Vec::new());
    }

    let languages = svc
        .sec
        .find::<LanguageEntity>(&scope)
        .filter(sea_orm::Condition::all().add(Expr::col(LanguageColumn::Id).is_in(language_ids)))
        .all(svc.sec.conn())
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    Ok(languages.into_iter().map(Into::into).collect())
}
