// @fdd-change:fdd-analytics-feature-gts-core-change-response-processing:ph-1
// @fdd-req:fdd-analytics-feature-gts-core-req-tolerant-reader:ph-1
use serde_json::{Map, Value};
use std::collections::HashSet;

// @fdd-change:fdd-analytics-feature-gts-core-change-response-processing:ph-1
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldCategory {
    ClientProvided,
    ServerManaged,
    Computed,
    Secret,
}

pub struct FieldHandler {
    server_managed_fields: HashSet<String>,
    secret_fields: HashSet<String>,
    computed_fields: HashSet<String>,
}

impl FieldHandler {
    pub fn new() -> Self {
        let mut server_managed = HashSet::new();
        server_managed.insert("id".to_string());
        server_managed.insert("type".to_string());
        server_managed.insert("registered_at".to_string());
        server_managed.insert("updated_at".to_string());
        server_managed.insert("deleted_at".to_string());
        server_managed.insert("tenant".to_string());
        server_managed.insert("registered_by".to_string());
        server_managed.insert("updated_by".to_string());
        server_managed.insert("deleted_by".to_string());

        let mut secrets = HashSet::new();
        secrets.insert("entity/api_key".to_string());
        secrets.insert("entity/credentials".to_string());
        secrets.insert("entity/password".to_string());
        secrets.insert("entity/secret".to_string());
        secrets.insert("entity/token".to_string());

        let mut computed = HashSet::new();
        computed.insert("asset_path".to_string());

        Self {
            server_managed_fields: server_managed,
            secret_fields: secrets,
            computed_fields: computed,
        }
    }

    pub fn categorize_field(&self, field_path: &str) -> FieldCategory {
        if self.server_managed_fields.contains(field_path) {
            return FieldCategory::ServerManaged;
        }

        if self.secret_fields.contains(field_path) {
            return FieldCategory::Secret;
        }

        if self.computed_fields.contains(field_path) {
            return FieldCategory::Computed;
        }

        FieldCategory::ClientProvided
    }

    pub fn filter_request(&self, mut value: Value) -> Value {
        if let Some(obj) = value.as_object_mut() {
            for field in &self.server_managed_fields {
                obj.remove(field);
            }
        }
        value
    }

    pub fn filter_response(&self, mut value: Value) -> Value {
        if let Some(obj) = value.as_object_mut() {
            if let Some(entity) = obj.get_mut("entity") {
                if let Some(entity_obj) = entity.as_object_mut() {
                    let keys_to_remove: Vec<String> = entity_obj
                        .keys()
                        .filter(|k| {
                            let path = format!("entity/{}", k);
                            self.secret_fields.contains(&path)
                        })
                        .cloned()
                        .collect();

                    for key in keys_to_remove {
                        entity_obj.remove(&key);
                    }
                }
            }
        }
        value
    }

    pub fn inject_computed_fields(&self, mut value: Value, id: &str, _type_id: &str) -> Value {
        if let Some(obj) = value.as_object_mut() {
            let asset_path = format!("/api/analytics/v1/gts/{}", id);
            obj.insert("asset_path".to_string(), Value::String(asset_path));
        }
        value
    }

    pub fn validate_patch_path(&self, path: &str) -> Result<(), String> {
        if path.starts_with("/entity/") {
            Ok(())
        } else {
            Err(format!(
                "JSON Patch path '{}' is not allowed. Only /entity/* paths are permitted.",
                path
            ))
        }
    }

    pub fn apply_field_projection(&self, mut value: Value, select_fields: &[String]) -> Value {
        if select_fields.is_empty() {
            return value;
        }

        if let Some(obj) = value.as_object_mut() {
            let mut projected = Map::new();

            for field in select_fields {
                if let Some(entity_field) = field.strip_prefix("entity/") {
                    if let Some(entity) = obj.get("entity") {
                        if let Some(entity_obj) = entity.as_object() {
                            if let Some(field_value) = entity_obj.get(entity_field) {
                                let entry = projected
                                    .entry("entity")
                                    .or_insert_with(|| Value::Object(Map::new()));

                                if let Value::Object(entity_map) = entry {
                                    entity_map
                                        .insert(entity_field.to_string(), field_value.clone());
                                }
                            }
                        }
                    }
                } else if let Some(field_value) = obj.get(field.as_str()) {
                    projected.insert(field.clone(), field_value.clone());
                }
            }

            return Value::Object(projected);
        }

        value
    }
}

