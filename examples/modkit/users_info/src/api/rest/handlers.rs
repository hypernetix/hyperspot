use axum::{extract::Path, Extension};
use tracing::{field::Empty, info};
use uuid::Uuid;

use crate::api::rest::dto::{CreateUserReq, UpdateUserReq, UserDto, UserEvent};

use modkit::api::odata::OData;
use modkit::api::prelude::*;

use crate::domain::service::Service;
use modkit::SseBroadcaster;

// Type aliases for our specific API with DomainError
use crate::domain::error::DomainError;
type UsersResult<T> = ApiResult<T, DomainError>;
type UsersApiError = ApiError<DomainError>;

/// List users with cursor-based pagination
#[tracing::instrument(
    name = "users_info.list_users",
    skip(svc, query),
    fields(
        limit = query.limit,
        request_id = Empty
    )
)]
pub async fn list_users(
    Extension(svc): Extension<std::sync::Arc<Service>>,
    OData(query): OData,
) -> UsersResult<JsonPage<UserDto>> {
    info!("Listing users with cursor pagination");

    let page = svc.list_users_page(query).await?.map_items(UserDto::from);
    Ok(Json(page))
}

/// Get a specific user by ID
#[tracing::instrument(
    name = "users_info.get_user",
    skip(svc),
    fields(
        user.id = %id,
        request_id = Empty
    )
)]
pub async fn get_user(
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path(id): Path<Uuid>,
) -> UsersResult<JsonBody<UserDto>> {
    info!("Getting user with id: {}", id);

    let user = svc.get_user(id).await.map_err(UsersApiError::from_domain)?;
    Ok(Json(UserDto::from(user)))
}

/// Create a new user
#[tracing::instrument(
    name = "users_info.create_user",
    skip(svc, req_body),
    fields(
        user.email = %req_body.email,
        user.display_name = %req_body.display_name,
        request_id = Empty
    )
)]
pub async fn create_user(
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Json(req_body): Json<CreateUserReq>,
) -> UsersResult<impl IntoResponse> {
    info!("Creating user: {:?}", req_body);

    let new_user = req_body.into();
    let user = svc
        .create_user(new_user)
        .await
        .map_err(UsersApiError::from_domain)?;
    Ok(created_json(UserDto::from(user)))
}

/// Update an existing user
#[tracing::instrument(
    name = "users_info.update_user",
    skip(svc, req_body),
    fields(
        user.id = %id,
        request_id = Empty
    )
)]
pub async fn update_user(
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path(id): Path<Uuid>,
    Json(req_body): Json<UpdateUserReq>,
) -> UsersResult<JsonBody<UserDto>> {
    info!("Updating user {} with: {:?}", id, req_body);

    let patch = req_body.into();
    let user = svc
        .update_user(id, patch)
        .await
        .map_err(UsersApiError::from_domain)?;
    Ok(Json(UserDto::from(user)))
}

/// Delete a user by ID
#[tracing::instrument(
    name = "users_info.delete_user",
    skip(svc),
    fields(
        user.id = %id,
        request_id = Empty
    )
)]
pub async fn delete_user(
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path(id): Path<Uuid>,
) -> UsersResult<impl IntoResponse> {
    info!("Deleting user: {}", id);

    svc.delete_user(id)
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
