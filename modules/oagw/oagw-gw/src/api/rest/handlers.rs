//! REST handlers for OAGW.
//!
//! Handlers are thin: parse/validate input, call domain service, map errors to Problem.

use std::sync::Arc;

use axum::extract::{Extension, Path};
use axum::http::Uri;
use axum::Json;
use modkit::api::odata::OData;
use modkit::api::prelude::*;
use modkit_security::SecurityContext;
use uuid::Uuid;

use super::dto::{
    CreateLinkRequest, CreateRouteRequest, HealthResponse, InvokeRequest, InvokeResponse, LinkDto,
    RouteDto, UpdateLinkRequest, UpdateRouteRequest,
};
use crate::domain::service::Service;

// === Health Endpoints ===

/// GET /oagw/v1/health - Liveness probe (no auth required).
pub async fn health() -> ApiResult<JsonBody<HealthResponse>> {
    Ok(Json(HealthResponse::default()))
}

/// GET /oagw/v1/ready - Readiness probe (no auth required).
///
/// TODO(v2): Check database connectivity and other critical dependencies.
pub async fn ready() -> ApiResult<JsonBody<HealthResponse>> {
    // TODO(v2): Implement actual readiness checks
    Ok(Json(HealthResponse::default()))
}

// === Route Endpoints ===

/// POST /oagw/v1/routes - Create a new route.
#[tracing::instrument(skip(ctx, svc, req), fields(base_url = %req.base_url))]
pub async fn create_route(
    uri: Uri,
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<Service>>,
    Json(req): Json<CreateRouteRequest>,
) -> ApiResult<impl axum::response::IntoResponse> {
    let tenant_id = ctx.tenant_id();
    let new_route = req.into_new_route(tenant_id);

    let route = svc.create_route(&ctx, new_route).await?;
    let id_str = route.id.to_string();

    Ok(created_json(RouteDto::from(route), &uri, &id_str))
}

/// GET /oagw/v1/routes - List routes with OData pagination.
#[tracing::instrument(skip(ctx, svc))]
pub async fn list_routes(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<Service>>,
    OData(query): OData,
) -> ApiResult<JsonPage<RouteDto>> {
    let page = svc.list_routes(&ctx, query).await?;
    let dto_page = page.map_items(RouteDto::from);
    Ok(Json(dto_page))
}

/// GET /oagw/v1/routes/{routeId} - Get a route by ID.
#[tracing::instrument(skip(ctx, svc), fields(route_id = %route_id))]
pub async fn get_route(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<Service>>,
    Path(route_id): Path<Uuid>,
) -> ApiResult<JsonBody<RouteDto>> {
    let route = svc.get_route(&ctx, route_id).await?;
    Ok(Json(RouteDto::from(route)))
}

/// PATCH /oagw/v1/routes/{routeId} - Update a route.
#[tracing::instrument(skip(ctx, svc, req), fields(route_id = %route_id))]
pub async fn update_route(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<Service>>,
    Path(route_id): Path<Uuid>,
    Json(req): Json<UpdateRouteRequest>,
) -> ApiResult<JsonBody<RouteDto>> {
    let patch = req.into();
    let route = svc.update_route(&ctx, route_id, patch).await?;
    Ok(Json(RouteDto::from(route)))
}

/// DELETE /oagw/v1/routes/{routeId} - Delete a route.
#[tracing::instrument(skip(ctx, svc), fields(route_id = %route_id))]
pub async fn delete_route(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<Service>>,
    Path(route_id): Path<Uuid>,
) -> ApiResult<impl axum::response::IntoResponse> {
    svc.delete_route(&ctx, route_id).await?;
    Ok(no_content())
}

// === Link Endpoints ===

/// POST /oagw/v1/links - Create a new link.
#[tracing::instrument(skip(ctx, svc, req), fields(route_id = %req.route_id))]
pub async fn create_link(
    uri: Uri,
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<Service>>,
    Json(req): Json<CreateLinkRequest>,
) -> ApiResult<impl axum::response::IntoResponse> {
    let tenant_id = ctx.tenant_id();
    let new_link = req.into_new_link(tenant_id);

    let link = svc.create_link(&ctx, new_link).await?;
    let id_str = link.id.to_string();

    Ok(created_json(LinkDto::from(link), &uri, &id_str))
}

