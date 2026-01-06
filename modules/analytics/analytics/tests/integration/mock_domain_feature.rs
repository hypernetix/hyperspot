use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockGtsEntity {
    pub id: String,
    #[serde(rename = "type")]
    pub entity_type: String,
    pub entity: serde_json::Value,
    pub registered_at: String,
    pub tenant: String,
}

#[derive(Debug, Clone)]
pub enum MockBehavior {
    Success,
    NotFound,
    InternalError,
    Unavailable,
    ValidationError(String),
}

#[derive(Clone)]
pub struct MockDomainFeature {
    behavior: Arc<Mutex<MockBehavior>>,
    entities: Arc<Mutex<Vec<MockGtsEntity>>>,
    call_count: Arc<Mutex<usize>>,
}

impl MockDomainFeature {
    pub fn new() -> Self {
        Self {
            behavior: Arc::new(Mutex::new(MockBehavior::Success)),
            entities: Arc::new(Mutex::new(Vec::new())),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn set_behavior(&self, behavior: MockBehavior) {
        *self.behavior.lock().unwrap() = behavior;
    }

    pub fn add_entity(&self, entity: MockGtsEntity) {
        self.entities.lock().unwrap().push(entity);
    }

    pub fn clear_entities(&self) {
        self.entities.lock().unwrap().clear();
    }

    pub fn get_call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }

    pub fn reset_call_count(&self) {
        *self.call_count.lock().unwrap() = 0;
    }

    pub fn handle_create(&self, body: serde_json::Value) -> Response {
        *self.call_count.lock().unwrap() += 1;

        match *self.behavior.lock().unwrap() {
            MockBehavior::Success => {
                let entity = MockGtsEntity {
                    id: uuid::Uuid::new_v4().to_string(),
                    entity_type: "gts.test.type.v1~".to_string(),
                    entity: body,
                    registered_at: chrono::Utc::now().to_rfc3339(),
                    tenant: "test-tenant".to_string(),
                };
                self.add_entity(entity.clone());
                (StatusCode::CREATED, Json(entity)).into_response()
            }
            MockBehavior::NotFound => {
                (StatusCode::NOT_FOUND, "Entity not found").into_response()
            }
            MockBehavior::InternalError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
            }
            MockBehavior::Unavailable => {
                (StatusCode::SERVICE_UNAVAILABLE, "Service unavailable").into_response()
            }
            MockBehavior::ValidationError(ref msg) => {
                (StatusCode::BAD_REQUEST, msg.clone()).into_response()
            }
        }
    }

