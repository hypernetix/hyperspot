use axum::{extract::Path, http::Uri, Extension};
use tracing::{field::Empty, info};
use uuid::Uuid;

use crate::api::rest::dto::{CreateUserReq, UpdateUserReq, UserDto, UserEvent};

use modkit::api::odata::OData;
use modkit::api::prelude::*;
use modkit::api::select::{apply_select, page_to_projected_json};

use crate::domain::service::Service;
use modkit::SseBroadcaster;

use modkit_security::SecurityContext;

/// List users with cursor-based pagination and optional field projection via $select
#[tracing::instrument(
    skip(svc, query, ctx),
    fields(
        limit = query.limit,
        request_id = Empty,
        user.id = %ctx.subject_id()
    )
)]
pub async fn list_users(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    OData(query): OData,
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

/// Get a specific user by ID with optional field projection via $select
#[tracing::instrument(
    skip(svc, ctx),
    fields(
        user.id = %id,
        request_id = Empty,
        requester.id = %ctx.subject_id()
    )
)]
pub async fn get_user(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path(id): Path<Uuid>,
    OData(query): OData,
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

/// Create a new user
#[tracing::instrument(
    skip(svc, req_body, ctx, uri),
    fields(
        user.email = %req_body.email,
        user.display_name = %req_body.display_name,
        user.tenant_id = %req_body.tenant_id,
        request_id = Empty,
        creator.id = %ctx.subject_id()
    )
)]
pub async fn create_user(
    uri: Uri,
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Json(req_body): Json<CreateUserReq>,
) -> ApiResult<impl IntoResponse> {
    info!(
        email = %req_body.email,
        display_name = %req_body.display_name,
        tenant_id = %req_body.tenant_id,
        creator_id = %ctx.subject_id(),
        "Creating new user"
    );

    let CreateUserReq {
        id,
        tenant_id,
        email,
        display_name,
    } = req_body;

    // Authorization check:
    // - root scope: allow any tenant_id
    // - non-root: tenant_id must be present in scope.tenant_ids()
    // TODO: the check must be done in the service layer
    // let scope = ctx.scope();
    // if !scope.is_root() {
    //     let allowed = scope.tenant_ids().iter().any(|t| t == &tenant_id);
    //     if !allowed {
    //         return Err(DomainError::validation(
    //             "tenant_id",
    //             format!("Tenant {tenant_id} is not allowed in current security scope"),
    //         )
    //         .into());
    //     }
    // }

    let new_user = user_info_sdk::NewUser {
        id,
        tenant_id,
        email,
        display_name,
    };

    let user = svc.create_user(&ctx, new_user).await?;
    let id_str = user.id.to_string();
    Ok(created_json(UserDto::from(user), &uri, &id_str))
}

/// Update an existing user
#[tracing::instrument(
    skip(svc, req_body, ctx),
    fields(
        user.id = %id,
        request_id = Empty,
        updater.id = %ctx.subject_id()
    )
)]
pub async fn update_user(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path(id): Path<Uuid>,
    Json(req_body): Json<UpdateUserReq>,
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

/// Delete a user by ID
#[tracing::instrument(
    skip(svc, ctx),
    fields(
        user.id = %id,
        request_id = Empty,
        deleter.id = %ctx.subject_id()
    )
)]
pub async fn delete_user(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path(id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    info!(
        user_id = %id,
        deleter_id = %ctx.subject_id(),
        "Deleting user"
    );

    svc.delete_user(&ctx, id).await?;
    Ok(no_content())
}

/// SSE endpoint returning a live stream of `UserEvent`.
#[tracing::instrument(
    skip(sse),
    fields(request_id = Empty)
)]
pub async fn users_events(
    Extension(sse): Extension<SseBroadcaster<UserEvent>>,
) -> impl IntoResponse {
    info!("New SSE connection for user events");
    sse.sse_response_named("users_events")
}