impl Default for FieldHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // @fdd-test:fdd-analytics-feature-gts-core-test-client-cannot-override-fields:ph-1
    #[test]
    fn test_categorize_server_managed_field() {
        let handler = FieldHandler::new();
        assert_eq!(handler.categorize_field("id"), FieldCategory::ServerManaged);
        assert_eq!(
            handler.categorize_field("type"),
            FieldCategory::ServerManaged
        );
        assert_eq!(
            handler.categorize_field("tenant"),
            FieldCategory::ServerManaged
        );
    }

    #[test]
    fn test_categorize_secret_field() {
        let handler = FieldHandler::new();
        assert_eq!(
            handler.categorize_field("entity/api_key"),
            FieldCategory::Secret
        );
        assert_eq!(
            handler.categorize_field("entity/credentials"),
            FieldCategory::Secret
        );
    }

    #[test]
    fn test_categorize_computed_field() {
        let handler = FieldHandler::new();
        assert_eq!(
            handler.categorize_field("asset_path"),
            FieldCategory::Computed
        );
    }

    #[test]
    fn test_categorize_client_provided_field() {
        let handler = FieldHandler::new();
        assert_eq!(
            handler.categorize_field("entity/name"),
            FieldCategory::ClientProvided
        );
        assert_eq!(
            handler.categorize_field("entity/description"),
            FieldCategory::ClientProvided
        );
    }

    #[test]
    fn test_filter_request_removes_system_fields() {
        let handler = FieldHandler::new();

        // fdd-begin fdd-analytics-feature-gts-core-test-client-cannot-override-fields:ph-1:inst-send-request-with-readonly-fields
        let input = json!({
            "id": "custom-id",
            "type": "custom-type",
            "tenant": "custom-tenant",
            "entity": {
                "name": "Test"
            }
        });
        // fdd-end fdd-analytics-feature-gts-core-test-client-cannot-override-fields:ph-1:inst-send-request-with-readonly-fields

        let filtered = handler.filter_request(input);

        // fdd-begin fdd-analytics-feature-gts-core-test-client-cannot-override-fields:ph-1:inst-verify-readonly-fields-not-applied
        assert!(filtered.get("id").is_none());
        assert!(filtered.get("type").is_none());
        assert!(filtered.get("tenant").is_none());
        assert!(filtered.get("entity").is_some());

        // fdd-end fdd-analytics-feature-gts-core-test-client-cannot-override-fields:ph-1:inst-verify-readonly-fields-not-applied
    }

    #[test]
    fn test_filter_response_removes_secrets() {
        let handler = FieldHandler::new();
        let input = json!({
            "id": "test-id",
            "entity": {
                "name": "Test",
                "api_key": "secret123",
                "credentials": "secret456"
            }
        });

        let filtered = handler.filter_response(input);
        let entity = filtered.get("entity").unwrap().as_object().unwrap();
        assert!(entity.get("name").is_some());
        assert!(entity.get("api_key").is_none());
        assert!(entity.get("credentials").is_none());
    }

    #[test]
    fn test_inject_computed_fields() {
        let handler = FieldHandler::new();
        let input = json!({
            "id": "test-id",
            "entity": {"name": "Test"}
        });

        let result = handler.inject_computed_fields(input, "test-id", "test-type");
        assert_eq!(
            result.get("asset_path").unwrap().as_str().unwrap(),
            "/api/analytics/v1/gts/test-id"
        );
    }

    #[test]
    fn test_validate_patch_path_allows_entity_paths() {
        let handler = FieldHandler::new();
        assert!(handler.validate_patch_path("/entity/name").is_ok());
        assert!(handler.validate_patch_path("/entity/description").is_ok());
    }

    #[test]
    fn test_validate_patch_path_rejects_non_entity_paths() {
        let handler = FieldHandler::new();
        assert!(handler.validate_patch_path("/id").is_err());
        assert!(handler.validate_patch_path("/type").is_err());
        assert!(handler.validate_patch_path("/tenant").is_err());
    }

    #[test]
    fn test_apply_field_projection() {
        let handler = FieldHandler::new();
        let input = json!({
            "id": "test-id",
            "type": "test-type",
            "entity": {
                "name": "Test",
                "description": "Desc",
                "value": 123
            }
        });

        let select = vec!["id".to_string(), "entity/name".to_string()];
        let result = handler.apply_field_projection(input, &select);

        assert!(result.get("id").is_some());
        assert!(result.get("type").is_none());
        let entity = result.get("entity").unwrap().as_object().unwrap();
        assert!(entity.get("name").is_some());
        assert!(entity.get("description").is_none());
        assert!(entity.get("value").is_none());
    }
}
