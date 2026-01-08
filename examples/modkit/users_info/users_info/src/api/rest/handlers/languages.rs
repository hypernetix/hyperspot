use axum::http::Uri;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use super::{
    apply_select, created_json, info, no_content, page_to_projected_json, ApiResult,
    CreateLanguageReq, Json, JsonBody, JsonPage, LanguageDto, SecurityContext, Service,
    UpdateLanguageReq,
};

pub(super) async fn list_languages(
    ctx: SecurityContext,
    svc: std::sync::Arc<Service>,
    query: modkit::api::odata::ODataQuery,
) -> ApiResult<JsonPage<serde_json::Value>> {
    info!(
        user_id = %ctx.subject_id(),
        "Listing languages with cursor pagination"
    );

    let page = svc
        .list_languages_page(&ctx, &query)
        .await?
        .map_items(LanguageDto::from);

    Ok(Json(page_to_projected_json(&page, query.selected_fields())))
}

pub(super) async fn get_language(
    ctx: SecurityContext,
    svc: std::sync::Arc<Service>,
    id: Uuid,
    query: modkit::api::odata::ODataQuery,
) -> ApiResult<JsonBody<serde_json::Value>> {
    info!(
        language_id = %id,
        requester_id = %ctx.subject_id(),
        "Getting language details"
    );

    let language = svc.get_language(&ctx, id).await?;
    let language_dto = LanguageDto::from(language);

    let projected = apply_select(&language_dto, query.selected_fields());

    Ok(Json(projected))
}

pub(super) async fn create_language(
    uri: Uri,
    ctx: SecurityContext,
    svc: std::sync::Arc<Service>,
    req_body: CreateLanguageReq,
) -> ApiResult<Response> {
    info!(
        code = %req_body.code,
        name = %req_body.name,
        tenant_id = %req_body.tenant_id,
        creator_id = %ctx.subject_id(),
        "Creating new language"
    );

    let new_language = req_body.into();
    let language = svc.create_language(&ctx, new_language).await?;
    let id_str = language.id.to_string();
    Ok(created_json(LanguageDto::from(language), &uri, &id_str).into_response())
}

pub(super) async fn update_language(
    ctx: SecurityContext,
    svc: std::sync::Arc<Service>,
    id: Uuid,
    req_body: UpdateLanguageReq,
) -> ApiResult<JsonBody<LanguageDto>> {
    info!(
        language_id = %id,
        updater_id = %ctx.subject_id(),
        "Updating language"
    );

    let patch = req_body.into();
    let language = svc.update_language(&ctx, id, patch).await?;
    Ok(Json(LanguageDto::from(language)))
}

pub(super) async fn delete_language(
    ctx: SecurityContext,
    svc: std::sync::Arc<Service>,
    id: Uuid,
) -> ApiResult<Response> {
    info!(
        language_id = %id,
        deleter_id = %ctx.subject_id(),
        "Deleting language"
    );

    svc.delete_language(&ctx, id).await?;
    Ok(no_content().into_response())
}
