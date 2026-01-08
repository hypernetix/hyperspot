use super::{
    debug, info, paginate_odata, City, CityAM, CityEntity, CityFilterField, CityODataMapper,
    CityPatch, DomainError, NewCity, ODataQuery, OffsetDateTime, Page, SecurityContext, Service,
    Set, SortDir, Uuid,
};

pub(super) async fn get_city(
    svc: &Service,
    ctx: &SecurityContext,
    id: Uuid,
) -> Result<City, DomainError> {
    debug!("Getting city by id");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let found = svc
        .sec
        .find_by_id::<CityEntity>(&scope, id)
        .map_err(|e| DomainError::database(e.to_string()))?
        .one(svc.sec.conn())
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    found
        .map(Into::into)
        .ok_or_else(|| DomainError::not_found("City", id))
}

pub(super) async fn list_cities_page(
    svc: &Service,
    ctx: &SecurityContext,
    query: &ODataQuery,
) -> Result<Page<City>, DomainError> {
    debug!("Listing cities with cursor pagination");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let secure_query = svc.sec.find::<CityEntity>(&scope);
    let base_query = secure_query.into_inner();

    let page = paginate_odata::<CityFilterField, CityODataMapper, _, _, _, _>(
        base_query,
        svc.sec.conn(),
        query,
        ("id", SortDir::Desc),
        svc.limit_cfg(),
        Into::into,
    )
    .await
    .map_err(|e| DomainError::database(e.to_string()))?;

    debug!("Successfully listed {} cities in page", page.items.len());
    Ok(page)
}

pub(super) async fn create_city(
    svc: &Service,
    ctx: &SecurityContext,
    new_city: NewCity,
) -> Result<City, DomainError> {
    info!("Creating new city");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let now = OffsetDateTime::now_utc();
    let id = new_city.id.unwrap_or_else(Uuid::now_v7);

    let city = City {
        id,
        tenant_id: new_city.tenant_id,
        name: new_city.name,
        country: new_city.country,
        created_at: now,
        updated_at: now,
    };

    let m = CityAM {
        id: Set(city.id),
        tenant_id: Set(city.tenant_id),
        name: Set(city.name.clone()),
        country: Set(city.country.clone()),
        created_at: Set(city.created_at),
        updated_at: Set(city.updated_at),
    };

    let _ = svc
        .sec
        .insert::<CityEntity>(&scope, m)
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    info!("Successfully created city with id={}", city.id);
    Ok(city)
}

pub(super) async fn update_city(
    svc: &Service,
    ctx: &SecurityContext,
    id: Uuid,
    patch: CityPatch,
) -> Result<City, DomainError> {
    info!("Updating city");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let found = svc
        .sec
        .find_by_id::<CityEntity>(&scope, id)
        .map_err(|e| DomainError::database(e.to_string()))?
        .one(svc.sec.conn())
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    let mut current: City = found
        .ok_or_else(|| DomainError::not_found("City", id))?
        .into();

    if let Some(name) = patch.name {
        current.name = name;
    }
    if let Some(country) = patch.country {
        current.country = country;
    }
    current.updated_at = OffsetDateTime::now_utc();

    let m = CityAM {
        id: Set(current.id),
        tenant_id: Set(current.tenant_id),
        name: Set(current.name.clone()),
        country: Set(current.country.clone()),
        created_at: Set(current.created_at),
        updated_at: Set(current.updated_at),
    };

    let _ = svc
        .sec
        .update_with_ctx::<CityEntity>(&scope, current.id, m)
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    info!("Successfully updated city");
    Ok(current)
}

pub(super) async fn delete_city(
    svc: &Service,
    ctx: &SecurityContext,
    id: Uuid,
) -> Result<(), DomainError> {
    info!("Deleting city");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let deleted = svc
        .sec
        .delete_by_id::<CityEntity>(&scope, id)
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    if !deleted {
        return Err(DomainError::not_found("City", id));
    }

    info!("Successfully deleted city");
    Ok(())
}
