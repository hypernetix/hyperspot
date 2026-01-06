use super::{
    debug, info, Address, AddressAM, AddressColumn, AddressEntity, AddressPatch, DomainError, Expr,
    NewAddress, OffsetDateTime, SecurityContext, Service, Set, UserEntity, Uuid,
};

pub(super) async fn get_address(
    svc: &Service,
    ctx: &SecurityContext,
    id: Uuid,
) -> Result<Address, DomainError> {
    debug!("Getting address by id");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let found = svc
        .sec
        .find_by_id::<AddressEntity>(&scope, id)
        .map_err(|e| DomainError::database(e.to_string()))?
        .one(svc.sec.conn())
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    found
        .map(Into::into)
        .ok_or_else(|| DomainError::not_found("Address", id))
}

pub(super) async fn get_user_address(
    svc: &Service,
    ctx: &SecurityContext,
    user_id: Uuid,
) -> Result<Option<Address>, DomainError> {
    debug!("Getting address by user_id");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let found = svc
        .sec
        .find::<AddressEntity>(&scope)
        .filter(sea_orm::Condition::all().add(Expr::col(AddressColumn::UserId).eq(user_id)))
        .one(svc.sec.conn())
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    Ok(found.map(Into::into))
}

pub(super) async fn get_address_by_user(
    svc: &Service,
    ctx: &SecurityContext,
    user_id: Uuid,
) -> Result<Option<Address>, DomainError> {
    get_user_address(svc, ctx, user_id).await
}

#[allow(clippy::cognitive_complexity)]
pub(super) async fn put_user_address(
    svc: &Service,
    ctx: &SecurityContext,
    user_id: Uuid,
    address: NewAddress,
) -> Result<Address, DomainError> {
    info!("Upserting address for user");

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

    let existing = svc
        .sec
        .find::<AddressEntity>(&scope)
        .filter(sea_orm::Condition::all().add(Expr::col(AddressColumn::UserId).eq(user_id)))
        .one(svc.sec.conn())
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    let now = OffsetDateTime::now_utc();

    if let Some(existing_model) = existing {
        let mut updated: Address = existing_model.into();
        updated.city_id = address.city_id;
        updated.street = address.street;
        updated.postal_code = address.postal_code;
        updated.updated_at = now;

        let m = AddressAM {
            id: Set(updated.id),
            tenant_id: Set(updated.tenant_id),
            user_id: Set(updated.user_id),
            city_id: Set(updated.city_id),
            street: Set(updated.street.clone()),
            postal_code: Set(updated.postal_code.clone()),
            created_at: Set(updated.created_at),
            updated_at: Set(updated.updated_at),
        };

        let _ = svc
            .sec
            .update_with_ctx::<AddressEntity>(&scope, updated.id, m)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        info!("Successfully updated address for user");
        Ok(updated)
    } else {
        let id = address.id.unwrap_or_else(Uuid::now_v7);

        let new_address = Address {
            id,
            tenant_id: user.tenant_id,
            user_id,
            city_id: address.city_id,
            street: address.street,
            postal_code: address.postal_code,
            created_at: now,
            updated_at: now,
        };

        let m = AddressAM {
            id: Set(new_address.id),
            tenant_id: Set(new_address.tenant_id),
            user_id: Set(new_address.user_id),
            city_id: Set(new_address.city_id),
            street: Set(new_address.street.clone()),
            postal_code: Set(new_address.postal_code.clone()),
            created_at: Set(new_address.created_at),
            updated_at: Set(new_address.updated_at),
        };

        let _ = svc
            .sec
            .insert::<AddressEntity>(&scope, m)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        info!("Successfully created address for user");
        Ok(new_address)
    }
}

pub(super) async fn delete_user_address(
    svc: &Service,
    ctx: &SecurityContext,
    user_id: Uuid,
) -> Result<(), DomainError> {
    info!("Deleting address for user");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let result = svc
        .sec
        .delete_many::<AddressEntity>(&scope)
        .filter(sea_orm::Condition::all().add(Expr::col(AddressColumn::UserId).eq(user_id)))
        .exec(svc.sec.conn())
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    if result.rows_affected == 0 {
        return Err(DomainError::not_found("Address", user_id));
    }

    info!("Successfully deleted address for user");
    Ok(())
}

pub(super) async fn create_address(
    svc: &Service,
    ctx: &SecurityContext,
    new_address: NewAddress,
) -> Result<Address, DomainError> {
    info!("Creating new address");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let now = OffsetDateTime::now_utc();
    let id = new_address.id.unwrap_or_else(Uuid::now_v7);

    let address = Address {
        id,
        tenant_id: new_address.tenant_id,
        user_id: new_address.user_id,
        city_id: new_address.city_id,
        street: new_address.street,
        postal_code: new_address.postal_code,
        created_at: now,
        updated_at: now,
    };

    let m = AddressAM {
        id: Set(address.id),
        tenant_id: Set(address.tenant_id),
        user_id: Set(address.user_id),
        city_id: Set(address.city_id),
        street: Set(address.street.clone()),
        postal_code: Set(address.postal_code.clone()),
        created_at: Set(address.created_at),
        updated_at: Set(address.updated_at),
    };

    let _ = svc
        .sec
        .insert::<AddressEntity>(&scope, m)
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    info!("Successfully created address with id={}", address.id);
    Ok(address)
}

pub(super) async fn update_address(
    svc: &Service,
    ctx: &SecurityContext,
    id: Uuid,
    patch: AddressPatch,
) -> Result<Address, DomainError> {
    info!("Updating address");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let found = svc
        .sec
        .find_by_id::<AddressEntity>(&scope, id)
        .map_err(|e| DomainError::database(e.to_string()))?
        .one(svc.sec.conn())
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    let mut current: Address = found
        .ok_or_else(|| DomainError::not_found("Address", id))?
        .into();

    if let Some(city_id) = patch.city_id {
        current.city_id = city_id;
    }
    if let Some(street) = patch.street {
        current.street = street;
    }
    if let Some(postal_code) = patch.postal_code {
        current.postal_code = postal_code;
    }
    current.updated_at = OffsetDateTime::now_utc();

    let m = AddressAM {
        id: Set(current.id),
        tenant_id: Set(current.tenant_id),
        user_id: Set(current.user_id),
        city_id: Set(current.city_id),
        street: Set(current.street.clone()),
        postal_code: Set(current.postal_code.clone()),
        created_at: Set(current.created_at),
        updated_at: Set(current.updated_at),
    };

    let _ = svc
        .sec
        .update_with_ctx::<AddressEntity>(&scope, current.id, m)
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    info!("Successfully updated address");
    Ok(current)
}

pub(super) async fn delete_address(
    svc: &Service,
    ctx: &SecurityContext,
    id: Uuid,
) -> Result<(), DomainError> {
    info!("Deleting address");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let deleted = svc
        .sec
        .delete_by_id::<AddressEntity>(&scope, id)
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    if !deleted {
        return Err(DomainError::not_found("Address", id));
    }

    info!("Successfully deleted address");
    Ok(())
}
