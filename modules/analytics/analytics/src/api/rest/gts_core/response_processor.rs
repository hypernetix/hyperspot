use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::Value;
use crate::domain::gts_core::FieldHandler;
use modkit::Problem;

pub struct ResponseProcessor {
    field_handler: FieldHandler,
}

impl ResponseProcessor {
    pub fn new() -> Self {
        Self {
            field_handler: FieldHandler::new(),
        }
    }

    pub fn process_response(
        &self,
        mut value: Value,
        id: Option<&str>,
        type_id: Option<&str>,
        select_fields: Option<&[String]>,
    ) -> Response {
        value = self.field_handler.filter_response(value);
        
        if let (Some(id), Some(type_id)) = (id, type_id) {
            value = self.field_handler.inject_computed_fields(value, id, type_id);
        }
        
        if let Some(fields) = select_fields {
            value = self.field_handler.apply_field_projection(value, fields);
        }
        
        Json(value).into_response()
    }

    pub fn process_request(&self, value: Value) -> Value {
        self.field_handler.filter_request(value)
    }

    pub fn validate_patch_operations(&self, operations: &[serde_json::Value]) -> Result<(), Response> {
        for op in operations {
            if let Some(path) = op.get("path").and_then(|p| p.as_str()) {
                if let Err(err) = self.field_handler.validate_patch_path(path) {
                    let problem = Problem::new(
                        StatusCode::BAD_REQUEST,
                        "Invalid JSON Patch Path",
                        err,
                    );
                    return Err(problem.into_response());
                }
            }
        }
        Ok(())
    }
}

impl Default for ResponseProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_process_response_filters_secrets() {
        let processor = ResponseProcessor::new();
        let input = json!({
            "id": "test-id",
            "entity": {
                "name": "Test",
                "api_key": "secret"
            }
        });
        
        let response = processor.process_response(input, Some("test-id"), Some("test-type"), None);
        let status = response.status();
        assert_eq!(status, StatusCode::OK);
    }

    #[test]
    fn test_process_request_removes_system_fields() {
        let processor = ResponseProcessor::new();
        let input = json!({
            "id": "custom",
            "type": "custom",
            "entity": {"name": "Test"}
        });
        
        let result = processor.process_request(input);
        assert!(result.get("id").is_none());
        assert!(result.get("type").is_none());
        assert!(result.get("entity").is_some());
    }

    #[test]
    fn test_validate_patch_operations_allows_entity_paths() {
        let processor = ResponseProcessor::new();
        let operations = vec![
            json!({"op": "replace", "path": "/entity/name", "value": "New"}),
            json!({"op": "add", "path": "/entity/description", "value": "Desc"}),
        ];
        
        assert!(processor.validate_patch_operations(&operations).is_ok());
    }

    #[test]
    fn test_validate_patch_operations_rejects_system_paths() {
        let processor = ResponseProcessor::new();
        let operations = vec![
            json!({"op": "replace", "path": "/id", "value": "new-id"}),
        ];
        
        assert!(processor.validate_patch_operations(&operations).is_err());
    }

    #[test]
    fn test_validate_patch_operations_rejects_type_paths() {
        let processor = ResponseProcessor::new();
        let operations = vec![
            json!({"op": "replace", "path": "/type", "value": "new-type"}),
        ];
        
        assert!(processor.validate_patch_operations(&operations).is_err());
    }
}
