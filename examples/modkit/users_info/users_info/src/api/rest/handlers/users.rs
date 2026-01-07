use axum::http::Uri;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use super::{
    apply_select, created_json, info, no_content, page_to_projected_json, ApiResult, Json,
    JsonBody, JsonPage, SecurityContext, UpdateUserReq, UserDto, UserFullDto,
};
use crate::module::ConcreteAppServices;

pub(super) async fn list_users(
    ctx: SecurityContext,
    svc: std::sync::Arc<ConcreteAppServices>,
    query: modkit::api::odata::ODataQuery,
) -> ApiResult<JsonPage<serde_json::Value>> {
    info!(
        user_id = %ctx.subject_id(),
        "Listing users with cursor pagination"
    );

    let page: modkit_odata::Page<user_info_sdk::User> =
        svc.users.list_users_page(&ctx, &query).await?;
    let page = page.map_items(UserDto::from);

    Ok(Json(page_to_projected_json(&page, query.selected_fields())))
}

pub(super) async fn get_user(
    ctx: SecurityContext,
    svc: std::sync::Arc<ConcreteAppServices>,
    id: Uuid,
    query: modkit::api::odata::ODataQuery,
) -> ApiResult<JsonBody<serde_json::Value>> {
    info!(
        user_id = %id,
        requester_id = %ctx.subject_id(),
        "Getting user details with related entities"
    );

    let user_full = svc.users.get_user_full(&ctx, id).await?;
    let user_full_dto = UserFullDto::from(user_full);
    let projected = apply_select(&user_full_dto, query.selected_fields());
    Ok(Json(projected))
}

pub(super) async fn create_user(
    uri: Uri,
    ctx: SecurityContext,
    svc: std::sync::Arc<ConcreteAppServices>,
    new_user: user_info_sdk::NewUser,
) -> ApiResult<Response> {
    let user = svc.users.create_user(&ctx, new_user).await?;
    let id_str = user.id.to_string();
    Ok(created_json(UserDto::from(user), &uri, &id_str).into_response())
}

pub(super) async fn update_user(
    ctx: SecurityContext,
    svc: std::sync::Arc<ConcreteAppServices>,
    id: Uuid,
    req_body: UpdateUserReq,
) -> ApiResult<JsonBody<UserDto>> {
    info!(
        user_id = %id,
        updater_id = %ctx.subject_id(),
        "Updating user"
    );

    let patch = req_body.into();
    let user = svc.users.update_user(&ctx, id, patch).await?;
    Ok(Json(UserDto::from(user)))
}

pub(super) async fn delete_user(
    ctx: SecurityContext,
    svc: std::sync::Arc<ConcreteAppServices>,
    id: Uuid,
) -> ApiResult<Response> {
    info!(
        user_id = %id,
        deleter_id = %ctx.subject_id(),
        "Deleting user"
    );

    svc.users.delete_user(&ctx, id).await?;
    Ok(no_content().into_response())
}
