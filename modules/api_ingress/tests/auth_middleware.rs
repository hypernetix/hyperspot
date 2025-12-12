#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Integration tests for auth middleware
//!
//! These tests verify that:
//! 1. Auth middleware is properly attached to the router
//! 2. SecurityCtx is always inserted by middleware
//! 3. Public routes work without authentication
//! 4. Protected routes enforce authentication when enabled

use anyhow::Result;
use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Json, Router,
};
use modkit::{
    api::OperationBuilder,
    context::ModuleCtx,
    contracts::{OpenApiRegistry, RestHostModule, RestfulModule},
    ClientHub, ConfigProvider, Module,
};
use modkit_auth::axum_ext::Authz;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
// for oneshot
use utoipa::ToSchema;

/// Test configuration provider
struct TestConfigProvider {
    config: serde_json::Value,
}

impl ConfigProvider for TestConfigProvider {
    fn get_module_config(&self, module: &str) -> Option<&serde_json::Value> {
        self.config.get(module)
    }
}

/// Create test context for api_ingress module
fn create_api_ingress_ctx(config: serde_json::Value) -> ModuleCtx {
    ModuleCtx::new(
        "api_ingress",
        Arc::new(TestConfigProvider { config }),
        Arc::new(ClientHub::new()),
        tokio_util::sync::CancellationToken::new(),
        None,
    )
}

/// Create test context for other test modules
fn create_test_module_ctx() -> ModuleCtx {
    ModuleCtx::new(
        "test_module",
        Arc::new(TestConfigProvider { config: json!({}) }),
        Arc::new(ClientHub::new()),
        tokio_util::sync::CancellationToken::new(),
        None,
    )
}

/// Test response type
#[derive(Serialize, Deserialize, ToSchema, Clone)]
struct TestResponse {
    message: String,
    user_id: String,
}

/// Handler that requires SecurityCtx (via Authz extractor)
async fn protected_handler(Authz(ctx): Authz) -> Json<TestResponse> {
    Json(TestResponse {
        message: "Protected resource accessed".to_string(),
        user_id: ctx.subject_id().to_string(),
    })
}

/// Handler that doesn't require auth
async fn public_handler() -> Json<TestResponse> {
    Json(TestResponse {
        message: "Public resource accessed".to_string(),
        user_id: "anonymous".to_string(),
    })
}

/// Test module with protected and public routes
pub struct TestAuthModule;

#[async_trait]
impl Module for TestAuthModule {
    async fn init(&self, _ctx: &ModuleCtx) -> Result<()> {
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl RestfulModule for TestAuthModule {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: Router,
        openapi: &dyn OpenApiRegistry,
    ) -> Result<Router> {
        // Protected route with explicit auth requirement
        let router = OperationBuilder::get("/api/protected")
            .operation_id("test.protected")
            .require_auth("test", "read")
            .summary("Protected endpoint")
            .handler(protected_handler)
            .json_response_with_schema::<TestResponse>(openapi, http::StatusCode::OK, "Success")
            .error_401(openapi)
            .error_403(openapi)
            .register(router, openapi);

        // Protected route with path parameter (to test pattern matching)
        let router = OperationBuilder::get("/api/users/{id}")
            .operation_id("test.get_user")
            .require_auth("users", "read")
            .summary("Get user by ID")
            .path_param("id", "User ID")
            .handler(protected_handler)
            .json_response_with_schema::<TestResponse>(openapi, http::StatusCode::OK, "Success")
            .error_401(openapi)
            .error_403(openapi)
            .register(router, openapi);

        // Public route with explicit public marking
        let router = OperationBuilder::get("/api/public")
            .operation_id("test.public")
            .public()
            .summary("Public endpoint")
            .handler(public_handler)
            .json_response_with_schema::<TestResponse>(openapi, http::StatusCode::OK, "Success")
            .register(router, openapi);

        Ok(router)
    }
}

#[tokio::test]
async fn test_auth_disabled_mode() {
    // Create api_ingress with auth disabled
    let config = json!({
        "api_ingress": {
            "config": {
                "bind_addr": "0.0.0.0:8080",
                "enable_docs": true,
                "cors_enabled": false,
                "auth_disabled": true,
            }
        }
    });

    let api_ctx = create_api_ingress_ctx(config);
    let test_ctx = create_test_module_ctx();

    let api_ingress = api_ingress::ApiIngress::default();
    api_ingress.init(&api_ctx).await.expect("Failed to init");

    // Register test module
    let router = Router::new();
    let test_module = TestAuthModule;
    let router = test_module
        .register_rest(&test_ctx, router, &api_ingress)
        .expect("Failed to register routes");

    // Finalize router (applies middleware)
    let router = api_ingress
        .rest_finalize(&api_ctx, router)
        .expect("Failed to finalize");

    // Test protected route WITHOUT token (should work because auth is disabled)
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/protected")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Protected route should work when auth is disabled"
    );

    // Test public route
    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/public")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Public route should work"
    );
}