    pub fn handle_get(&self, id: &str) -> Response {
        *self.call_count.lock().unwrap() += 1;

        match *self.behavior.lock().unwrap() {
            MockBehavior::Success => {
                let entities = self.entities.lock().unwrap();
                if let Some(entity) = entities.iter().find(|e| e.id == id) {
                    (StatusCode::OK, Json(entity.clone())).into_response()
                } else {
                    (StatusCode::NOT_FOUND, "Entity not found").into_response()
                }
            }
            MockBehavior::NotFound => {
                (StatusCode::NOT_FOUND, "Entity not found").into_response()
            }
            MockBehavior::InternalError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
            }
            MockBehavior::Unavailable => {
                (StatusCode::SERVICE_UNAVAILABLE, "Service unavailable").into_response()
            }
            MockBehavior::ValidationError(ref msg) => {
                (StatusCode::BAD_REQUEST, msg.clone()).into_response()
            }
        }
    }

    pub fn handle_list(&self) -> Response {
        *self.call_count.lock().unwrap() += 1;

        match *self.behavior.lock().unwrap() {
            MockBehavior::Success => {
                let entities = self.entities.lock().unwrap();
                (StatusCode::OK, Json(entities.clone())).into_response()
            }
            MockBehavior::NotFound => {
                (StatusCode::OK, Json(Vec::<MockGtsEntity>::new())).into_response()
            }
            MockBehavior::InternalError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
            }
            MockBehavior::Unavailable => {
                (StatusCode::SERVICE_UNAVAILABLE, "Service unavailable").into_response()
            }
            MockBehavior::ValidationError(ref msg) => {
                (StatusCode::BAD_REQUEST, msg.clone()).into_response()
            }
        }
    }

    pub fn handle_update(&self, id: &str, body: serde_json::Value) -> Response {
        *self.call_count.lock().unwrap() += 1;

        match *self.behavior.lock().unwrap() {
            MockBehavior::Success => {
                let mut entities = self.entities.lock().unwrap();
                if let Some(entity) = entities.iter_mut().find(|e| e.id == id) {
                    entity.entity = body;
                    (StatusCode::OK, Json(entity.clone())).into_response()
                } else {
                    (StatusCode::NOT_FOUND, "Entity not found").into_response()
                }
            }
            MockBehavior::NotFound => {
                (StatusCode::NOT_FOUND, "Entity not found").into_response()
            }
            MockBehavior::InternalError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
            }
            MockBehavior::Unavailable => {
                (StatusCode::SERVICE_UNAVAILABLE, "Service unavailable").into_response()
            }
            MockBehavior::ValidationError(ref msg) => {
                (StatusCode::BAD_REQUEST, msg.clone()).into_response()
            }
        }
    }

    pub fn handle_delete(&self, id: &str) -> Response {
        *self.call_count.lock().unwrap() += 1;

        match *self.behavior.lock().unwrap() {
            MockBehavior::Success => {
                let mut entities = self.entities.lock().unwrap();
                if let Some(pos) = entities.iter().position(|e| e.id == id) {
                    entities.remove(pos);
                    StatusCode::NO_CONTENT.into_response()
                } else {
                    (StatusCode::NOT_FOUND, "Entity not found").into_response()
                }
            }
            MockBehavior::NotFound => {
                (StatusCode::NOT_FOUND, "Entity not found").into_response()
            }
            MockBehavior::InternalError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
            }
            MockBehavior::Unavailable => {
                (StatusCode::SERVICE_UNAVAILABLE, "Service unavailable").into_response()
            }
            MockBehavior::ValidationError(ref msg) => {
                (StatusCode::BAD_REQUEST, msg.clone()).into_response()
            }
        }
    }

    pub fn validate_security_ctx(&self, expected_tenant: &str) -> bool {
        true
    }
}

impl Default for MockDomainFeature {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_success_behavior() {
        let mock = MockDomainFeature::new();
        mock.set_behavior(MockBehavior::Success);

        let response = mock.handle_create(serde_json::json!({"name": "test"}));
        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(mock.get_call_count(), 1);
    }

    #[test]
    fn test_mock_not_found_behavior() {
        let mock = MockDomainFeature::new();
        mock.set_behavior(MockBehavior::NotFound);

        let response = mock.handle_get("non-existent-id");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_mock_unavailable_behavior() {
        let mock = MockDomainFeature::new();
        mock.set_behavior(MockBehavior::Unavailable);

        let response = mock.handle_list();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_mock_validation_error_behavior() {
        let mock = MockDomainFeature::new();
        mock.set_behavior(MockBehavior::ValidationError("Invalid input".to_string()));

        let response = mock.handle_create(serde_json::json!({}));
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_mock_call_count_tracking() {
        let mock = MockDomainFeature::new();
        
        mock.handle_create(serde_json::json!({"name": "test1"}));
        mock.handle_list();
        mock.handle_get("test-id");
        
        assert_eq!(mock.get_call_count(), 3);
        
        mock.reset_call_count();
        assert_eq!(mock.get_call_count(), 0);
    }

    #[test]
    fn test_mock_entity_storage() {
        let mock = MockDomainFeature::new();
        
        let entity = MockGtsEntity {
            id: "test-id".to_string(),
            entity_type: "gts.test.type.v1~".to_string(),
            entity: serde_json::json!({"name": "test"}),
            registered_at: "2024-01-01T00:00:00Z".to_string(),
            tenant: "test-tenant".to_string(),
        };
        
        mock.add_entity(entity);
        
        let response = mock.handle_get("test-id");
        assert_eq!(response.status(), StatusCode::OK);
        
        mock.clear_entities();
        let response = mock.handle_get("test-id");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
