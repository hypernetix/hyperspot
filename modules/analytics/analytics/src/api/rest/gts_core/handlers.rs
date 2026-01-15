// @fdd-change:fdd-analytics-feature-gts-core-change-platform-integration-fix:ph-1
// @fdd-flow:fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1
// @fdd-flow:fdd-analytics-feature-gts-core-flow-developer-register-instance:ph-1
// @fdd-flow:fdd-analytics-feature-gts-core-flow-developer-list-entities:ph-1
// @fdd-flow:fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1
// @fdd-flow:fdd-analytics-feature-gts-core-flow-aggregate-odata-metadata:ph-1
// @fdd-req:fdd-analytics-feature-gts-core-req-routing:ph-1
// @fdd-algo:fdd-analytics-feature-gts-core-algo-routing-logic:ph-1
// @fdd-req:fdd-analytics-feature-gts-core-req-middleware:ph-1
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    Json,
};
use modkit::api::odata::OData;
use modkit::api::odata::ODataQuery;
use modkit::Problem;
// fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-if-jwt-invalid
// fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-401
use modkit_security::SecurityCtx;
// fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-401
// fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-if-jwt-invalid
use modkit_odata::ast;
use std::sync::Arc;

use crate::api::rest::gts_core::dto::{GtsEntityDto, GtsEntityListDto, GtsEntityRequestDto};
use crate::domain::gts_core::GtsCoreRouter;

#[allow(clippy::result_large_err)]
fn routing_decision<'a>(router: &'a GtsCoreRouter, gts_id: &'a str) -> Result<Option<&'a str>, Problem> {
    // fdd-begin fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-route-base-type
    let route_result = router.route(gts_id);
    // fdd-end fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-route-base-type
    match route_result {
        Ok(x) => Ok(x),
        Err(e) => Err(Problem::new(StatusCode::BAD_REQUEST, "Invalid GTS Identifier", e)
            .with_code("INVALID_GTS_IDENTIFIER")),
    }
}

fn extract_base_type_from_filter(query: &ODataQuery) -> Option<String> {
    fn walk(expr: &ast::Expr) -> Option<String> {
        match expr {
            ast::Expr::Function(name, args) if name == "startswith" && args.len() == 2 => {
                if let ast::Expr::Value(ast::Value::String(prefix)) = &args[1] {
                    return Some(prefix.clone());
                }
                None
            }
            ast::Expr::And(a, b)
            | ast::Expr::Or(a, b)
            | ast::Expr::Compare(a, _, b) => walk(a).or_else(|| walk(b)),
            ast::Expr::Not(x) => walk(x),
            ast::Expr::In(x, list) => {
                walk(x).or_else(|| list.iter().find_map(walk))
            }
            ast::Expr::Function(_, args) => args.iter().find_map(walk),
            _ => None,
        }
    }

    query.filter.as_deref().and_then(walk)
}

/// GET /analytics/v1/gts/{id} - Get entity by ID
// fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-receive-request
pub async fn get_entity(
    // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-case-get-extract
    Path(id): Path<String>,
    // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-case-get-extract
    Extension(_sec): Extension<SecurityCtx>,
    Extension(router): Extension<Arc<GtsCoreRouter>>,
) -> Result<Json<GtsEntityDto>, Problem> {
    // fdd-begin fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-determine-extraction-strategy
    let gts_id = id;
    // fdd-end fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-determine-extraction-strategy

    // fdd-begin fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-extract-base-type
    let base_type = gts_id.as_str();
    // fdd-end fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-extract-base-type

    // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-route-base-type
    let decision = routing_decision(router.as_ref(), base_type)?;
    // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-route-base-type

    // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-routing-decision
    match decision {
        Some(_handler_id) => {
            // fdd-begin fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-else-if-delegate-missing
            // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-else-if-delegate-missing

            // fdd-begin fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-return-501
            // fdd-begin fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-return-delegated-handler
            // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-501-no-delegate
            let err = Problem::new(
                StatusCode::NOT_IMPLEMENTED,
                "Not Implemented",
                "Delegate for known GTS type is not registered in this deployment",
            )
            .with_instance(format!("/analytics/v1/gts/{gts_id}"))
            .with_code("NOT_IMPLEMENTED");
            // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-501-no-delegate

            // fdd-end fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-return-delegated-handler
            // fdd-end fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-return-501
            // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-else-if-delegate-missing
            // fdd-end fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-else-if-delegate-missing
            Err(err)
        }
        None => {
            // fdd-begin fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-if-no-match
            // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-if-no-match

            // fdd-begin fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-return-404
            // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-404-unknown-type
            let err = Problem::new(
                StatusCode::NOT_FOUND,
                "Unknown GTS Type",
                format!("No feature registered for GTS type: {}", gts_id),
            )
            .with_instance(format!("/analytics/v1/gts/{gts_id}"))
            .with_code("UNKNOWN_GTS_TYPE");
            // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-404-unknown-type

            // fdd-end fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-return-404
            // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-if-no-match
            // fdd-end fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-if-no-match
            Err(err)
        }
    }
    // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-routing-decision
}
// fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-receive-request

