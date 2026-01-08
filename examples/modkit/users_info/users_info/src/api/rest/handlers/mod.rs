use axum::{extract::Path, http::Uri, Extension};
use tracing::{field::Empty, info};
use uuid::Uuid;

use crate::api::rest::dto::{
    AddressDto, CityDto, CreateCityReq, CreateLanguageReq, CreateUserReq, LanguageDto,
    PutAddressReq, UpdateCityReq, UpdateLanguageReq, UpdateUserReq, UserDto, UserEvent,
};

use modkit::api::odata::OData;
use modkit::api::prelude::*;
use modkit::api::select::{apply_select, page_to_projected_json};

use crate::domain::service::Service;
use modkit::SseBroadcaster;

use modkit_security::SecurityContext;

mod addresses;
mod cities;
mod events;
mod languages;
mod relations;
mod users;

// ==================== User Handlers ====================

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
    users::list_users(ctx, svc, query).await
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
    users::get_user(ctx, svc, id, query).await
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
    // TODO(phase-1): Move tenant_id authorization check to service layer for proper separation of concerns
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

    users::create_user(uri, ctx, svc, new_user).await
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
    users::update_user(ctx, svc, id, req_body).await
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
    users::delete_user(ctx, svc, id).await
}

// ==================== Event Handlers (SSE) ====================

/// SSE endpoint returning a live stream of `UserEvent`.
#[tracing::instrument(
    skip(sse),
    fields(request_id = Empty)
)]
pub async fn users_events(
    Extension(sse): Extension<SseBroadcaster<UserEvent>>,
) -> impl IntoResponse {
    events::users_events(&sse)
}

// ==================== City Handlers ====================

/// List cities with cursor-based pagination and optional field projection via $select
#[tracing::instrument(
    skip(svc, query, ctx),
    fields(
        limit = query.limit,
        request_id = Empty,
        user.id = %ctx.subject_id()
    )
)]
pub async fn list_cities(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    OData(query): OData,
) -> ApiResult<JsonPage<serde_json::Value>> {
    cities::list_cities(ctx, svc, query).await
}

/// Get a specific city by ID with optional field projection via $select
#[tracing::instrument(
    skip(svc, ctx),
    fields(
        city.id = %id,
        request_id = Empty,
        requester.id = %ctx.subject_id()
    )
)]
pub async fn get_city(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path(id): Path<Uuid>,
    OData(query): OData,
) -> ApiResult<JsonBody<serde_json::Value>> {
    cities::get_city(ctx, svc, id, query).await
}

/// Create a new city
#[tracing::instrument(
    skip(svc, req_body, ctx, uri),
    fields(
        city.name = %req_body.name,
        city.country = %req_body.country,
        city.tenant_id = %req_body.tenant_id,
        request_id = Empty,
        creator.id = %ctx.subject_id()
    )
)]
pub async fn create_city(
    uri: Uri,
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Json(req_body): Json<CreateCityReq>,
) -> ApiResult<impl IntoResponse> {
    cities::create_city(uri, ctx, svc, req_body).await
}

/// Update an existing city
#[tracing::instrument(
    skip(svc, req_body, ctx),
    fields(
        city.id = %id,
        request_id = Empty,
        updater.id = %ctx.subject_id()
    )
)]
pub async fn update_city(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path(id): Path<Uuid>,
    Json(req_body): Json<UpdateCityReq>,
) -> ApiResult<JsonBody<CityDto>> {
    cities::update_city(ctx, svc, id, req_body).await
}

/// Delete a city by ID
#[tracing::instrument(
    skip(svc, ctx),
    fields(
        city.id = %id,
        request_id = Empty,
        deleter.id = %ctx.subject_id()
    )
)]
pub async fn delete_city(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path(id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    cities::delete_city(ctx, svc, id).await
}

// ==================== Language Handlers ====================

/// List languages with cursor-based pagination and optional field projection via $select
#[tracing::instrument(
    skip(svc, query, ctx),
    fields(
        limit = query.limit,
        request_id = Empty,
        user.id = %ctx.subject_id()
    )
)]
pub async fn list_languages(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    OData(query): OData,
) -> ApiResult<JsonPage<serde_json::Value>> {
    languages::list_languages(ctx, svc, query).await
}

