#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Integration tests for the API Gateway router and new `OperationBuilder`
//!
//! This test demonstrates that the new type-safe `OperationBuilder` works
//! correctly with the API Gateway module for routing and `OpenAPI` generation.

use anyhow::Result;
use async_trait::async_trait;
use axum::{
    extract::{Json, Path},
    routing::get,
    Router,
};
use modkit::{
    config::ConfigProvider, contracts::OpenApiRegistry, Module, ModuleCtx, RestfulModule,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

/// Helper to create a test `ModuleCtx`
struct EmptyConfigProvider;

impl ConfigProvider for EmptyConfigProvider {
    fn get_module_config(&self, _module: &str) -> Option<&serde_json::Value> {
        None
    }
}

fn create_test_module_ctx() -> ModuleCtx {
    ModuleCtx::new(
        "test_module",
        Uuid::new_v4(),
        Arc::new(EmptyConfigProvider),
        Arc::new(modkit::ClientHub::new()),
        tokio_util::sync::CancellationToken::new(),
        None, // No database handle for these tests
    )
}

/// Test user structure
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[schema(title = "User")]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

/// Test request for creating users
#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[schema(title = "CreateUserRequest")]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

/// Test module that demonstrates the new `OperationBuilder` API
pub struct TestUsersModule;

#[async_trait]
impl Module for TestUsersModule {
    async fn init(&self, _ctx: &modkit::ModuleCtx) -> Result<()> {
        Ok(())
    }
}

impl RestfulModule for TestUsersModule {
    fn register_rest(
        &self,
        _ctx: &modkit::ModuleCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> Result<axum::Router> {
        use modkit::api::OperationBuilder;

        // Schemas will be auto-registered when used in operations

        // GET /users - List users
        let router = OperationBuilder::get("/tests/v1/users")
            .operation_id("users:list")
            .summary("List all users")
            .description("Retrieve a paginated list of users")
            .tag("Users")
            .query_param("limit", false, "Maximum number of users to return")
            .query_param("offset", false, "Number of users to skip")
            .public()
            .json_response_with_schema::<Vec<User>>(
                openapi,
                http::StatusCode::OK,
                "Users retrieved successfully",
            )
            .json_response(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            )
            .handler(get(list_users_handler))
            .register(router, openapi);

        // GET /users/{id} - Get user by ID
        let router = OperationBuilder::get("/tests/v1/users/{id}")
            .operation_id("users:get")
            .summary("Get user by ID")
            .description("Retrieve a specific user by their ID")
            .tag("Users")
            .path_param("id", "User ID")
            .public()
            .json_response_with_schema::<User>(openapi, http::StatusCode::OK, "User found")
            .json_response(http::StatusCode::NOT_FOUND, "User not found")
            .json_response(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            )
            .handler(get(get_user_handler))
            .register(router, openapi);

        // POST /users - Create user
        let router = OperationBuilder::post("/tests/v1/users")
            .operation_id("users:create")
            .summary("Create new user")
            .description("Create a new user with the provided data")
            .tag("Users")
            .json_request::<CreateUserRequest>(openapi, "User creation data")
            .public()
            .json_response_with_schema::<User>(
                openapi,
                http::StatusCode::CREATED,
                "User created successfully",
            )
            .json_response(http::StatusCode::BAD_REQUEST, "Invalid input data")
            .json_response(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            )
            .handler(axum::routing::post(create_user_handler))
            .register(router, openapi);

        Ok(router)
    }
}

// Handler functions for the test endpoints
async fn list_users_handler() -> Json<Vec<User>> {
    Json(vec![
        User {
            id: 1,
            name: "Alice Test".to_owned(),
            email: "alice@test.com".to_owned(),
        },
        User {
            id: 2,
            name: "Bob Test".to_owned(),
            email: "bob@test.com".to_owned(),
        },
    ])
}

async fn get_user_handler(Path(id): Path<u32>) -> Json<User> {
    Json(User {
        id,
        name: "Test User".to_owned(),
        email: "test@example.com".to_owned(),
    })
}

async fn create_user_handler(Json(req): Json<CreateUserRequest>) -> Json<User> {
    Json(User {
        id: 999,
        name: req.name,
        email: req.email,
    })
}

#[tokio::test]
async fn test_operation_builder_integration() {
    // Test that our new OperationBuilder works with the registry
    let registry = api_gateway::ApiGateway::default();
    let router = Router::new();

    let test_module = TestUsersModule;
    // Create a test ModuleCtx directly for testing
    let ctx = create_test_module_ctx();
    let _final_router = test_module
        .register_rest(&ctx, router, &registry)
        .expect("Failed to register routes");

    // Basic test that the router was created without errors
    // In a full integration test, we would start the server and make HTTP requests
}

#[tokio::test]
async fn test_schema_registration() {
    // Test that schemas are properly registered
    let registry = api_gateway::ApiGateway::default();
    let router = Router::new();

    let test_module = TestUsersModule;
    let ctx = create_test_module_ctx();
    let _final_router = test_module
        .register_rest(&ctx, router, &registry)
        .expect("Failed to register routes");

    // This test would verify that schemas were registered, but since the
    // schema registry is internal, we just verify no compilation errors
}