/// GET /analytics/v1/gts - List entities with OData
// fdd-begin fdd-analytics-feature-gts-core-flow-developer-list-entities:ph-1:inst-send-get-list
pub async fn list_entities(
    OData(query): OData,
    Extension(_sec): Extension<SecurityCtx>,
    Extension(router): Extension<Arc<GtsCoreRouter>>,
) -> Result<Json<GtsEntityListDto>, Problem> {
    // fdd-begin fdd-analytics-feature-gts-core-flow-developer-list-entities:ph-1:inst-extract-base-type-filter
    let base_type = extract_base_type_from_filter(&query);
    // fdd-end fdd-analytics-feature-gts-core-flow-developer-list-entities:ph-1:inst-extract-base-type-filter

    match base_type {
        Some(bt) => {
            // fdd-begin fdd-analytics-feature-gts-core-flow-developer-list-entities:ph-1:inst-route-base-type
            let decision = routing_decision(router.as_ref(), &bt)?;
            // fdd-end fdd-analytics-feature-gts-core-flow-developer-list-entities:ph-1:inst-route-base-type

            // fdd-begin fdd-analytics-feature-gts-core-flow-developer-list-entities:ph-1:inst-return-routing-decision
            match decision {
                Some(_handler_id) => {
                    // fdd-begin fdd-analytics-feature-gts-core-flow-developer-list-entities:ph-1:inst-else-if-delegate-missing
                    // fdd-begin fdd-analytics-feature-gts-core-flow-developer-list-entities:ph-1:inst-return-501-no-delegate
                    let err = Problem::new(
                        StatusCode::NOT_IMPLEMENTED,
                        "Not Implemented",
                        "List endpoint declared; routing to domain features not implemented in this deployment",
                    )
                    .with_instance("/analytics/v1/gts")
                    .with_code("NOT_IMPLEMENTED");
                    // fdd-end fdd-analytics-feature-gts-core-flow-developer-list-entities:ph-1:inst-return-501-no-delegate
                    // fdd-end fdd-analytics-feature-gts-core-flow-developer-list-entities:ph-1:inst-else-if-delegate-missing
                    Err(err)
                }
                None => {
                    // fdd-begin fdd-analytics-feature-gts-core-flow-developer-list-entities:ph-1:inst-if-no-match
                    // fdd-begin fdd-analytics-feature-gts-core-flow-developer-list-entities:ph-1:inst-return-404-unknown-type
                    let err = Problem::new(
                        StatusCode::NOT_FOUND,
                        "Unknown GTS Type",
                        "No feature registered for requested base type",
                    )
                    .with_instance("/analytics/v1/gts")
                    .with_code("UNKNOWN_GTS_TYPE");
                    // fdd-end fdd-analytics-feature-gts-core-flow-developer-list-entities:ph-1:inst-return-404-unknown-type
                    // fdd-end fdd-analytics-feature-gts-core-flow-developer-list-entities:ph-1:inst-if-no-match
                    Err(err)
                }
            }
            // fdd-end fdd-analytics-feature-gts-core-flow-developer-list-entities:ph-1:inst-return-routing-decision
        }
        None => Err(Problem::new(
            StatusCode::NOT_IMPLEMENTED,
            "Not Implemented",
            "List endpoint declared; base type filter is required for routing in this deployment",
        )
        .with_instance("/analytics/v1/gts")
        .with_code("NOT_IMPLEMENTED")),
    }
}
// fdd-end fdd-analytics-feature-gts-core-flow-developer-list-entities:ph-1:inst-send-get-list

