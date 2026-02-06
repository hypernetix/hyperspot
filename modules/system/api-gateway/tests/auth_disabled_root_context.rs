#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Test that `auth_disabled` mode properly injects default tenant context
use axum::{
    Extension, Router,
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use modkit_security::{NoopPolicyEngine, SecurityContext};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::{Uuid, uuid};

/// Test tenant ID for auth-disabled mode tests
const TEST_DEFAULT_TENANT_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000001");
/// Test subject ID for auth-disabled mode (matches `api_gateway` constant)
const TEST_DEFAULT_SUBJECT_ID: Uuid = uuid!("11111111-0000-0000-0000-000000000001");

/// Test handler that extracts `SecurityContext` and returns its properties as JSON
async fn test_handler(Extension(ctx): Extension<SecurityContext>) -> impl IntoResponse {
    let scope = ctx
        .scope(Arc::new(NoopPolicyEngine))
        .prepare()
        .await
        .unwrap();

    let is_empty = scope.is_empty();
    let tenant_ids = scope.tenant_ids().to_vec();
    let tenant_count = tenant_ids.len();

    axum::Json(serde_json::json!({
        "is_empty": is_empty,
        "tenant_count": tenant_count,
        "tenant_ids": tenant_ids,
        "subject_id": ctx.subject_id()
    }))
}

/// Middleware that simulates `auth_disabled` mode by injecting default tenant context
async fn inject_default_tenant_context(mut req: Request, next: Next) -> Response {
    // This simulates what api_gateway does in auth_disabled mode:
    let ctx = SecurityContext::builder()
        .tenant_id(TEST_DEFAULT_TENANT_ID)
        .subject_id(TEST_DEFAULT_SUBJECT_ID)
        .build();

    req.extensions_mut().insert(ctx);
    next.run(req).await
}

#[tokio::test]
async fn test_auth_disabled_injects_default_tenant_context() {
    // Build a router with the auth-disabled middleware
    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(middleware::from_fn(inject_default_tenant_context));

    // Make a request
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Verify response
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify default tenant context properties
    assert_eq!(
        json["is_empty"], false,
        "Default tenant scope should not be empty"
    );
    assert_eq!(
        json["tenant_count"], 1,
        "Should have exactly one tenant (the default)"
    );
    assert_eq!(
        json["subject_id"],
        TEST_DEFAULT_SUBJECT_ID.to_string(),
        "Subject should be TEST_DEFAULT_SUBJECT_ID"
    );
}

#[tokio::test]
async fn test_auth_disabled_scoped_to_default_tenant() {
    // Handler that verifies the context is scoped to the default tenant
    async fn check_tenant_access(Extension(ctx): Extension<SecurityContext>) -> impl IntoResponse {
        let scope = ctx
            .scope(Arc::new(NoopPolicyEngine))
            .prepare()
            .await
            .unwrap();

        // In disabled mode, we should have access scoped to the default tenant
        assert!(
            !scope.is_empty(),
            "Should have non-empty scope in disabled mode"
        );

        // Should have exactly the default tenant
        let tenant_ids = scope.tenant_ids();
        assert_eq!(tenant_ids.len(), 1, "Should have exactly one tenant");
        assert_eq!(
            tenant_ids[0], TEST_DEFAULT_TENANT_ID,
            "Should be the default tenant"
        );

        axum::Json(serde_json::json!({
            "access": "granted",
            "mode": "default_tenant",
            "tenant_id": tenant_ids[0]
        }))
    }

    let app = Router::new()
        .route("/check", get(check_tenant_access))
        .layer(middleware::from_fn(inject_default_tenant_context));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/check")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["access"], "granted");
    assert_eq!(json["mode"], "default_tenant");
    assert_eq!(json["tenant_id"], TEST_DEFAULT_TENANT_ID.to_string());
}

#[tokio::test]
async fn test_auth_disabled_uses_default_subject() {
    // Handler that verifies the default subject ID is used
    async fn check_subject(Extension(ctx): Extension<SecurityContext>) -> impl IntoResponse {
        axum::Json(serde_json::json!({
            "subject_id": ctx.subject_id(),
            "is_default_subject": ctx.subject_id() == TEST_DEFAULT_SUBJECT_ID,
        }))
    }

    let app = Router::new()
        .route("/subject", get(check_subject))
        .layer(middleware::from_fn(inject_default_tenant_context));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/subject")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(
        json["is_default_subject"], true,
        "Auth-disabled mode should use TEST_DEFAULT_SUBJECT_ID"
    );
}

#[tokio::test]
async fn test_default_tenant_vs_normal_scope() {
    // Handler that reports scope info
    async fn scope_info(Extension(ctx): Extension<SecurityContext>) -> impl IntoResponse {
        let scope = ctx
            .scope(Arc::new(NoopPolicyEngine))
            .prepare()
            .await
            .unwrap();

        axum::Json(serde_json::json!({
            "is_empty": scope.is_empty(),
            "has_tenants": scope.has_tenants(),
            "tenant_count": scope.tenant_ids().len(),
        }))
    }

    // Test: Default tenant context (auth disabled)
    let app = Router::new()
        .route("/info", get(scope_info))
        .layer(middleware::from_fn(inject_default_tenant_context));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/info")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Default tenant context has explicit tenant, not a bypass
    assert_eq!(json["is_empty"], false);
    assert_eq!(json["has_tenants"], true);
    assert_eq!(json["tenant_count"], 1);
}
