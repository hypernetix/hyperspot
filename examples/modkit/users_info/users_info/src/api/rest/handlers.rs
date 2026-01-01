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
    info!(
        user_id = %ctx.subject_id(),
        "Listing cities with cursor pagination"
    );

    let page = svc
        .list_cities_page(&ctx, &query)
        .await?
        .map_items(CityDto::from);

    Ok(Json(page_to_projected_json(&page, query.selected_fields())))
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
    info!(
        city_id = %id,
        requester_id = %ctx.subject_id(),
        "Getting city details"
    );

    let city = svc.get_city(&ctx, id).await?;
    let city_dto = CityDto::from(city);

    let projected = apply_select(&city_dto, query.selected_fields());

    Ok(Json(projected))
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
    info!(
        name = %req_body.name,
        country = %req_body.country,
        tenant_id = %req_body.tenant_id,
        creator_id = %ctx.subject_id(),
        "Creating new city"
    );

    let new_city = req_body.into();
    let city = svc.create_city(&ctx, new_city).await?;
    let id_str = city.id.to_string();
    Ok(created_json(CityDto::from(city), &uri, &id_str))
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
    info!(
        city_id = %id,
        updater_id = %ctx.subject_id(),
        "Updating city"
    );

    let patch = req_body.into();
    let city = svc.update_city(&ctx, id, patch).await?;
    Ok(Json(CityDto::from(city)))
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
    info!(
        city_id = %id,
        deleter_id = %ctx.subject_id(),
        "Deleting city"
    );

    svc.delete_city(&ctx, id).await?;
    Ok(no_content())
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
    Ok(created_json(LanguageDto::from(language), &uri, &id_str))
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
    info!(
        language_id = %id,
        updater_id = %ctx.subject_id(),
        "Updating language"
    );

    let patch = req_body.into();
    let language = svc.update_language(&ctx, id, patch).await?;
    Ok(Json(LanguageDto::from(language)))
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
    info!(
        language_id = %id,
        deleter_id = %ctx.subject_id(),
        "Deleting language"
    );

    svc.delete_language(&ctx, id).await?;
    Ok(no_content())
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
    info!(
        user_id = %user_id,
        requester_id = %ctx.subject_id(),
        "Getting user address"
    );

    let address = svc
        .get_user_address(&ctx, user_id)
        .await?
        .ok_or_else(|| crate::domain::error::DomainError::not_found("Address", user_id))?;

    Ok(Json(AddressDto::from(address)))
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
    info!(
        user_id = %user_id,
        updater_id = %ctx.subject_id(),
        "Upserting user address"
    );

    let new_address = req_body.into_new_address(user_id);
    let address = svc.put_user_address(&ctx, user_id, new_address).await?;

    Ok(Json(AddressDto::from(address)))
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
    info!(
        user_id = %user_id,
        deleter_id = %ctx.subject_id(),
        "Deleting user address"
    );

    svc.delete_user_address(&ctx, user_id).await?;
    Ok(no_content())
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
    info!(
        user_id = %user_id,
        requester_id = %ctx.subject_id(),
        "Listing user languages"
    );

    let languages = svc.list_user_languages(&ctx, user_id).await?;
    let dtos: Vec<LanguageDto> = languages.into_iter().map(LanguageDto::from).collect();

    Ok(Json(dtos))
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
    info!(
        user_id = %user_id,
        language_id = %language_id,
        updater_id = %ctx.subject_id(),
        "Assigning language to user"
    );

    svc.assign_language_to_user(&ctx, user_id, language_id)
        .await?;
    Ok(no_content())
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
    info!(
        user_id = %user_id,
        language_id = %language_id,
        deleter_id = %ctx.subject_id(),
        "Removing language from user"
    );

    svc.remove_language_from_user(&ctx, user_id, language_id)
        .await?;
    Ok(no_content())
}
