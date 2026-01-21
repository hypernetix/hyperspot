use axum::http::Uri;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use super::{
    ApiResult, CityDto, CreateCityReq, Json, JsonBody, JsonPage, SecurityContext, UpdateCityReq,
    apply_select, created_json, info, no_content, page_to_projected_json,
};
use crate::module::ConcreteAppServices;

pub(super) async fn list_cities(
    ctx: SecurityContext,
    svc: std::sync::Arc<ConcreteAppServices>,
    query: modkit::api::odata::ODataQuery,
) -> ApiResult<JsonPage<serde_json::Value>> {
    info!(
        user_id = %ctx.subject_id(),
        "Listing cities with cursor pagination"
    );

    let page: modkit_odata::Page<user_info_sdk::City> =
        svc.cities.list_cities_page(&ctx, &query).await?;
    let page = page.map_items(CityDto::from);

    Ok(Json(page_to_projected_json(&page, query.selected_fields())))
}

pub(super) async fn get_city(
    ctx: SecurityContext,
    svc: std::sync::Arc<ConcreteAppServices>,
    id: Uuid,
    query: modkit::api::odata::ODataQuery,
) -> ApiResult<JsonBody<serde_json::Value>> {
    info!(
        city_id = %id,
        requester_id = %ctx.subject_id(),
        "Getting city details"
    );

    let city = svc.cities.get_city(&ctx, id).await?;
    let city_dto = CityDto::from(city);

    let projected = apply_select(&city_dto, query.selected_fields());

    Ok(Json(projected))
}

pub(super) async fn create_city(
    uri: Uri,
    ctx: SecurityContext,
    svc: std::sync::Arc<ConcreteAppServices>,
    req_body: CreateCityReq,
) -> ApiResult<Response> {
    info!(
        name = %req_body.name,
        country = %req_body.country,
        tenant_id = %req_body.tenant_id,
        creator_id = %ctx.subject_id(),
        "Creating new city"
    );

    let new_city = req_body.into();
    let city = svc.cities.create_city(&ctx, new_city).await?;
    let id_str = city.id.to_string();
    Ok(created_json(CityDto::from(city), &uri, &id_str).into_response())
}

pub(super) async fn update_city(
    ctx: SecurityContext,
    svc: std::sync::Arc<ConcreteAppServices>,
    id: Uuid,
    req_body: UpdateCityReq,
) -> ApiResult<JsonBody<CityDto>> {
    info!(
        city_id = %id,
        updater_id = %ctx.subject_id(),
        "Updating city"
    );

    let patch = req_body.into();
    let city = svc.cities.update_city(&ctx, id, patch).await?;
    Ok(Json(CityDto::from(city)))
}

pub(super) async fn delete_city(
    ctx: SecurityContext,
    svc: std::sync::Arc<ConcreteAppServices>,
    id: Uuid,
) -> ApiResult<Response> {
    info!(
        city_id = %id,
        deleter_id = %ctx.subject_id(),
        "Deleting city"
    );

    svc.cities.delete_city(&ctx, id).await?;
    Ok(no_content().into_response())
}
