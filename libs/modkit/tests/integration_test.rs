#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Integration tests for the type-safe API operation builder
//!
//! These tests demonstrate correct usage patterns and verify that
//! the builder works as expected when used correctly.

use axum::{Json, Router, response::IntoResponse};
use modkit::api::{
    Missing, OpenApiRegistry, OperationBuilder, OperationSpec, ParamLocation,
    operation_builder::{AuthReqAction, AuthReqResource, LicenseFeature},
};
use serde_json::Value;
use std::sync::Mutex;

enum TestResource {
    Users,
}

impl AsRef<str> for TestResource {
    fn as_ref(&self) -> &'static str {
        match self {
            TestResource::Users => "users",
        }
    }
}

impl AuthReqResource for TestResource {}

enum Action {
    Read,
    Write,
}

impl AsRef<str> for Action {
    fn as_ref(&self) -> &'static str {
        match self {
            Action::Read => "read",
            Action::Write => "write",
        }
    }
}

impl AuthReqAction for Action {}

#[allow(dead_code)]
enum TestLicenseFeatures {
    Base,
}

impl AsRef<str> for TestLicenseFeatures {
    fn as_ref(&self) -> &'static str {
        match self {
            TestLicenseFeatures::Base => "gts.x.core.lic.feat.v1~x.core.global.base.v1",
        }
    }
}

impl LicenseFeature for TestLicenseFeatures {}

// Test registry that captures operations
#[derive(Default)]
struct TestRegistry {
    operations: Mutex<Vec<OperationSpec>>,
}

impl OpenApiRegistry for TestRegistry {
    fn register_operation(&self, spec: &OperationSpec) {
        self.operations.lock().unwrap().push(spec.clone());
    }

    fn ensure_schema_raw(
        &self,
        name: &str,
        _schemas: Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) -> String {
        // Test implementation - return the schema name
        name.to_owned()
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
    router = OperationBuilder::<Missing, Missing, ()>::get("/users-info/v1/users")
        .operation_id("users.list")
        .public()
        .summary("List all users")
        .description("Returns a paginated list of users in the system")
        .tag("users")
        .query_param("limit", false, "Maximum number of users to return")
        .query_param("offset", false, "Number of users to skip")
        .json_response(http::StatusCode::OK, "List of users")
        .json_response(
            http::StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error",
        )
        .handler(get_users_handler)
        .register(router, &registry);

    // Test POST endpoint
    router = OperationBuilder::<Missing, Missing, ()>::post("/users-info/v1/users")
        .operation_id("users.create")
        .summary("Create a new user")
        .require_auth(&TestResource::Users, &Action::Write)
        .require_license_features::<TestLicenseFeatures>([])
        .description("Creates a new user in the system")
        .tag("users")
        .json_response(http::StatusCode::CREATED, "User created successfully")
        .json_response(http::StatusCode::BAD_REQUEST, "Invalid user data")
        .json_response(
            http::StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error",
        )
        .handler(create_user_handler)
        .register(router, &registry);

    // Test GET endpoint with path parameter
    let _router = OperationBuilder::<Missing, Missing, ()>::get("/users-info/v1/users/{id}")
        .operation_id("users.get")
        .summary("Get user by ID")
        .require_auth(&TestResource::Users, &Action::Read)
        .require_license_features::<TestLicenseFeatures>([])
        .description("Retrieves a specific user by their unique identifier")
        .tag("users")
        .path_param("id", "User unique identifier")
        .json_response(http::StatusCode::OK, "User details")
        .json_response(http::StatusCode::NOT_FOUND, "User not found")
        .json_response(
            http::StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error",
        )
        .handler(get_user_handler)
        .register(router, &registry);

    // Verify all operations were registered
    let operations = registry.get_operations();
    assert_eq!(operations.len(), 3);

    // Verify GET /users-info/v1/users operation
    let list_op = &operations[0];
    assert_eq!(list_op.method, http::Method::GET);
    assert_eq!(list_op.path, "/users-info/v1/users");
    assert_eq!(list_op.operation_id, Some("users.list".to_owned()));
    assert_eq!(list_op.summary, Some("List all users".to_owned()));
    assert_eq!(list_op.tags, vec!["users"]);
    assert_eq!(list_op.params.len(), 2); // limit and offset
    assert_eq!(list_op.responses.len(), 2); // 200 and 500

    // Verify POST /users-info/v1/users operation
    let create_op = &operations[1];
    assert_eq!(create_op.method, http::Method::POST);
    assert_eq!(create_op.path, "/users-info/v1/users");
    assert_eq!(create_op.operation_id, Some("users.create".to_owned()));
    assert_eq!(create_op.responses.len(), 3); // 201, 400, 500

    // Verify GET /users-info/v1/users/{id} operation
    let get_op = &operations[2];
    assert_eq!(get_op.method, http::Method::GET);
    assert_eq!(get_op.path, "/users-info/v1/users/{id}");
    assert_eq!(get_op.operation_id, Some("users.get".to_owned()));
    assert_eq!(get_op.params.len(), 1); // id path param
    assert_eq!(get_op.responses.len(), 3); // 200, 404, 500
}

#[test]
fn test_builder_convenience_methods() {
    // Test all HTTP method convenience constructors
    let get_builder = OperationBuilder::<Missing, Missing, ()>::get("/tests/v1/test");
    assert_eq!(get_builder.spec().method, http::Method::GET);

    let post_builder = OperationBuilder::<Missing, Missing, ()>::post("/tests/v1/test");
    assert_eq!(post_builder.spec().method, http::Method::POST);

    let put_builder = OperationBuilder::<Missing, Missing, ()>::put("/tests/v1/test");
    assert_eq!(put_builder.spec().method, http::Method::PUT);

    let delete_builder = OperationBuilder::<Missing, Missing, ()>::delete("/tests/v1/test");
    assert_eq!(delete_builder.spec().method, http::Method::DELETE);

    let patch_builder = OperationBuilder::<Missing, Missing, ()>::patch("/tests/v1/test");
    assert_eq!(patch_builder.spec().method, http::Method::PATCH);
}

#[test]
fn test_builder_chaining_flexibility() {
    // Test that descriptive methods can be called in any order
    let builder1 = OperationBuilder::<Missing, Missing, ()>::get("/tests/v1/test")
        .summary("Test endpoint")
        .description("A test endpoint")
        .tag("test")
        .operation_id("test.endpoint");

    let builder2 = OperationBuilder::<Missing, Missing, ()>::get("/tests/v1/test")
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

    let _router = OperationBuilder::<Missing, Missing, ()>::get("/tests/v1/text")
        .require_auth(&TestResource::Users, &Action::Read)
        .require_license_features::<TestLicenseFeatures>([])
        .text_response(http::StatusCode::OK, "Plain text response", "text/plain")
        .html_response(http::StatusCode::OK, "HTML response")
        .json_response(http::StatusCode::INTERNAL_SERVER_ERROR, "Error response")
        .handler(|| async { "Hello" })
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
    let builder = OperationBuilder::<Missing, Missing, ()>::get("/tests/v1/test/{id}")
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
