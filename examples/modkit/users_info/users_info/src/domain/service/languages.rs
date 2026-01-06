use super::{
    debug, info, paginate_odata, DomainError, Language, LanguageAM, LanguageEntity,
    LanguageFilterField, LanguageODataMapper, LanguagePatch, LimitCfg, NewLanguage, ODataQuery,
    OffsetDateTime, Page, SecurityContext, Service, Set, SortDir, Uuid,
};

pub(super) async fn get_language(
    svc: &Service,
    ctx: &SecurityContext,
    id: Uuid,
) -> Result<Language, DomainError> {
    debug!("Getting language by id");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let found = svc
        .sec
        .find_by_id::<LanguageEntity>(&scope, id)
        .map_err(|e| DomainError::database(e.to_string()))?
        .one(svc.sec.conn())
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    found
        .map(Into::into)
        .ok_or_else(|| DomainError::not_found("Language", id))
}

pub(super) async fn list_languages_page(
    svc: &Service,
    ctx: &SecurityContext,
    query: &ODataQuery,
) -> Result<Page<Language>, DomainError> {
    debug!("Listing languages with cursor pagination");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let secure_query = svc.sec.find::<LanguageEntity>(&scope);
    let base_query = secure_query.into_inner();

    let page = paginate_odata::<LanguageFilterField, LanguageODataMapper, _, _, _, _>(
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

    debug!("Successfully listed {} languages in page", page.items.len());
    Ok(page)
}

pub(super) async fn create_language(
    svc: &Service,
    ctx: &SecurityContext,
    new_language: NewLanguage,
) -> Result<Language, DomainError> {
    info!("Creating new language");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let now = OffsetDateTime::now_utc();
    let id = new_language.id.unwrap_or_else(Uuid::now_v7);

    let language = Language {
        id,
        tenant_id: new_language.tenant_id,
        code: new_language.code,
        name: new_language.name,
        created_at: now,
        updated_at: now,
    };

    let m = LanguageAM {
        id: Set(language.id),
        tenant_id: Set(language.tenant_id),
        code: Set(language.code.clone()),
        name: Set(language.name.clone()),
        created_at: Set(language.created_at),
        updated_at: Set(language.updated_at),
    };

    let _ = svc
        .sec
        .insert::<LanguageEntity>(&scope, m)
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    info!("Successfully created language with id={}", language.id);
    Ok(language)
}

pub(super) async fn update_language(
    svc: &Service,
    ctx: &SecurityContext,
    id: Uuid,
    patch: LanguagePatch,
) -> Result<Language, DomainError> {
    info!("Updating language");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let found = svc
        .sec
        .find_by_id::<LanguageEntity>(&scope, id)
        .map_err(|e| DomainError::database(e.to_string()))?
        .one(svc.sec.conn())
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    let mut current: Language = found
        .ok_or_else(|| DomainError::not_found("Language", id))?
        .into();

    if let Some(code) = patch.code {
        current.code = code;
    }
    if let Some(name) = patch.name {
        current.name = name;
    }
    current.updated_at = OffsetDateTime::now_utc();

    let m = LanguageAM {
        id: Set(current.id),
        tenant_id: Set(current.tenant_id),
        code: Set(current.code.clone()),
        name: Set(current.name.clone()),
        created_at: Set(current.created_at),
        updated_at: Set(current.updated_at),
    };

    let _ = svc
        .sec
        .update_with_ctx::<LanguageEntity>(&scope, current.id, m)
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    info!("Successfully updated language");
    Ok(current)
}

pub(super) async fn delete_language(
    svc: &Service,
    ctx: &SecurityContext,
    id: Uuid,
) -> Result<(), DomainError> {
    info!("Deleting language");

    let scope = ctx
        .scope(svc.policy_engine.clone())
        .include_tenant_children()
        .prepare()
        .await?;

    let deleted = svc
        .sec
        .delete_by_id::<LanguageEntity>(&scope, id)
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

    if !deleted {
        return Err(DomainError::not_found("Language", id));
    }

    info!("Successfully deleted language");
    Ok(())
}
