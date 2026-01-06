use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use modkit::Problem;

use crate::domain::gts_core::GtsCoreRouter;
use crate::api::rest::gts_core::ResponseProcessor;

pub async fn handle_gts_request(
    State(router): State<Arc<GtsCoreRouter>>,
    Path(id): Path<String>,
) -> Response {
    match router.route(&id) {
        Ok(Some(feature_name)) => {
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "routed_to": feature_name,
                    "gts_id": id,
                })),
            )
                .into_response()
        }
        Ok(None) => {
            let problem = Problem::new(
                StatusCode::NOT_FOUND,
                "Unknown GTS Type",
                format!("No feature registered for GTS type in identifier: {}", id),
            );
            problem.into_response()
        }
        Err(e) => {
            let problem = Problem::new(
                StatusCode::BAD_REQUEST,
                "Invalid GTS Identifier",
                e,
            );
            problem.into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::gts_core::RoutingTable;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_handler_routes_valid_request() {
        let mut table = RoutingTable::new();
        table.register("gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.test.v1", "feature-one").unwrap();
        let router = Arc::new(GtsCoreRouter::new(table));

        let app = axum::Router::new()
            .route("/gts/{id}", axum::routing::get(handle_gts_request))
            .with_state(router);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/gts/gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.instance_123.v1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handler_returns_404_for_unknown_type() {
        let table = RoutingTable::new();
        let router = Arc::new(GtsCoreRouter::new(table));

        let app = axum::Router::new()
            .route("/gts/{id}", axum::routing::get(handle_gts_request))
            .with_state(router);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/gts/gts.hypernetix.hyperspot.ax.unknown_type.v1~acme.analytics._.instance.v1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_handler_returns_400_for_invalid_identifier() {
        let table = RoutingTable::new();
        let router = Arc::new(GtsCoreRouter::new(table));

        let app = axum::Router::new()
            .route("/gts/{id}", axum::routing::get(handle_gts_request))
            .with_state(router);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/gts/invalid_identifier")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_end_to_end_create_with_system_fields() {
        use serde_json::json;
        
        let processor = ResponseProcessor::new();
        let input = json!({
            "id": "custom-id-override",
            "type": "custom-type-override",
            "tenant": "custom-tenant-override",
            "entity": {
                "name": "Test Entity"
            }
        });
        
        let filtered = processor.process_request(input);
        
        assert!(filtered.get("id").is_none());
        assert!(filtered.get("type").is_none());
        assert!(filtered.get("tenant").is_none());
        assert!(filtered.get("entity").is_some());
        assert_eq!(
            filtered.get("entity").unwrap().get("name").unwrap().as_str().unwrap(),
            "Test Entity"
        );
    }

    #[tokio::test]
    async fn test_end_to_end_read_with_secrets() {
        use serde_json::json;
        
        let processor = ResponseProcessor::new();
        let entity_with_secrets = json!({
            "id": "test-id",
            "type": "test-type",
            "entity": {
                "name": "Test",
                "description": "Public description",
                "api_key": "secret_key_12345",
                "credentials": "secret_credentials"
            }
        });
        
        let response = processor.process_response(
            entity_with_secrets,
            Some("test-id"),
            Some("test-type"),
            None
        );
        
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_end_to_end_patch_with_path_validation() {
        use serde_json::json;
        
        let processor = ResponseProcessor::new();
        
        let valid_ops = vec![
            json!({"op": "replace", "path": "/entity/name", "value": "New Name"}),
            json!({"op": "add", "path": "/entity/description", "value": "New Description"}),
        ];
        assert!(processor.validate_patch_operations(&valid_ops).is_ok());
        
        let invalid_ops_id = vec![
            json!({"op": "replace", "path": "/id", "value": "new-id"}),
        ];
        assert!(processor.validate_patch_operations(&invalid_ops_id).is_err());
        
        let invalid_ops_type = vec![
            json!({"op": "replace", "path": "/type", "value": "new-type"}),
        ];
        assert!(processor.validate_patch_operations(&invalid_ops_type).is_err());
    }

    #[test]
    fn test_field_projection_with_select() {
        use serde_json::json;
        use crate::domain::gts_core::FieldHandler;
        
        let handler = FieldHandler::new();
        let entity = json!({
            "id": "test-id",
            "type": "test-type",
            "entity": {
                "name": "Test",
                "description": "Desc",
                "value": 123,
                "created_at": "2024-01-01"
            }
        });
        
        let select_fields = vec![
            "id".to_string(),
            "entity/name".to_string(),
            "entity/created_at".to_string()
        ];
        
        let projected = handler.apply_field_projection(entity, &select_fields);
        
        assert!(projected.get("id").is_some());
        assert!(projected.get("type").is_none());
        
        let entity_obj = projected.get("entity").unwrap().as_object().unwrap();
        assert!(entity_obj.get("name").is_some());
        assert!(entity_obj.get("created_at").is_some());
        assert!(entity_obj.get("description").is_none());
        assert!(entity_obj.get("value").is_none());
    }

    #[test]
    fn test_nested_field_filtering() {
        use serde_json::json;
        use crate::domain::gts_core::FieldHandler;
        
        let handler = FieldHandler::new();
        let entity = json!({
            "id": "test-id",
            "entity": {
                "name": "Test",
                "api_key": "secret123",
                "nested": {
                    "value": "data"
                }
            }
        });
        
        let filtered = handler.filter_response(entity);
        
        let entity_obj = filtered.get("entity").unwrap().as_object().unwrap();
        assert!(entity_obj.get("name").is_some());
        assert!(entity_obj.get("api_key").is_none());
        assert!(entity_obj.get("nested").is_some());
    }

    #[test]
    fn test_computed_fields_with_missing_dependencies() {
        use serde_json::json;
        use crate::domain::gts_core::FieldHandler;
        
        let handler = FieldHandler::new();
        let entity = json!({
            "entity": {"name": "Test"}
        });
        
        let result = handler.inject_computed_fields(entity, "test-id", "test-type");
        
        assert!(result.get("asset_path").is_some());
        assert_eq!(
            result.get("asset_path").unwrap().as_str().unwrap(),
            "/api/analytics/v1/gts/test-id"
        );
    }

    #[test]
    fn test_patch_with_mixed_valid_invalid_paths() {
        use serde_json::json;
        
        let processor = ResponseProcessor::new();
        
        let mixed_ops = vec![
            json!({"op": "replace", "path": "/entity/name", "value": "Valid"}),
            json!({"op": "replace", "path": "/id", "value": "Invalid"}),
        ];
        
        let result = processor.validate_patch_operations(&mixed_ops);
        assert!(result.is_err());
    }
}
