//! Integration tests for the type-safe API operation builder
//!
//! These tests demonstrate correct usage patterns and verify that
//! the builder works as expected when used correctly.

use axum::{response::IntoResponse, Json, Router};
use modkit::api::{Missing, OpenApiRegistry, OperationBuilder, OperationSpec, ParamLocation};
use serde_json::Value;
use std::sync::Mutex;

// Test registry that captures operations
#[derive(Default)]
struct TestRegistry {
    operations: Mutex<Vec<OperationSpec>>,
}

impl OpenApiRegistry for TestRegistry {
    fn register_operation(&self, spec: &OperationSpec) {
        self.operations.lock().unwrap().push(spec.clone());
    }

    fn ensure_schema_raw(&self, name: &str, _schemas: Vec<(String, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>)>) -> String {
        // Test implementation - return the schema name
        name.to_string()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}


impl TestRegistry {
    fn get_operations(&self) -> Vec<OperationSpec> {
        self.operations.lock().unwrap().clone()
    }
}

// Test handlers
async fn get_users_handler() -> Json<Value> {
    Json(serde_json::json!({"users": []}))
}

async fn create_user_handler() -> impl IntoResponse {
    Json(serde_json::json!({"id": 1, "name": "Test User"}))
}

async fn get_user_handler() -> Json<Value> {
    Json(serde_json::json!({"id": 1, "name": "Test User"}))
}

#[tokio::test]
async fn test_complete_api_builder_flow() {
    let registry = TestRegistry::default();
    let mut router = Router::new();

    // Test GET endpoint with all features
    router = OperationBuilder::<Missing, Missing, ()>::get("/users")
        .operation_id("users.list")
        .summary("List all users")
        .description("Returns a paginated list of users in the system")
        .tag("users")
        .query_param("limit", false, "Maximum number of users to return")
        .query_param("offset", false, "Number of users to skip")
        .json_response(200, "List of users")
        .json_response(500, "Internal server error")
        .handler(get_users_handler)
        .register(router, &registry);

    // Test POST endpoint
    router = OperationBuilder::<Missing, Missing, ()>::post("/users")
        .operation_id("users.create")
        .summary("Create a new user")
        .description("Creates a new user in the system")
        .tag("users")
        .json_response(201, "User created successfully")
        .json_response(400, "Invalid user data")
        .json_response(500, "Internal server error")
        .handler(create_user_handler)
        .register(router, &registry);

    // Test GET endpoint with path parameter
    let _router = OperationBuilder::<Missing, Missing, ()>::get("/users/{id}")
        .operation_id("users.get")
        .summary("Get user by ID")
        .description("Retrieves a specific user by their unique identifier")
        .tag("users")
        .path_param("id", "User unique identifier")
        .json_response(200, "User details")
        .json_response(404, "User not found")
        .json_response(500, "Internal server error")
        .handler(get_user_handler)
        .register(router, &registry);

    // Verify all operations were registered
    let operations = registry.get_operations();
    assert_eq!(operations.len(), 3);

    // Verify GET /users operation
    let list_op = &operations[0];
    assert_eq!(list_op.method, http::Method::GET);
    assert_eq!(list_op.path, "/users");
    assert_eq!(list_op.operation_id, Some("users.list".to_string()));
    assert_eq!(list_op.summary, Some("List all users".to_string()));
    assert_eq!(list_op.tags, vec!["users"]);
    assert_eq!(list_op.params.len(), 2); // limit and offset
    assert_eq!(list_op.responses.len(), 2); // 200 and 500

    // Verify POST /users operation
    let create_op = &operations[1];
    assert_eq!(create_op.method, http::Method::POST);
    assert_eq!(create_op.path, "/users");
    assert_eq!(create_op.operation_id, Some("users.create".to_string()));
    assert_eq!(create_op.responses.len(), 3); // 201, 400, 500

    // Verify GET /users/{id} operation
    let get_op = &operations[2];
    assert_eq!(get_op.method, http::Method::GET);
    assert_eq!(get_op.path, "/users/{id}");
    assert_eq!(get_op.operation_id, Some("users.get".to_string()));
    assert_eq!(get_op.params.len(), 1); // id path param
    assert_eq!(get_op.responses.len(), 3); // 200, 404, 500
}

#[test]
fn test_builder_convenience_methods() {
    // Test all HTTP method convenience constructors
    let get_builder = OperationBuilder::<Missing, Missing, ()>::get("/test");
    assert_eq!(get_builder.spec().method, http::Method::GET);

    let post_builder = OperationBuilder::<Missing, Missing, ()>::post("/test");
    assert_eq!(post_builder.spec().method, http::Method::POST);

    let put_builder = OperationBuilder::<Missing, Missing, ()>::put("/test");
    assert_eq!(put_builder.spec().method, http::Method::PUT);

    let delete_builder = OperationBuilder::<Missing, Missing, ()>::delete("/test");
    assert_eq!(delete_builder.spec().method, http::Method::DELETE);

    let patch_builder = OperationBuilder::<Missing, Missing, ()>::patch("/test");
    assert_eq!(patch_builder.spec().method, http::Method::PATCH);
}

#[test]
fn test_builder_chaining_flexibility() {
    // Test that descriptive methods can be called in any order
    let builder1 = OperationBuilder::<Missing, Missing, ()>::get("/test")
        .summary("Test endpoint")
        .description("A test endpoint")
        .tag("test")
        .operation_id("test.endpoint");

    let builder2 = OperationBuilder::<Missing, Missing, ()>::get("/test")
        .operation_id("test.endpoint")
        .tag("test")
        .description("A test endpoint")
        .summary("Test endpoint");

    // Both should have the same final spec (regardless of order)
    assert_eq!(builder1.spec().summary, builder2.spec().summary);
    assert_eq!(builder1.spec().description, builder2.spec().description);
    assert_eq!(builder1.spec().tags, builder2.spec().tags);
    assert_eq!(builder1.spec().operation_id, builder2.spec().operation_id);
}

#[test]
fn test_response_types() {
    let registry = TestRegistry::default();
    let router = Router::new();

    async fn text_handler() -> &'static str {
        "Hello"
    }

