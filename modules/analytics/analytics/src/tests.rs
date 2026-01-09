/// Test utilities and mocks for analytics module
use modkit::api::{OpenApiRegistry, OperationSpec};
use std::sync::{Arc, Mutex};

/// Mock OpenAPI registry for testing
pub struct MockOpenApiRegistry {
    operations: Arc<Mutex<Vec<String>>>,
}

impl MockOpenApiRegistry {
    pub fn new() -> Self {
        Self {
            operations: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_registered_operations(&self) -> Vec<String> {
        self.operations.lock().unwrap().clone()
    }
}

impl Default for MockOpenApiRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenApiRegistry for MockOpenApiRegistry {
    fn register_operation(&self, spec: &OperationSpec) {
        if let Some(operation_id) = &spec.operation_id {
            self.operations.lock().unwrap().push(operation_id.clone());
        }
    }

    fn ensure_schema_raw(
        &self,
        _root_name: &str,
        _schemas: Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) -> String {
        // Return mock schema reference
        format!("#/components/schemas/{}", _root_name)
    }

    fn as_any(&self) -> &(dyn std::any::Any + 'static) {
        self
    }
}
