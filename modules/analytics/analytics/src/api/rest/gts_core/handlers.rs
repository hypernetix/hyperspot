// @fdd-change:fdd-analytics-feature-gts-core-change-platform-integration-fix
// @fdd-flow:fdd-analytics-feature-gts-core-flow-admin-register-type
// @fdd-flow:fdd-analytics-feature-gts-core-flow-developer-register-instance
// @fdd-flow:fdd-analytics-feature-gts-core-flow-developer-list-entities
// @fdd-flow:fdd-analytics-feature-gts-core-flow-route-crud-operations
// @fdd-flow:fdd-analytics-feature-gts-core-flow-aggregate-odata-metadata
// @fdd-req:fdd-analytics-feature-gts-core-req-routing
// @fdd-algo:fdd-analytics-feature-gts-core-algo-routing-logic
// @fdd-req:fdd-analytics-feature-gts-core-req-middleware
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    Json,
};
use modkit::api::odata::OData;
use modkit::Problem;
use modkit_security::SecurityCtx;
use std::sync::Arc;

use crate::api::rest::gts_core::dto::{GtsEntityDto, GtsEntityListDto, GtsEntityRequestDto};
use crate::domain::gts_core::GtsCoreRouter;

/// GET /analytics/v1/gts/{id} - Get entity by ID
pub async fn get_entity(
    Path(id): Path<String>,
    Extension(_sec): Extension<SecurityCtx>,
    Extension(router): Extension<Arc<GtsCoreRouter>>,
) -> Result<Json<GtsEntityDto>, Problem> {
    match router.route(&id) {
        Ok(Some(_handler_id)) => Err(Problem::new(
            StatusCode::NOT_IMPLEMENTED,
            "Not Implemented",
            "Delegate for known GTS type is not registered in this deployment",
        )
        .with_instance(format!("/analytics/v1/gts/{id}"))
        .with_code("NOT_IMPLEMENTED")),
        Ok(None) => Err(Problem::new(
            StatusCode::NOT_FOUND,
            "Unknown GTS Type",
            format!("No feature registered for GTS type: {}", id),
        )
        .with_instance(format!("/analytics/v1/gts/{id}"))
        .with_code("UNKNOWN_GTS_TYPE")),
        Err(e) => Err(
            Problem::new(StatusCode::BAD_REQUEST, "Invalid GTS Identifier", e)
                .with_instance(format!("/analytics/v1/gts/{id}"))
                .with_code("INVALID_GTS_IDENTIFIER"),
        ),
    }
}

/// GET /analytics/v1/gts - List entities with OData
pub async fn list_entities(
    OData(_query): OData,
    Extension(_sec): Extension<SecurityCtx>,
    Extension(_router): Extension<Arc<GtsCoreRouter>>,
) -> Result<Json<GtsEntityListDto>, Problem> {
    Err(Problem::new(
        StatusCode::NOT_IMPLEMENTED,
        "Not Implemented",
        "List endpoint declared; routing to domain features not implemented in this deployment",
    )
    .with_instance("/analytics/v1/gts")
    .with_code("NOT_IMPLEMENTED"))
}

/// POST /analytics/v1/gts - Create entity
pub async fn create_entity(
    Extension(_sec): Extension<SecurityCtx>,
    Extension(_router): Extension<Arc<GtsCoreRouter>>,
    Json(request): Json<GtsEntityRequestDto>,
) -> Result<(StatusCode, Json<GtsEntityDto>), Problem> {
    let _ = request;
    Err(Problem::new(
        StatusCode::NOT_IMPLEMENTED,
        "Not Implemented",
        "Create endpoint declared; routing/delegation not implemented in this deployment",
    )
    .with_instance("/analytics/v1/gts")
    .with_code("NOT_IMPLEMENTED"))
}

/// PUT /analytics/v1/gts/{id} - Update entity
pub async fn update_entity(
    Path(id): Path<String>,
    Extension(_sec): Extension<SecurityCtx>,
    Extension(_router): Extension<Arc<GtsCoreRouter>>,
    Json(request): Json<GtsEntityRequestDto>,
) -> Result<Json<GtsEntityDto>, Problem> {
    let _ = request;
    Err(Problem::new(
        StatusCode::NOT_IMPLEMENTED,
        "Not Implemented",
        "Update endpoint declared; routing/delegation not implemented in this deployment",
    )
    .with_instance(format!("/analytics/v1/gts/{id}"))
    .with_code("NOT_IMPLEMENTED"))
}

/// PATCH /analytics/v1/gts/{id} - Partial update
pub async fn patch_entity(
    Path(id): Path<String>,
    Extension(_sec): Extension<SecurityCtx>,
    Extension(_router): Extension<Arc<GtsCoreRouter>>,
    Json(_patch): Json<serde_json::Value>,
) -> Result<Json<GtsEntityDto>, Problem> {
    Err(Problem::new(
        StatusCode::NOT_IMPLEMENTED,
        "Not Implemented",
        "Patch endpoint declared; JSON Patch routing/delegation not implemented in this deployment",
    )
    .with_instance(format!("/analytics/v1/gts/{id}"))
    .with_code("NOT_IMPLEMENTED"))
}

/// DELETE /analytics/v1/gts/{id} - Delete entity
pub async fn delete_entity(
    Path(_id): Path<String>,
    Extension(_sec): Extension<SecurityCtx>,
    Extension(_router): Extension<Arc<GtsCoreRouter>>,
) -> Result<StatusCode, Problem> {
    Err(Problem::new(
        StatusCode::NOT_IMPLEMENTED,
        "Not Implemented",
        "Delete endpoint declared; routing/delegation not implemented in this deployment",
    )
    .with_instance("/analytics/v1/gts/{id}")
    .with_code("NOT_IMPLEMENTED"))
}

/// GET /analytics/v1/$metadata - OData metadata
/// Returns OData JSON CSDL with Capabilities vocabulary annotations
pub async fn get_metadata() -> Result<Json<serde_json::Value>, Problem> {
    // Return minimal OData CSDL metadata structure
    // Full implementation will aggregate metadata from all domain features
    Err(Problem::new(
        StatusCode::NOT_IMPLEMENTED,
        "Not Implemented",
        "Metadata endpoint is declared, but OData CSDL aggregation is not implemented yet",
    )
    .with_instance("/analytics/v1/$metadata")
    .with_code("NOT_IMPLEMENTED"))
}

/// PUT /analytics/v1/gts/{id}/enablement - Configure tenant access
pub async fn update_enablement(
    Path(id): Path<String>,
    Extension(_sec): Extension<SecurityCtx>,
    Extension(_router): Extension<Arc<GtsCoreRouter>>,
    Json(_enablement): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, Problem> {
    Err(Problem::new(
        StatusCode::NOT_IMPLEMENTED,
        "Not Implemented",
        "Enablement endpoint is declared, but tenant access configuration is not implemented yet",
    )
    .with_instance(format!("/analytics/v1/gts/{id}/enablement"))
    .with_code("NOT_IMPLEMENTED"))
}

#[cfg(test)]
mod tests {
    // Old tests removed as they tested anti-pattern implementation
}
