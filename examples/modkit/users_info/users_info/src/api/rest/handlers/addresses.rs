use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use super::{
    info, no_content, AddressDto, ApiResult, Json, JsonBody, PutAddressReq, SecurityContext,
};
use crate::module::ConcreteAppServices;

pub(super) async fn get_user_address(
    ctx: SecurityContext,
    svc: std::sync::Arc<ConcreteAppServices>,
    user_id: Uuid,
) -> ApiResult<JsonBody<AddressDto>> {
    info!(
        user_id = %user_id,
        requester_id = %ctx.subject_id(),
        "Getting user address"
    );

    let address: Option<user_info_sdk::Address> =
        svc.addresses.get_user_address(&ctx, user_id).await?;

    let address =
        address.ok_or_else(|| crate::domain::error::DomainError::not_found("Address", user_id))?;

    Ok(Json(AddressDto::from(address)))
}

pub(super) async fn put_user_address(
    ctx: SecurityContext,
    svc: std::sync::Arc<ConcreteAppServices>,
    user_id: Uuid,
    req_body: PutAddressReq,
) -> ApiResult<Response> {
    info!(
        user_id = %user_id,
        updater_id = %ctx.subject_id(),
        "Upserting user address"
    );

    let new_address = req_body.into_new_address(user_id);
    let address = svc
        .addresses
        .put_user_address(&ctx, user_id, new_address)
        .await?;

    Ok(Json(AddressDto::from(address)).into_response())
}

pub(super) async fn delete_user_address(
    ctx: SecurityContext,
    svc: std::sync::Arc<ConcreteAppServices>,
    user_id: Uuid,
) -> ApiResult<Response> {
    info!(
        user_id = %user_id,
        deleter_id = %ctx.subject_id(),
        "Deleting user address"
    );

    svc.addresses.delete_user_address(&ctx, user_id).await?;
    Ok(no_content().into_response())
}
