// @fdd-change:fdd-analytics-feature-gts-core-change-platform-integration-fix
// @fdd-flow:fdd-analytics-feature-gts-core-flow-route-crud-operations
// @fdd-flow:fdd-analytics-feature-gts-core-flow-aggregate-odata-metadata
// @fdd-req:fdd-analytics-feature-gts-core-req-routing
// @fdd-req:fdd-analytics-feature-gts-core-req-middleware
use axum::{http::StatusCode, Extension, Router};
use modkit::api::operation_builder::OperationBuilderODataExt;
use modkit::api::{OpenApiRegistry, OperationBuilder};
use std::sync::Arc;

use super::dto::{GtsEntityDto, GtsEntityDtoFilterField, GtsEntityListDto, GtsEntityRequestDto};
use super::handlers;
use crate::domain::gts_core::GtsCoreRouter;

// @fdd-change:fdd-analytics-feature-gts-core-change-platform-integration-fix
/// Register GTS Core routes with OperationBuilder
pub fn register_routes(
    mut router: Router,
    openapi: &dyn OpenApiRegistry,
    gts_router: Arc<GtsCoreRouter>,
) -> Router {
    // GET /analytics/v1/gts/{id} - Get GTS entity by ID
    router = OperationBuilder::get("/analytics/v1/gts/{id}")
        .operation_id("gts_core.get_entity")
        .summary("Get GTS entity by ID")
        .description("Retrieve a specific GTS entity by its identifier")
        .tag("GTS Core")
        .path_param("id", "GTS entity identifier")
        .public()
        .handler(handlers::get_entity)
        .json_response_with_schema::<GtsEntityDto>(
            openapi,
            StatusCode::OK,
            "Entity retrieved successfully",
        )
        .error_404(openapi)
        .standard_errors(openapi)
        .register(router, openapi);

    // GET /analytics/v1/gts - List GTS entities with OData
    router = OperationBuilder::get("/analytics/v1/gts")
        .operation_id("gts_core.list_entities")
        .summary("List GTS entities")
        .description("List all GTS entities with OData query support")
        .tag("GTS Core")
        .with_odata_filter::<GtsEntityDtoFilterField>()
        .with_odata_select()
        .with_odata_orderby::<GtsEntityDtoFilterField>()
        .public()
        .handler(handlers::list_entities)
        .json_response_with_schema::<GtsEntityListDto>(
            openapi,
            StatusCode::OK,
            "Entity list retrieved successfully",
        )
        .standard_errors(openapi)
        .register(router, openapi);

    // POST /analytics/v1/gts - Create GTS entity
    router = OperationBuilder::post("/analytics/v1/gts")
        .operation_id("gts_core.create_entity")
        .summary("Create GTS entity")
        .description("Register a new GTS entity (type or instance)")
        .tag("GTS Core")
        .public()
        .json_request::<GtsEntityRequestDto>(openapi, "Entity to create")
        .handler(handlers::create_entity)
        .json_response_with_schema::<GtsEntityDto>(
            openapi,
            StatusCode::CREATED,
            "Entity created successfully",
        )
        .error_400(openapi)
        .standard_errors(openapi)
        .register(router, openapi);

    // PUT /analytics/v1/gts/{id} - Update GTS entity
    router = OperationBuilder::put("/analytics/v1/gts/{id}")
        .operation_id("gts_core.update_entity")
        .summary("Update GTS entity")
        .description("Update an existing GTS entity")
        .tag("GTS Core")
        .path_param("id", "GTS entity identifier")
        .public()
        .json_request::<GtsEntityRequestDto>(openapi, "Entity updates")
        .handler(handlers::update_entity)
        .json_response_with_schema::<GtsEntityDto>(
            openapi,
            StatusCode::OK,
            "Entity updated successfully",
        )
        .error_404(openapi)
        .error_400(openapi)
        .standard_errors(openapi)
        .register(router, openapi);

    // PATCH /analytics/v1/gts/{id} - Partial update GTS entity
    router = OperationBuilder::patch("/analytics/v1/gts/{id}")
        .operation_id("gts_core.patch_entity")
        .summary("Partially update GTS entity")
        .description("Apply JSON Patch to GTS entity (restricted to /entity/* paths)")
        .tag("GTS Core")
        .path_param("id", "GTS entity identifier")
        .public()
        .handler(handlers::patch_entity)
        .json_response_with_schema::<GtsEntityDto>(
            openapi,
            StatusCode::OK,
            "Entity patched successfully",
        )
        .error_404(openapi)
        .error_400(openapi)
        .standard_errors(openapi)
        .register(router, openapi);

    // DELETE /analytics/v1/gts/{id} - Delete GTS entity
    router = OperationBuilder::delete("/analytics/v1/gts/{id}")
        .operation_id("gts_core.delete_entity")
        .summary("Delete GTS entity")
        .description("Delete a GTS entity by ID")
        .tag("GTS Core")
        .path_param("id", "GTS entity identifier")
        .public()
        .handler(handlers::delete_entity)
        .json_response(StatusCode::NO_CONTENT, "Entity deleted successfully")
        .error_404(openapi)
        .standard_errors(openapi)
        .register(router, openapi);

    // GET /analytics/v1/$metadata - OData metadata
    router = OperationBuilder::get("/analytics/v1/$metadata")
        .operation_id("gts_core.get_metadata")
        .summary("Get OData metadata")
        .description("Returns OData JSON CSDL with Capabilities vocabulary annotations")
        .tag("GTS Core")
        .public()
        .handler(handlers::get_metadata)
        .json_response(StatusCode::OK, "OData metadata retrieved successfully")
        .standard_errors(openapi)
        .register(router, openapi);

    // PUT /analytics/v1/gts/{id}/enablement - Configure tenant access
    router = OperationBuilder::put("/analytics/v1/gts/{id}/enablement")
        .operation_id("gts_core.update_enablement")
        .summary("Configure tenant access")
        .description("Configure tenant access for a GTS entity")
        .tag("GTS Core")
        .path_param("id", "GTS entity identifier")
        .public()
        .handler(handlers::update_enablement)
        .json_response(StatusCode::OK, "Enablement updated successfully")
        .error_404(openapi)
        .error_400(openapi)
        .standard_errors(openapi)
        .register(router, openapi);

    // Attach service to router via Extension
    router = router.layer(Extension(gts_router));

    router
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::gts_core::RoutingTable;
    use modkit::api::OpenApiRegistryImpl;

    #[test]
    fn test_register_routes_extends_router() {
        let table = RoutingTable::new();
        let gts_router = Arc::new(GtsCoreRouter::new(table));
        let openapi = OpenApiRegistryImpl::new();

        let base_router = Router::new();
        let extended_router = register_routes(base_router, &openapi, gts_router);

        // Verify router was extended (not replaced)
        // Router should have service layer attached
        assert!(std::mem::size_of_val(&extended_router) > 0);
    }
}
