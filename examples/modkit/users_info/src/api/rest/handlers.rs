use axum::{extract::Path, Extension};
use tracing::{field::Empty, info};
use uuid::Uuid;

use crate::api::rest::dto::{CreateUserReq, UpdateUserReq, UserDto, UserEvent};

use modkit::api::odata::OData;
use modkit::api::prelude::*;

use crate::domain::service::Service;
use modkit::SseBroadcaster;

// Import auth extractors
use modkit_auth::axum_ext::Authz;

// Type aliases for our specific API with DomainError
use crate::domain::error::DomainError;
type UsersResult<T> = ApiResult<T, DomainError>;
type UsersApiError = ApiError<DomainError>;

/// List users with cursor-based pagination
#[tracing::instrument(
    name = "users_info.list_users",
    skip(svc, query, ctx),
    fields(
        limit = query.limit,
        request_id = Empty,
        user.id = %ctx.subject_id()
    )
)]
pub async fn list_users(
    Authz(ctx): Authz,                         // ← Validated SecurityCtx from middleware
    Extension(svc): Extension<std::sync::Arc<Service>>,
    OData(query): OData,
) -> UsersResult<JsonPage<UserDto>> {
    info!(
        user_id = %ctx.subject_id(),
        "Listing users with cursor pagination"
    );

    // Pass the validated SecurityCtx to service; secure-ORM will apply tenant scope
    let page = svc
        .list_users_page(&ctx, query)
        .await?
        .map_items(UserDto::from);
    Ok(Json(page))
}

/// Get a specific user by ID
#[tracing::instrument(
    name = "users_info.get_user",
    skip(svc, ctx),
    fields(
        user.id = %id,
        request_id = Empty,
        requester.id = %ctx.subject_id()
    )
)]
pub async fn get_user(
    Authz(ctx): Authz,                         // ← Validated SecurityCtx
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path(id): Path<Uuid>,
) -> UsersResult<JsonBody<UserDto>> {
    info!(
        user_id = %id,
        requester_id = %ctx.subject_id(),
        "Getting user details"
    );

    let user = svc
        .get_user(&ctx, id)
        .await
        .map_err(UsersApiError::from_domain)?;
    Ok(Json(UserDto::from(user)))
}

/// Create a new user
#[tracing::instrument(
    name = "users_info.create_user",
    skip(svc, req_body, ctx),
    fields(
        user.email = %req_body.email,
        user.display_name = %req_body.display_name,
        request_id = Empty,
        creator.id = %ctx.subject_id()
    )
)]
pub async fn create_user(
    Authz(ctx): Authz,                         // ← Validated SecurityCtx
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Json(req_body): Json<CreateUserReq>,
) -> UsersResult<impl IntoResponse> {
    info!(
        email = %req_body.email,
        display_name = %req_body.display_name,
        creator_id = %ctx.subject_id(),
        "Creating new user"
    );

    let new_user = req_body.into();
    let user = svc
        .create_user(&ctx, new_user)
        .await
        .map_err(UsersApiError::from_domain)?;
    Ok(created_json(UserDto::from(user)))
}

/// Update an existing user
#[tracing::instrument(
    name = "users_info.update_user",
    skip(svc, req_body, ctx),
    fields(
        user.id = %id,
        request_id = Empty,
        updater.id = %ctx.subject_id()
    )
)]
pub async fn update_user(
    Authz(ctx): Authz,                         // ← Validated SecurityCtx
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path(id): Path<Uuid>,
    Json(req_body): Json<UpdateUserReq>,
) -> UsersResult<JsonBody<UserDto>> {
    info!(
        user_id = %id,
        updater_id = %ctx.subject_id(),
        "Updating user"
    );

    let patch = req_body.into();
    let user = svc
        .update_user(&ctx, id, patch)
        .await
        .map_err(UsersApiError::from_domain)?;
    Ok(Json(UserDto::from(user)))
}

/// Delete a user by ID
#[tracing::instrument(
    name = "users_info.delete_user",
    skip(svc, ctx),
    fields(
        user.id = %id,
        request_id = Empty,
        deleter.id = %ctx.subject_id()
    )
)]
pub async fn delete_user(
    Authz(ctx): Authz,                         // ← Validated SecurityCtx
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path(id): Path<Uuid>,
) -> UsersResult<impl IntoResponse> {
    info!(
        user_id = %id,
        deleter_id = %ctx.subject_id(),
        "Deleting user"
    );

    svc.delete_user(&ctx, id)
        .await
        .map_err(UsersApiError::from_domain)?;
    Ok(no_content())
}

/// SSE endpoint returning a live stream of `UserEvent`.
#[tracing::instrument(
    name = "users_info.users_events",
    skip(sse),
    fields(request_id = Empty)
)]
pub async fn users_events(
    Extension(sse): Extension<SseBroadcaster<UserEvent>>,
) -> impl IntoResponse {
    info!("New SSE connection for user events");
    sse.sse_response_named("users_events")
}