/// Get a specific language by ID with optional field projection via $select
#[tracing::instrument(
    skip(svc, ctx),
    fields(
        language.id = %id,
        request_id = Empty,
        requester.id = %ctx.subject_id()
    )
)]
pub async fn get_language(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path(id): Path<Uuid>,
    OData(query): OData,
) -> ApiResult<JsonBody<serde_json::Value>> {
    languages::get_language(ctx, svc, id, query).await
}

/// Create a new language
#[tracing::instrument(
    skip(svc, req_body, ctx, uri),
    fields(
        language.code = %req_body.code,
        language.name = %req_body.name,
        language.tenant_id = %req_body.tenant_id,
        request_id = Empty,
        creator.id = %ctx.subject_id()
    )
)]
pub async fn create_language(
    uri: Uri,
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Json(req_body): Json<CreateLanguageReq>,
) -> ApiResult<impl IntoResponse> {
    languages::create_language(uri, ctx, svc, req_body).await
}

/// Update an existing language
#[tracing::instrument(
    skip(svc, req_body, ctx),
    fields(
        language.id = %id,
        request_id = Empty,
        updater.id = %ctx.subject_id()
    )
)]
pub async fn update_language(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path(id): Path<Uuid>,
    Json(req_body): Json<UpdateLanguageReq>,
) -> ApiResult<JsonBody<LanguageDto>> {
    languages::update_language(ctx, svc, id, req_body).await
}

/// Delete a language by ID
#[tracing::instrument(
    skip(svc, ctx),
    fields(
        language.id = %id,
        request_id = Empty,
        deleter.id = %ctx.subject_id()
    )
)]
pub async fn delete_language(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path(id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    languages::delete_language(ctx, svc, id).await
}

// ==================== Address Handlers ====================

/// Get address for a specific user
#[tracing::instrument(
    skip(svc, ctx),
    fields(
        user.id = %user_id,
        request_id = Empty,
        requester.id = %ctx.subject_id()
    )
)]
pub async fn get_user_address(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path(user_id): Path<Uuid>,
) -> ApiResult<JsonBody<AddressDto>> {
    addresses::get_user_address(ctx, svc, user_id).await
}

/// Upsert address for a specific user (PUT = create or replace)
#[tracing::instrument(
    skip(svc, req_body, ctx),
    fields(
        user.id = %user_id,
        request_id = Empty,
        updater.id = %ctx.subject_id()
    )
)]
pub async fn put_user_address(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path(user_id): Path<Uuid>,
    Json(req_body): Json<PutAddressReq>,
) -> ApiResult<impl IntoResponse> {
    addresses::put_user_address(ctx, svc, user_id, req_body).await
}

/// Delete address for a specific user
#[tracing::instrument(
    skip(svc, ctx),
    fields(
        user.id = %user_id,
        request_id = Empty,
        deleter.id = %ctx.subject_id()
    )
)]
pub async fn delete_user_address(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path(user_id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    addresses::delete_user_address(ctx, svc, user_id).await
}

// ==================== User-Language Relationship Handlers ====================

/// List all languages assigned to a user
#[tracing::instrument(
    skip(svc, ctx),
    fields(
        user.id = %user_id,
        request_id = Empty,
        requester.id = %ctx.subject_id()
    )
)]
pub async fn list_user_languages(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path(user_id): Path<Uuid>,
) -> ApiResult<JsonBody<Vec<LanguageDto>>> {
    relations::list_user_languages(ctx, svc, user_id).await
}

/// Assign a language to a user (idempotent)
#[tracing::instrument(
    skip(svc, ctx),
    fields(
        user.id = %user_id,
        language.id = %language_id,
        request_id = Empty,
        updater.id = %ctx.subject_id()
    )
)]
pub async fn assign_language_to_user(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path((user_id, language_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<impl IntoResponse> {
    relations::assign_language_to_user(ctx, svc, user_id, language_id).await
}

/// Remove a language from a user (idempotent)
#[tracing::instrument(
    skip(svc, ctx),
    fields(
        user.id = %user_id,
        language.id = %language_id,
        request_id = Empty,
        deleter.id = %ctx.subject_id()
    )
)]
pub async fn remove_language_from_user(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<std::sync::Arc<Service>>,
    Path((user_id, language_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<impl IntoResponse> {
    relations::remove_language_from_user(ctx, svc, user_id, language_id).await
}
