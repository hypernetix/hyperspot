use axum::{
    extract::{Path, Query, Extension},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use modkit::Problem;
use modkit_security::SecurityCtx;
use serde::Deserialize;

use crate::domain::gts_core::GtsCoreRouter;
use crate::api::rest::gts_core::dto::{GtsEntityDto, GtsEntityRequestDto, GtsEntityListDto};

#[derive(Debug, Deserialize)]
pub struct ODataQueryParams {
    #[serde(rename = "$filter")]
    pub filter: Option<String>,
    #[serde(rename = "$select")]
    pub select: Option<String>,
    #[serde(rename = "$top")]
    pub top: Option<i32>,
    #[serde(rename = "$skip")]
    pub skip: Option<i32>,
    #[serde(rename = "$count")]
    pub count: Option<bool>,
}

/// GET /analytics/v1/gts/{id} - Get entity by ID
///
/// # Example: SecurityCtx Usage
/// ```rust,ignore
/// // SecurityCtx automatically injected by api_ingress middleware
/// pub async fn get_entity(
///     Path(id): Path<String>,
///     Extension(ctx): Extension<SecurityCtx>,  // ← Automatic injection
///     Extension(router): Extension<Arc<GtsCoreRouter>>,
/// ) -> Result<Json<GtsEntityDto>, Problem> {
///     // ctx.scope() provides tenant isolation
///     let tenant = format!("{:?}", ctx.scope());
///     // ...
/// }
/// ```
pub async fn get_entity(
    Path(id): Path<String>,
    Extension(ctx): Extension<SecurityCtx>,
    Extension(router): Extension<Arc<GtsCoreRouter>>,
) -> Result<Json<GtsEntityDto>, Problem> {
    // Route to appropriate domain feature
    match router.route(&id) {
        Ok(Some(feature_name)) => {
            // TODO: Actual domain feature call
            Ok(Json(GtsEntityDto {
                id: id.clone(),
                type_id: "gts.example.type.v1~".to_string(),
                entity: serde_json::json!({"routed_to": feature_name}),
                tenant: format!("{:?}", ctx.scope()),
                registered_at: chrono::Utc::now().to_rfc3339(),
            }))
        }
        Ok(None) => Err(Problem::new(
            StatusCode::NOT_FOUND,
            "Unknown GTS Type",
            format!("No feature registered for GTS type: {}", id)
        )),
        Err(e) => Err(Problem::new(
            StatusCode::BAD_REQUEST,
            "Invalid GTS Identifier",
            e
        )),
    }
}

/// GET /analytics/v1/gts - List entities with OData
pub async fn list_entities(
    Query(params): Query<ODataQueryParams>,
    Extension(_ctx): Extension<SecurityCtx>,
    Extension(_router): Extension<Arc<GtsCoreRouter>>,
) -> Result<Json<GtsEntityListDto>, Problem> {
    // TODO: Implement actual list logic with OData filters
    Ok(Json(GtsEntityListDto {
        odata_context: "/api/analytics/v1/$metadata#gts".to_string(),
        odata_count: if params.count.unwrap_or(false) { Some(0) } else { None },
        odata_next_link: None,
        value: vec![],
    }))
}

/// POST /analytics/v1/gts - Create entity
pub async fn create_entity(
    Extension(ctx): Extension<SecurityCtx>,
    Extension(_router): Extension<Arc<GtsCoreRouter>>,
    Json(request): Json<GtsEntityRequestDto>,
) -> Result<(StatusCode, Json<GtsEntityDto>), Problem> {
    // TODO: Validate and create entity
    let entity_id = request.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    Ok((StatusCode::CREATED, Json(GtsEntityDto {
        id: entity_id.clone(),
        type_id: "gts.example.type.v1~".to_string(),
        entity: request.entity,
        tenant: format!("{:?}", ctx.scope()),
        registered_at: chrono::Utc::now().to_rfc3339(),
    })))
}

/// PUT /analytics/v1/gts/{id} - Update entity
pub async fn update_entity(
    Path(id): Path<String>,
    Extension(ctx): Extension<SecurityCtx>,
    Extension(_router): Extension<Arc<GtsCoreRouter>>,
    Json(request): Json<GtsEntityRequestDto>,
) -> Result<Json<GtsEntityDto>, Problem> {
    // TODO: Validate and update entity
    Ok(Json(GtsEntityDto {
        id: id.clone(),
        type_id: "gts.example.type.v1~".to_string(),
        entity: request.entity,
        tenant: format!("{:?}", ctx.scope()),
        registered_at: chrono::Utc::now().to_rfc3339(),
    }))
}

/// PATCH /analytics/v1/gts/{id} - Partial update
pub async fn patch_entity(
    Path(id): Path<String>,
    Extension(ctx): Extension<SecurityCtx>,
    Extension(_router): Extension<Arc<GtsCoreRouter>>,
    Json(_patch): Json<serde_json::Value>,
) -> Result<Json<GtsEntityDto>, Problem> {
    // TODO: Validate patch operations (only /entity/* paths allowed)
    // TODO: Apply patch
    Ok(Json(GtsEntityDto {
        id: id.clone(),
        type_id: "gts.example.type.v1~".to_string(),
        entity: serde_json::json!({"patched": true}),
        tenant: format!("{:?}", ctx.scope()),
        registered_at: chrono::Utc::now().to_rfc3339(),
    }))
}

/// DELETE /analytics/v1/gts/{id} - Delete entity
pub async fn delete_entity(
    Path(_id): Path<String>,
    Extension(_ctx): Extension<SecurityCtx>,
    Extension(_router): Extension<Arc<GtsCoreRouter>>,
) -> Result<StatusCode, Problem> {
    // TODO: Delete entity
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Add integration tests using api_ingress test helpers
    // Old tests removed as they tested anti-pattern implementation
}