    let _router = OperationBuilder::<Missing, Missing, ()>::get("/text")
        .text_response(200, "Plain text response")
        .html_response(200, "HTML response")
        .json_response(500, "Error response")
        .handler(text_handler)
        .register(router, &registry);

    let operations = registry.get_operations();
    assert_eq!(operations.len(), 1);

    let op = &operations[0];
    assert_eq!(op.responses.len(), 3);

    // Check different content types
    let content_types: Vec<_> = op.responses.iter().map(|r| r.content_type).collect();
    assert!(content_types.contains(&"text/plain"));
    assert!(content_types.contains(&"text/html"));
    assert!(content_types.contains(&"application/json"));
}

#[test]
fn test_parameter_types() {
    let builder = OperationBuilder::<Missing, Missing, ()>::get("/test/{id}")
        .path_param("id", "Resource identifier")
        .query_param("limit", false, "Result limit")
        .query_param("required_param", true, "Required parameter");

    assert_eq!(builder.spec().params.len(), 3);

    let id_param = &builder.spec().params[0];
    assert_eq!(id_param.name, "id");
    assert_eq!(id_param.location, ParamLocation::Path);
    assert!(id_param.required);

    let limit_param = &builder.spec().params[1];
    assert_eq!(limit_param.name, "limit");
    assert_eq!(limit_param.location, ParamLocation::Query);
    assert!(!limit_param.required);

    let required_param = &builder.spec().params[2];
    assert_eq!(required_param.name, "required_param");
    assert_eq!(required_param.location, ParamLocation::Query);
    assert!(required_param.required);
}