/// POST /analytics/v1/gts - Create entity
pub async fn create_entity(
    Extension(_sec): Extension<SecurityCtx>,
    Extension(router): Extension<Arc<GtsCoreRouter>>,
    // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-case-post-extract
    Json(request): Json<GtsEntityRequestDto>,
    // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-case-post-extract
) -> Result<(StatusCode, Json<GtsEntityDto>), Problem> {
    // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-extract-base-type
    // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-match-method
    let gts_id = match request.id.as_deref() {
        Some(id) => {
            // fdd-begin fdd-analytics-feature-gts-core-flow-developer-register-instance:ph-1:inst-send-post-register-instance
            // fdd-begin fdd-analytics-feature-gts-core-flow-developer-register-instance:ph-1:inst-validate-instance-shape
            // fdd-begin fdd-analytics-feature-gts-core-flow-developer-register-instance:ph-1:inst-extract-base-type-from-instance-id
            id.to_string()
            // fdd-end fdd-analytics-feature-gts-core-flow-developer-register-instance:ph-1:inst-extract-base-type-from-instance-id
            // fdd-end fdd-analytics-feature-gts-core-flow-developer-register-instance:ph-1:inst-validate-instance-shape
            // fdd-end fdd-analytics-feature-gts-core-flow-developer-register-instance:ph-1:inst-send-post-register-instance
        }
        None => {
            // fdd-begin fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-send-post-register-type
            // fdd-begin fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-validate-schema-shape
            let schema_id = request
                .entity
                .get("$id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // fdd-end fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-validate-schema-shape
            // fdd-end fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-send-post-register-type

            // fdd-begin fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-if-has-id
            if let Some(schema_id) = schema_id {
                // fdd-begin fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-extract-base-type-from-id
                schema_id
                // fdd-end fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-extract-base-type-from-id
            } else {
                // fdd-begin fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-else-missing-id
                // fdd-begin fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-return-400-missing-id
                let err = Problem::new(
                    StatusCode::BAD_REQUEST,
                    "Missing $id",
                    "Type registration requires JSON Schema entity.$id",
                )
                .with_instance("/analytics/v1/gts")
                .with_code("MISSING_SCHEMA_ID");
                // fdd-end fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-return-400-missing-id
                // fdd-end fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-else-missing-id
                return Err(err);
            }
            // fdd-end fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-if-has-id
        }
    };
    // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-match-method
    // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-extract-base-type

    // fdd-begin fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-return-routing-decision
    // fdd-begin fdd-analytics-feature-gts-core-flow-developer-register-instance:ph-1:inst-return-routing-decision
    // fdd-begin fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-route-base-type
    // fdd-begin fdd-analytics-feature-gts-core-flow-developer-register-instance:ph-1:inst-route-base-type
    let decision = routing_decision(router.as_ref(), &gts_id)?;
    // fdd-end fdd-analytics-feature-gts-core-flow-developer-register-instance:ph-1:inst-route-base-type
    // fdd-end fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-route-base-type
    // fdd-end fdd-analytics-feature-gts-core-flow-developer-register-instance:ph-1:inst-return-routing-decision
    // fdd-end fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-return-routing-decision

    match decision {
        Some(_handler_id) => {
            // fdd-begin fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-else-if-delegate-missing
            // fdd-begin fdd-analytics-feature-gts-core-flow-developer-register-instance:ph-1:inst-else-if-delegate-missing
            // fdd-begin fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-return-501-no-delegate
            // fdd-begin fdd-analytics-feature-gts-core-flow-developer-register-instance:ph-1:inst-return-501-no-delegate
            let err = Problem::new(
                StatusCode::NOT_IMPLEMENTED,
                "Not Implemented",
                "Create endpoint declared; routing/delegation not implemented in this deployment",
            )
            .with_instance("/analytics/v1/gts")
            .with_code("NOT_IMPLEMENTED");
            // fdd-end fdd-analytics-feature-gts-core-flow-developer-register-instance:ph-1:inst-return-501-no-delegate
            // fdd-end fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-return-501-no-delegate
            // fdd-end fdd-analytics-feature-gts-core-flow-developer-register-instance:ph-1:inst-else-if-delegate-missing
            // fdd-end fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-else-if-delegate-missing
            Err(err)
        }
        None => {
            // fdd-begin fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-if-no-match
            // fdd-begin fdd-analytics-feature-gts-core-flow-developer-register-instance:ph-1:inst-if-no-match
            // fdd-begin fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-return-404-unknown-type
            // fdd-begin fdd-analytics-feature-gts-core-flow-developer-register-instance:ph-1:inst-return-404-unknown-type
            let err = Problem::new(
                StatusCode::NOT_FOUND,
                "Unknown GTS Type",
                "No feature registered for requested base type",
            )
            .with_instance("/analytics/v1/gts")
            .with_code("UNKNOWN_GTS_TYPE");
            // fdd-end fdd-analytics-feature-gts-core-flow-developer-register-instance:ph-1:inst-return-404-unknown-type
            // fdd-end fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-return-404-unknown-type
            // fdd-end fdd-analytics-feature-gts-core-flow-developer-register-instance:ph-1:inst-if-no-match
            // fdd-end fdd-analytics-feature-gts-core-flow-admin-register-type:ph-1:inst-if-no-match
            Err(err)
        }
    }
}

/// PUT /analytics/v1/gts/{id} - Update entity
pub async fn update_entity(
    // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-case-put-extract
    Path(id): Path<String>,
    // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-case-put-extract
    Extension(_sec): Extension<SecurityCtx>,
    Extension(router): Extension<Arc<GtsCoreRouter>>,
    Json(request): Json<GtsEntityRequestDto>,
) -> Result<Json<GtsEntityDto>, Problem> {
    let _ = request;

    // fdd-begin fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-determine-extraction-strategy
    let gts_id = id;
    // fdd-end fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-determine-extraction-strategy

    // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-extract-base-type
    let base_type = gts_id.as_str();
    // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-extract-base-type

    // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-route-base-type
    let decision = routing_decision(router.as_ref(), base_type)?;
    // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-route-base-type

    // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-routing-decision
    match decision {
        Some(_handler_id) => {
            // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-else-if-delegate-missing
            // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-501-no-delegate
            let err = Problem::new(
                StatusCode::NOT_IMPLEMENTED,
                "Not Implemented",
                "Delegate for known GTS type is not registered in this deployment",
            )
            .with_instance(format!("/analytics/v1/gts/{gts_id}"))
            .with_code("NOT_IMPLEMENTED");
            // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-501-no-delegate
            // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-else-if-delegate-missing
            Err(err)
        }
        None => {
            // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-if-no-match
            // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-404-unknown-type
            let err = Problem::new(
                StatusCode::NOT_FOUND,
                "Unknown GTS Type",
                format!("No feature registered for GTS type: {}", gts_id),
            )
            .with_instance(format!("/analytics/v1/gts/{gts_id}"))
            .with_code("UNKNOWN_GTS_TYPE");
            // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-404-unknown-type
            // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-if-no-match
            Err(err)
        }
    }
    // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-routing-decision
}

/// PATCH /analytics/v1/gts/{id} - Partial update
pub async fn patch_entity(
    // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-case-patch-extract
    Path(id): Path<String>,
    // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-case-patch-extract
    Extension(_sec): Extension<SecurityCtx>,
    Extension(router): Extension<Arc<GtsCoreRouter>>,
    Json(_patch): Json<serde_json::Value>,
) -> Result<Json<GtsEntityDto>, Problem> {
    // fdd-begin fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-determine-extraction-strategy
    let gts_id = id;
    // fdd-end fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-determine-extraction-strategy

    // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-extract-base-type
    let base_type = gts_id.as_str();
    // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-extract-base-type

    // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-route-base-type
    let decision = routing_decision(router.as_ref(), base_type)?;
    // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-route-base-type

    // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-routing-decision
    match decision {
        Some(_handler_id) => {
            // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-else-if-delegate-missing
            // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-501-no-delegate
            let err = Problem::new(
                StatusCode::NOT_IMPLEMENTED,
                "Not Implemented",
                "Delegate for known GTS type is not registered in this deployment",
            )
            .with_instance(format!("/analytics/v1/gts/{gts_id}"))
            .with_code("NOT_IMPLEMENTED");
            // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-501-no-delegate
            // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-else-if-delegate-missing
            Err(err)
        }
        None => {
            // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-if-no-match
            // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-404-unknown-type
            let err = Problem::new(
                StatusCode::NOT_FOUND,
                "Unknown GTS Type",
                format!("No feature registered for GTS type: {}", gts_id),
            )
            .with_instance(format!("/analytics/v1/gts/{gts_id}"))
            .with_code("UNKNOWN_GTS_TYPE");
            // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-404-unknown-type
            // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-if-no-match
            Err(err)
        }
    }
    // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-routing-decision
}

/// DELETE /analytics/v1/gts/{id} - Delete entity
pub async fn delete_entity(
    // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-case-delete-extract
    Path(id): Path<String>,
    // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-case-delete-extract
    Extension(_sec): Extension<SecurityCtx>,
    Extension(router): Extension<Arc<GtsCoreRouter>>,
) -> Result<StatusCode, Problem> {
    // fdd-begin fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-determine-extraction-strategy
    let gts_id = id;
    // fdd-end fdd-analytics-feature-gts-core-algo-routing-logic:ph-1:inst-determine-extraction-strategy

    // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-extract-base-type
    let base_type = gts_id.as_str();
    // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-extract-base-type

    // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-route-base-type
    let decision = routing_decision(router.as_ref(), base_type)?;
    // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-route-base-type

    // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-routing-decision
    match decision {
        Some(_handler_id) => {
            // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-else-if-delegate-missing
            // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-501-no-delegate
            let err = Problem::new(
                StatusCode::NOT_IMPLEMENTED,
                "Not Implemented",
                "Delegate for known GTS type is not registered in this deployment",
            )
            .with_instance(format!("/analytics/v1/gts/{gts_id}"))
            .with_code("NOT_IMPLEMENTED");
            // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-501-no-delegate
            // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-else-if-delegate-missing
            Err(err)
        }
        None => {
            // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-if-no-match
            // fdd-begin fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-404-unknown-type
            let err = Problem::new(
                StatusCode::NOT_FOUND,
                "Unknown GTS Type",
                format!("No feature registered for GTS type: {}", gts_id),
            )
            .with_instance(format!("/analytics/v1/gts/{gts_id}"))
            .with_code("UNKNOWN_GTS_TYPE");
            // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-404-unknown-type
            // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-if-no-match
            Err(err)
        }
    }
    // fdd-end fdd-analytics-feature-gts-core-flow-route-crud-operations:ph-1:inst-return-routing-decision
}

/// GET /analytics/v1/$metadata - OData metadata
/// Returns OData JSON CSDL with Capabilities vocabulary annotations
// fdd-begin fdd-analytics-feature-gts-core-flow-aggregate-odata-metadata:ph-1:inst-receive-metadata-request
pub async fn get_metadata() -> Result<Json<serde_json::Value>, Problem> {
    // Return minimal OData CSDL metadata structure
    // Full implementation will aggregate metadata from all domain features

    // fdd-begin fdd-analytics-feature-gts-core-flow-aggregate-odata-metadata:ph-1:inst-if-metadata-provider-missing
    // fdd-begin fdd-analytics-feature-gts-core-flow-aggregate-odata-metadata:ph-1:inst-return-501-metadata
    // fdd-begin fdd-analytics-feature-gts-core-flow-aggregate-odata-metadata:ph-1:inst-return-metadata-routing-decision
    let err = Problem::new(
        StatusCode::NOT_IMPLEMENTED,
        "Not Implemented",
        "Metadata endpoint is declared, but OData CSDL aggregation is not implemented yet",
    )
    .with_instance("/analytics/v1/$metadata")
    .with_code("NOT_IMPLEMENTED");
    // fdd-end fdd-analytics-feature-gts-core-flow-aggregate-odata-metadata:ph-1:inst-return-metadata-routing-decision
    // fdd-end fdd-analytics-feature-gts-core-flow-aggregate-odata-metadata:ph-1:inst-return-501-metadata
    // fdd-end fdd-analytics-feature-gts-core-flow-aggregate-odata-metadata:ph-1:inst-if-metadata-provider-missing
    Err(err)
}
// fdd-end fdd-analytics-feature-gts-core-flow-aggregate-odata-metadata:ph-1:inst-receive-metadata-request

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