/// GET /oagw/v1/links - List links with OData pagination.
#[tracing::instrument(skip(ctx, svc))]
pub async fn list_links(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<Service>>,
    OData(query): OData,
) -> ApiResult<JsonPage<LinkDto>> {
    let page = svc.list_links(&ctx, query).await?;
    let dto_page = page.map_items(LinkDto::from);
    Ok(Json(dto_page))
}

/// GET /oagw/v1/links/{linkId} - Get a link by ID.
#[tracing::instrument(skip(ctx, svc), fields(link_id = %link_id))]
pub async fn get_link(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<Service>>,
    Path(link_id): Path<Uuid>,
) -> ApiResult<JsonBody<LinkDto>> {
    let link = svc.get_link(&ctx, link_id).await?;
    Ok(Json(LinkDto::from(link)))
}

/// PATCH /oagw/v1/links/{linkId} - Update a link.
#[tracing::instrument(skip(ctx, svc, req), fields(link_id = %link_id))]
pub async fn update_link(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<Service>>,
    Path(link_id): Path<Uuid>,
    Json(req): Json<UpdateLinkRequest>,
) -> ApiResult<JsonBody<LinkDto>> {
    let patch = req.into();
    let link = svc.update_link(&ctx, link_id, patch).await?;
    Ok(Json(LinkDto::from(link)))
}

/// DELETE /oagw/v1/links/{linkId} - Delete a link.
#[tracing::instrument(skip(ctx, svc), fields(link_id = %link_id))]
pub async fn delete_link(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<Service>>,
    Path(link_id): Path<Uuid>,
) -> ApiResult<impl axum::response::IntoResponse> {
    svc.delete_link(&ctx, link_id).await?;
    Ok(no_content())
}

// === Invocation Endpoints ===

/// POST /oagw/v1/invoke - Invoke an outbound API.
#[tracing::instrument(skip(ctx, svc, req), fields(route_id = %req.route_id, method = ?req.method))]
pub async fn invoke(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<Service>>,
    Json(req): Json<InvokeRequest>,
) -> ApiResult<JsonBody<InvokeResponse>> {
    // TODO(v2): Validate caller has invoke role
    // TODO(v2): Validate request against OpenAPI schema

    let invoke_req = req.into();
    let response = svc.invoke_unary(&ctx, invoke_req).await?;

    Ok(Json(InvokeResponse::from(response)))
}

// === Route Health Endpoint (v3) ===

/// GET /oagw/v1/routes/{routeId}/health - Route health check.
///
/// TODO(v3): Implement route health check with circuit breaker status.
#[tracing::instrument(skip(ctx, svc), fields(route_id = %route_id))]
pub async fn route_health(
    Extension(ctx): Extension<SecurityContext>,
    Extension(svc): Extension<Arc<Service>>,
    Path(route_id): Path<Uuid>,
) -> ApiResult<JsonBody<HealthResponse>> {
    // TODO(v3): Check circuit breaker state for all links
    // TODO(v3): Require route health role

    // For v1, just verify route exists
    let _route = svc.get_route(&ctx, route_id).await?;

    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
    }))
}

// === Token Cache Management (v3) ===

/// DELETE /oagw/v1/routes/{routeId}/cache/tokens - Clear cached tokens for route.
///
/// TODO(v3): Implement token cache clearing.
#[tracing::instrument(skip(_ctx, _svc), fields(route_id = %route_id))]
pub async fn clear_token_cache(
    Extension(_ctx): Extension<SecurityContext>,
    Extension(_svc): Extension<Arc<Service>>,
    Path(route_id): Path<Uuid>,
) -> ApiResult<impl axum::response::IntoResponse> {
    // TODO(v3): Clear token cache entries for route_id
    tracing::info!(route_id = %route_id, "Token cache clear requested (not yet implemented)");

    Ok(no_content())
}
