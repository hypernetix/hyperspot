//! REST route registration for OAGW.

use std::sync::Arc;

use axum::{Extension, Router};
use http::StatusCode;
use modkit::api::operation_builder::{
    AuthReqAction, AuthReqResource, LicenseFeature, OperationBuilderODataExt,
};
use modkit::api::{OpenApiRegistry, OperationBuilder};
use modkit_odata::Page;

use super::dto::{
    CreateLinkRequest, CreateRouteRequest, HealthResponse, InvokeRequest, InvokeResponse, LinkDto,
    LinkDtoFilterField, RouteDto, RouteDtoFilterField, UpdateLinkRequest, UpdateRouteRequest,
};
use super::handlers;
use crate::domain::service::Service;

/// License feature for OAGW.
struct OagwLicense;

impl AsRef<str> for OagwLicense {
    fn as_ref(&self) -> &'static str {
        "gts.x.core.lic.feat.v1~x.core.global.base.v1"
    }
}

impl LicenseFeature for OagwLicense {}

/// Resource enum for authorization.
#[derive(Debug, Clone, Copy)]
pub enum Resource {
    Routes,
    Links,
    Invoke,
    Health,
}

impl AsRef<str> for Resource {
    fn as_ref(&self) -> &'static str {
        match self {
            Self::Routes => "oagw.routes",
            Self::Links => "oagw.links",
            Self::Invoke => "oagw.invoke",
            Self::Health => "oagw.health",
        }
    }
}

impl AuthReqResource for Resource {}

/// Action enum for authorization.
#[derive(Debug, Clone, Copy)]
pub enum Action {
    Create,
    Read,
    Update,
    Delete,
    Execute,
}

impl AsRef<str> for Action {
    fn as_ref(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Read => "read",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::Execute => "execute",
        }
    }
}

impl AuthReqAction for Action {}

