use axum::http::Uri;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use super::{
    apply_select, created_json, info, no_content, page_to_projected_json, ApiResult, Json,
    JsonBody, JsonPage, SecurityContext, Service, UpdateUserReq, UserDto,
};

pub(super) async fn list_users(
    ctx: SecurityContext,
    svc: std::sync::Arc<Service>,
    query: modkit::api::odata::ODataQuery,
) -> ApiResult<JsonPage<serde_json::Value>> {
    info!(
        user_id = %ctx.subject_id(),
        "Listing users with cursor pagination"
    );

    let page = svc
        .list_users_page(&ctx, &query)
        .await?
        .map_items(UserDto::from);

    Ok(Json(page_to_projected_json(&page, query.selected_fields())))
}

pub(super) async fn get_user(
    ctx: SecurityContext,
    svc: std::sync::Arc<Service>,
    id: Uuid,
    query: modkit::api::odata::ODataQuery,
) -> ApiResult<JsonBody<serde_json::Value>> {
    info!(
        user_id = %id,
        requester_id = %ctx.subject_id(),
        "Getting user details"
    );

    let user = svc.get_user(&ctx, id).await?;
    let user_dto = UserDto::from(user);

    let projected = apply_select(&user_dto, query.selected_fields());

    Ok(Json(projected))
}

pub(super) async fn create_user(
    uri: Uri,
    ctx: SecurityContext,
    svc: std::sync::Arc<Service>,
    new_user: user_info_sdk::NewUser,
) -> ApiResult<Response> {
    let user = svc.create_user(&ctx, new_user).await?;
    let id_str = user.id.to_string();
    Ok(created_json(UserDto::from(user), &uri, &id_str).into_response())
}

pub(super) async fn update_user(
    ctx: SecurityContext,
    svc: std::sync::Arc<Service>,
    id: Uuid,
    req_body: UpdateUserReq,
) -> ApiResult<JsonBody<UserDto>> {
    info!(
        user_id = %id,
        updater_id = %ctx.subject_id(),
        "Updating user"
    );

    let patch = req_body.into();
    let user = svc.update_user(&ctx, id, patch).await?;
    Ok(Json(UserDto::from(user)))
}

pub(super) async fn delete_user(
    ctx: SecurityContext,
    svc: std::sync::Arc<Service>,
    id: Uuid,
) -> ApiResult<Response> {
    info!(
        user_id = %id,
        deleter_id = %ctx.subject_id(),
        "Deleting user"
    );

    svc.delete_user(&ctx, id).await?;
    Ok(no_content().into_response())
}