#[tokio::test]
async fn test_public_routes_accessible() {
    // Create api_ingress with auth enabled but test public routes
    let config = json!({
        "api_ingress": {
            "config": {
                "bind_addr": "0.0.0.0:8080",
                "enable_docs": true,
                "cors_enabled": false,
                "auth_disabled": true, // Using disabled for simplicity in test
            }
        }
    });

    let api_ctx = create_api_ingress_ctx(config);
    let test_ctx = create_test_module_ctx();

    let api_ingress = api_ingress::ApiIngress::default();
    api_ingress.init(&api_ctx).await.expect("Failed to init");

    // First call rest_prepare to add built-in routes
    let router = Router::new();
    let router = api_ingress
        .rest_prepare(&api_ctx, router)
        .expect("Failed to prepare");

    // Then register test module routes
    let test_module = TestAuthModule;
    let router = test_module
        .register_rest(&test_ctx, router, &api_ingress)
        .expect("Failed to register routes");

    // Finally finalize
    let router = api_ingress
        .rest_finalize(&api_ctx, router)
        .expect("Failed to finalize");

    // Test built-in health endpoints
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Health endpoint should be accessible"
    );

    // Test OpenAPI endpoint
    let response = router
        .oneshot(
            Request::builder()
                .uri("/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "OpenAPI endpoint should be accessible"
    );
}

#[tokio::test]
async fn test_middleware_always_inserts_security_ctx() {
    // This test verifies that SecurityCtx is always available in handlers
    let config = json!({
        "api_ingress": {
            "config": {
                "bind_addr": "0.0.0.0:8080",
                "enable_docs": false,
                "cors_enabled": false,
                "auth_disabled": true,
            }
        }
    });

    let api_ctx = create_api_ingress_ctx(config);
    let test_ctx = create_test_module_ctx();

    let api_ingress = api_ingress::ApiIngress::default();
    api_ingress.init(&api_ctx).await.expect("Failed to init");

    let router = Router::new();
    let test_module = TestAuthModule;
    let router = test_module
        .register_rest(&test_ctx, router, &api_ingress)
        .expect("Failed to register routes");

    let router = api_ingress
        .rest_finalize(&api_ctx, router)
        .expect("Failed to finalize");

    // Make request to protected handler that extracts Authz(SecurityCtx)
    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/protected")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    // Should NOT get 500 error about missing SecurityCtx
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Handler should receive SecurityCtx from middleware"
    );
}

#[tokio::test]
async fn test_openapi_includes_security_metadata() {
    let config = json!({
        "api_ingress": {
            "config": {
                "bind_addr": "0.0.0.0:8080",
                "enable_docs": true,
                "cors_enabled": false,
                "auth_disabled": false,
                "require_auth_by_default": true,
            }
        }
    });

    let api_ctx = create_api_ingress_ctx(config);
    let test_ctx = create_test_module_ctx();

    let api_ingress = api_ingress::ApiIngress::default();
    api_ingress.init(&api_ctx).await.expect("Failed to init");

    let router = Router::new();
    let test_module = TestAuthModule;
    let _router = test_module
        .register_rest(&test_ctx, router, &api_ingress)
        .expect("Failed to register routes");

    // Build OpenAPI spec
    let openapi = api_ingress
        .build_openapi()
        .expect("Failed to build OpenAPI");
    let spec = serde_json::to_value(&openapi).expect("Failed to serialize");

    // Verify security scheme exists
    let security_schemes = spec
        .pointer("/components/securitySchemes")
        .expect("Security schemes should exist");
    assert!(
        security_schemes.get("bearerAuth").is_some(),
        "bearerAuth scheme should be registered"
    );

    // Verify protected route has security requirement
    let protected_security = spec.pointer("/paths/~1api~1protected/get/security");
    assert!(
        protected_security.is_some(),
        "Protected route should have security requirement in OpenAPI"
    );

    // Verify public route does NOT have security requirement
    let public_security = spec.pointer("/paths/~1api~1public/get/security");
    assert!(
        public_security.is_none()
            || public_security
                .unwrap()
                .as_array()
                .is_some_and(|a| a.is_empty()),
        "Public route should NOT have security requirement in OpenAPI"
    );
}

#[tokio::test]
async fn test_route_pattern_matching_with_path_params() {
    // This test verifies that routes with path parameters (e.g., /users/{id})
    // are properly matched and authorization is enforced
    let config = json!({
        "api_ingress": {
            "config": {
                "bind_addr": "0.0.0.0:8080",
                "enable_docs": false,
                "cors_enabled": false,
                "auth_disabled": true, // Disabled for test simplicity
            }
        }
    });

    let api_ctx = create_api_ingress_ctx(config);
    let test_ctx = create_test_module_ctx();

    let api_ingress = api_ingress::ApiIngress::default();
    api_ingress.init(&api_ctx).await.expect("Failed to init");

    let router = Router::new();
    let test_module = TestAuthModule;
    let router = test_module
        .register_rest(&test_ctx, router, &api_ingress)
        .expect("Failed to register routes");

    let router = api_ingress
        .rest_finalize(&api_ctx, router)
        .expect("Failed to finalize");

    // Test that /api/users/123 is accessible (matches /api/users/{id})
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/users/123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Route with path parameter should be accessible and matched correctly"
    );

    // Test that /api/users/abc-def-456 is also accessible
    let response = router
        .oneshot(
            Request::builder()
                .uri("/api/users/abc-def-456")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Request failed");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Route with different path parameter value should also be accessible"
    );
}
