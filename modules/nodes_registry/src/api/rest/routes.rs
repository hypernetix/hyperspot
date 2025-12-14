use axum::http;
use axum::{Extension, Router};
use modkit::api::{Missing, OpenApiRegistry, OperationBuilder};
use std::sync::Arc;

use super::dto::{NodeDto, NodeSysCapDto, NodeSysInfoDto};
use super::handlers;
use crate::domain::service::Service;

/// Register all REST routes for the nodes registry module
pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    service: Arc<Service>,
) -> Router {
    // GET /nodes - List all nodes
    router = OperationBuilder::<Missing, Missing, ()>::get("/nodes")
        .operation_id("nodes_registry.list_nodes")
        .summary("List all nodes")
        .description("Get a list of all nodes in the deployment. Use ?details=true to include sysinfo and syscap. Use ?force_refresh=true to invalidate syscap cache.")
        .tag("nodes")
        .public()
        .query_param("details", false, "Include detailed system information and capabilities")
        .query_param("force_refresh", false, "Force refresh syscap, ignoring cache (only applies when details=true)")
        .handler(handlers::list_nodes)
        .json_response_with_schema::<Vec<NodeDto>>(openapi, http::StatusCode::OK, "List of nodes")
        .error_500(openapi)
        .register(router, openapi);

    // GET /nodes/{id} - Get a specific node
    router = OperationBuilder::<Missing, Missing, ()>::get("/nodes/{id}")
        .operation_id("nodes_registry.get_node")
        .summary("Get node by ID")
        .description("Get detailed information about a specific node. Use ?details=true to include sysinfo and syscap. Use ?force_refresh=true to invalidate syscap cache.")
        .tag("nodes")
        .public()
        .path_param("id", "Node UUID")
        .query_param("details", false, "Include detailed system information and capabilities")
        .query_param("force_refresh", false, "Force refresh syscap, ignoring cache (only applies when details=true)")
        .handler(handlers::get_node)
        .json_response_with_schema::<NodeDto>(openapi, http::StatusCode::OK, "Node details")
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /nodes/{id}/sysinfo - Get system information for a node
    router = OperationBuilder::<Missing, Missing, ()>::get("/nodes/{id}/sysinfo")
        .operation_id("nodes_registry.get_node_sysinfo")
        .summary("Get node system information")
        .description("Get detailed system information (OS, CPU, memory, etc.) for a specific node")
        .tag("nodes")
        .public()
        .path_param("id", "Node UUID")
        .handler(handlers::get_node_sysinfo)
        .json_response_with_schema::<NodeSysInfoDto>(
            openapi,
            http::StatusCode::OK,
            "System information",
        )
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // GET /nodes/{id}/syscap - Get system capabilities for a node
    router = OperationBuilder::<Missing, Missing, ()>::get("/nodes/{id}/syscap")
        .operation_id("nodes_registry.get_node_syscap")
        .summary("Get node system capabilities")
        .description("Get system capabilities (hardware, software features) for a specific node. Use ?force_refresh=true to invalidate cache and refresh all capabilities.")
        .tag("nodes")
        .public()
        .path_param("id", "Node UUID")
        .query_param("force_refresh", false, "Force refresh all syscap, ignoring cache and TTL")
        .handler(handlers::get_node_syscap)
        .json_response_with_schema::<NodeSysCapDto>(openapi, http::StatusCode::OK, "System capabilities (merged)")
        .error_404(openapi)
        .error_500(openapi)
        .register(router, openapi);

    // Attach service to router as extension
    router = router.layer(Extension(service));

    router
}