/// Register all OAGW REST routes.
#[allow(clippy::too_many_lines)] // Route registration is naturally verbose
#[allow(clippy::unnecessary_wraps)] // Result needed for error propagation in future
pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<Service>,
) -> anyhow::Result<Router> {
    // === Health Endpoints (public, no auth) ===

    // GET /oagw/v1/health - Liveness probe
    router = OperationBuilder::get("/oagw/v1/health")
        .operation_id("oagw.health")
        .summary("Liveness probe")
        .description("Returns healthy if the service is running")
        .tag("oagw-health")
        .public()
        .handler(handlers::health)
        .json_response_with_schema::<HealthResponse>(openapi, StatusCode::OK, "Service is healthy")
        .register(router, openapi);

    // GET /oagw/v1/ready - Readiness probe
    router = OperationBuilder::get("/oagw/v1/ready")
        .operation_id("oagw.ready")
        .summary("Readiness probe")
        .description("Returns healthy if the service is ready to accept requests")
        .tag("oagw-health")
        .public()
        .handler(handlers::ready)
        .json_response_with_schema::<HealthResponse>(openapi, StatusCode::OK, "Service is ready")
        .problem_response(
            openapi,
            StatusCode::SERVICE_UNAVAILABLE,
            "Service not ready",
        )
        .register(router, openapi);

    // === Route Endpoints ===

    // POST /oagw/v1/routes - Create route
    router = OperationBuilder::post("/oagw/v1/routes")
        .operation_id("oagw.create_route")
        .summary("Create a new outbound API route")
        .tag("oagw-routes")
        .require_auth(&Resource::Routes, &Action::Create)
        .require_license_features::<OagwLicense>([])
        .json_request::<CreateRouteRequest>(openapi, "Route configuration")
        .handler(handlers::create_route)
        .json_response_with_schema::<RouteDto>(openapi, StatusCode::CREATED, "Route created")
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_409(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /oagw/v1/routes - List routes
    router = OperationBuilder::get("/oagw/v1/routes")
        .operation_id("oagw.list_routes")
        .summary("List outbound API routes")
        .description("Returns a paginated list of routes with OData query support")
        .tag("oagw-routes")
        .require_auth(&Resource::Routes, &Action::Read)
        .require_license_features::<OagwLicense>([])
        .with_odata_filter::<RouteDtoFilterField>()
        .with_odata_select()
        .with_odata_orderby::<RouteDtoFilterField>()
        .handler(handlers::list_routes)
        .json_response_with_schema::<Page<RouteDto>>(
            openapi,
            StatusCode::OK,
            "Paginated list of routes",
        )
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /oagw/v1/routes/{routeId} - Get route
    router = OperationBuilder::get("/oagw/v1/routes/{routeId}")
        .operation_id("oagw.get_route")
        .summary("Get a route by ID")
        .tag("oagw-routes")
        .require_auth(&Resource::Routes, &Action::Read)
        .require_license_features::<OagwLicense>([])
        .path_param("routeId", "Route UUID")
        .handler(handlers::get_route)
        .json_response_with_schema::<RouteDto>(openapi, StatusCode::OK, "Route found")
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // PATCH /oagw/v1/routes/{routeId} - Update route
    router = OperationBuilder::patch("/oagw/v1/routes/{routeId}")
        .operation_id("oagw.update_route")
        .summary("Update a route")
        .tag("oagw-routes")
        .require_auth(&Resource::Routes, &Action::Update)
        .require_license_features::<OagwLicense>([])
        .path_param("routeId", "Route UUID")
        .json_request::<UpdateRouteRequest>(openapi, "Route update data")
        .handler(handlers::update_route)
        .json_response_with_schema::<RouteDto>(openapi, StatusCode::OK, "Route updated")
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // DELETE /oagw/v1/routes/{routeId} - Delete route
    router = OperationBuilder::delete("/oagw/v1/routes/{routeId}")
        .operation_id("oagw.delete_route")
        .summary("Delete a route")
        .tag("oagw-routes")
        .require_auth(&Resource::Routes, &Action::Delete)
        .require_license_features::<OagwLicense>([])
        .path_param("routeId", "Route UUID")
        .handler(handlers::delete_route)
        .json_response(StatusCode::NO_CONTENT, "Route deleted")
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /oagw/v1/routes/{routeId}/health - Route health (v3)
    router = OperationBuilder::get("/oagw/v1/routes/{routeId}/health")
        .operation_id("oagw.route_health")
        .summary("Check route health")
        .description("Returns health status of a route including circuit breaker states")
        .tag("oagw-health")
        .require_auth(&Resource::Health, &Action::Read)
        .require_license_features::<OagwLicense>([])
        .path_param("routeId", "Route UUID")
        .handler(handlers::route_health)
        .json_response_with_schema::<HealthResponse>(openapi, StatusCode::OK, "Route health status")
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // DELETE /oagw/v1/routes/{routeId}/cache/tokens - Clear token cache (v3)
    router = OperationBuilder::delete("/oagw/v1/routes/{routeId}/cache/tokens")
        .operation_id("oagw.clear_token_cache")
        .summary("Clear cached tokens for a route")
        .description("Evicts all cached OAuth tokens associated with this route")
        .tag("oagw-routes")
        .require_auth(&Resource::Routes, &Action::Update)
        .require_license_features::<OagwLicense>([])
        .path_param("routeId", "Route UUID")
        .handler(handlers::clear_token_cache)
        .json_response(StatusCode::NO_CONTENT, "Token cache cleared")
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // === Link Endpoints ===

    // POST /oagw/v1/links - Create link
    router = OperationBuilder::post("/oagw/v1/links")
        .operation_id("oagw.create_link")
        .summary("Create a new outbound API link")
        .tag("oagw-links")
        .require_auth(&Resource::Links, &Action::Create)
        .require_license_features::<OagwLicense>([])
        .json_request::<CreateLinkRequest>(openapi, "Link configuration")
        .handler(handlers::create_link)
        .json_response_with_schema::<LinkDto>(openapi, StatusCode::CREATED, "Link created")
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi) // Route not found
        .error_409(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /oagw/v1/links - List links
    router = OperationBuilder::get("/oagw/v1/links")
        .operation_id("oagw.list_links")
        .summary("List outbound API links")
        .description("Returns a paginated list of links with OData query support")
        .tag("oagw-links")
        .require_auth(&Resource::Links, &Action::Read)
        .require_license_features::<OagwLicense>([])
        .with_odata_filter::<LinkDtoFilterField>()
        .with_odata_select()
        .with_odata_orderby::<LinkDtoFilterField>()
        .handler(handlers::list_links)
        .json_response_with_schema::<Page<LinkDto>>(
            openapi,
            StatusCode::OK,
            "Paginated list of links",
        )
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /oagw/v1/links/{linkId} - Get link
    router = OperationBuilder::get("/oagw/v1/links/{linkId}")
        .operation_id("oagw.get_link")
        .summary("Get a link by ID")
        .tag("oagw-links")
        .require_auth(&Resource::Links, &Action::Read)
        .require_license_features::<OagwLicense>([])
        .path_param("linkId", "Link UUID")
        .handler(handlers::get_link)
        .json_response_with_schema::<LinkDto>(openapi, StatusCode::OK, "Link found")
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // PATCH /oagw/v1/links/{linkId} - Update link
    router = OperationBuilder::patch("/oagw/v1/links/{linkId}")
        .operation_id("oagw.update_link")
        .summary("Update a link")
        .tag("oagw-links")
        .require_auth(&Resource::Links, &Action::Update)
        .require_license_features::<OagwLicense>([])
        .path_param("linkId", "Link UUID")
        .json_request::<UpdateLinkRequest>(openapi, "Link update data")
        .handler(handlers::update_link)
        .json_response_with_schema::<LinkDto>(openapi, StatusCode::OK, "Link updated")
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // DELETE /oagw/v1/links/{linkId} - Delete link
    router = OperationBuilder::delete("/oagw/v1/links/{linkId}")
        .operation_id("oagw.delete_link")
        .summary("Delete a link")
        .tag("oagw-links")
        .require_auth(&Resource::Links, &Action::Delete)
        .require_license_features::<OagwLicense>([])
        .path_param("linkId", "Link UUID")
        .handler(handlers::delete_link)
        .json_response(StatusCode::NO_CONTENT, "Link deleted")
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // === Invocation Endpoint ===

    // POST /oagw/v1/invoke - Invoke outbound API
    router = OperationBuilder::post("/oagw/v1/invoke")
        .operation_id("oagw.invoke")
        .summary("Invoke an outbound API")
        .description("Execute an HTTP request to a configured downstream API")
        .tag("oagw-invoke")
        .require_auth(&Resource::Invoke, &Action::Execute)
        .require_license_features::<OagwLicense>([])
        .json_request::<InvokeRequest>(openapi, "Invocation request")
        .handler(handlers::invoke)
        .json_response_with_schema::<InvokeResponse>(openapi, StatusCode::OK, "Invocation response")
        .error_400(openapi)
        .error_401(openapi)
        .error_403(openapi)
        .error_404(openapi)
        .error_429(openapi) // Rate limit
        .error_500(openapi)
        .problem_response(openapi, StatusCode::BAD_GATEWAY, "Downstream error")
        .problem_response(
            openapi,
            StatusCode::SERVICE_UNAVAILABLE,
            "Service unavailable",
        )
        .problem_response(openapi, StatusCode::GATEWAY_TIMEOUT, "Gateway timeout")
        .register(router, openapi);

    // Attach service extension to all routes
    router = router.layer(Extension(service));

    Ok(router)
}
