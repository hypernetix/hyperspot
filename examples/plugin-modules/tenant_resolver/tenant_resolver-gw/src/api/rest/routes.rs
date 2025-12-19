//! REST API routes for tenant resolver gateway.

use axum::http;
use axum::{Extension, Router};
use modkit::api::operation_builder::OperationBuilderODataExt;
use modkit::api::{OpenApiRegistry, OperationBuilder};
use std::sync::Arc;

use crate::api::rest::dto::{GetChildrenResponseDto, GetParentsResponseDto, TenantDto};
use crate::api::rest::handlers;
use crate::domain::service::Service;

pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<Service>,
) -> Router {
    // GET /tenant-resolver/v1/root
    router = OperationBuilder::get("/tenant-resolver/v1/root")
        .operation_id("tenant_resolver_gateway.get_root_tenant")
        .summary("Get root tenant")
        .description("Returns the root tenant as resolved by the active plugin.")
        .tag("tenant_resolver_gateway")
        .public()
        .handler(handlers::get_root_tenant)
        .json_response_with_schema::<TenantDto>(openapi, http::StatusCode::OK, "Root tenant")
        .standard_errors(openapi)
        .register(router, openapi);

    // GET /tenant-resolver/v1/tenants
    router = OperationBuilder::get("/tenant-resolver/v1/tenants")
        .operation_id("tenant_resolver_gateway.list_tenants")
        .summary("List tenants")
        .description("Returns a paginated list of tenants with optional filtering by status.")
        .tag("tenant_resolver_gateway")
        .public()
        .query_param(
            "statuses",
            false,
            "Filter by statuses (comma-separated: ACTIVE,SOFT_DELETED)",
        )
        .query_param_typed(
            "limit",
            false,
            "Maximum number of tenants to return",
            "integer",
        )
        .query_param("cursor", false, "Cursor for pagination")
        .handler(handlers::list_tenants)
        .json_response_with_schema::<modkit_odata::Page<TenantDto>>(
            openapi,
            http::StatusCode::OK,
            "Paginated list of tenants",
        )
        .with_odata_select()
        .standard_errors(openapi)
        .register(router, openapi);

    // GET /tenant-resolver/v1/tenants/{id}/parents
    router = OperationBuilder::get("/tenant-resolver/v1/tenants/{id}/parents")
        .operation_id("tenant_resolver_gateway.get_parents")
        .summary("Get tenant parents")
        .description(
            "Returns all parents (direct and indirect) of the given tenant. \
             Parents are ordered from direct parent to root.",
        )
        .tag("tenant_resolver_gateway")
        .public()
        .path_param("id", "Tenant ID")
        .query_param(
            "statuses",
            false,
            "Filter by statuses (comma-separated: ACTIVE,SOFT_DELETED)",
        )
        .query_param(
            "ignore_access",
            false,
            "Ignore parent access constraints (true/false)",
        )
        .handler(handlers::get_parents)
        .json_response_with_schema::<GetParentsResponseDto>(
            openapi,
            http::StatusCode::OK,
            "Tenant with parent chain",
        )
        .standard_errors(openapi)
        .register(router, openapi);

    // GET /tenant-resolver/v1/tenants/{id}/children
    router = OperationBuilder::get("/tenant-resolver/v1/tenants/{id}/children")
        .operation_id("tenant_resolver_gateway.get_children")
        .summary("Get tenant children")
        .description(
            "Returns all children (direct and indirect) of the given tenant. \
             Children are returned in pre-order traversal (parent before subtree). \
             Use max_depth to limit: 0 = unlimited, 1 = direct children only.",
        )
        .tag("tenant_resolver_gateway")
        .public()
        .path_param("id", "Tenant ID")
        .query_param(
            "statuses",
            false,
            "Filter by statuses (comma-separated: ACTIVE,SOFT_DELETED)",
        )
        .query_param(
            "ignore_access",
            false,
            "Ignore parent access constraints (true/false)",
        )
        .query_param("max_depth", false, "Max depth (0=unlimited, 1=direct only)")
        .handler(handlers::get_children)
        .json_response_with_schema::<GetChildrenResponseDto>(
            openapi,
            http::StatusCode::OK,
            "List of child tenants",
        )
        .standard_errors(openapi)
        .register(router, openapi);

    router.layer(Extension(service))
}
