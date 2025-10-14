use axum::{extract::Path, Extension};
use tracing::{field::Empty, info};
use uuid::Uuid;

use crate::api::rest::dto::{CreateUserReq, UpdateUserReq, UserDto, UserEvent};

use modkit::api::odata::OData;
use modkit::api::prelude::*;

use crate::domain::service::Service;
use modkit::SseBroadcaster;

// Import SecurityCtx from modkit_db
use modkit_db::secure::SecurityCtx;

// Type aliases for our specific API with DomainError
use crate::domain::error::DomainError;
type UsersResult<T> = ApiResult<T, DomainError>;
type UsersApiError = ApiError<DomainError>;

/// Create a fake security context for demonstration purposes.
///
/// In a real application, this would be extracted from:
/// - JWT claims (tenant_id, user_id)
/// - Session cookies
/// - API key headers
/// - OAuth tokens
///
/// For now, we simulate a single-tenant context with a fake subject ID.
///
/// # TODO
/// - Integrate with actual auth middleware
/// - Extract tenant from JWT claims
/// - Handle multi-tenant scenarios
/// - Add role-based access control
fn fake_ctx_from_request() -> SecurityCtx {
    // In production: extract from JWT, session, or auth middleware
    // For now: simulate access for a default tenant
    let fake_tenant_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .expect("valid UUID for fake tenant");
    let fake_user_id =
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").expect("valid UUID for fake user");

    SecurityCtx::for_tenant(fake_tenant_id, fake_user_id)
}

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

    let ctx = fake_ctx_from_request();
    let page = svc
        .list_users_page(&ctx, query)
        .await?
        .map_items(UserDto::from);
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

    let ctx = fake_ctx_from_request();
    let user = svc
        .get_user(&ctx, id)
        .await
        .map_err(UsersApiError::from_domain)?;
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

    let ctx = fake_ctx_from_request();
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

    let ctx = fake_ctx_from_request();
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

    let ctx = fake_ctx_from_request();
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
