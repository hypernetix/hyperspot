#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Test that `auth_disabled` mode properly injects root context

use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use modkit_security::SecurityCtx;
use tower::ServiceExt;

/// Test handler that extracts `SecurityCtx` and returns its properties as JSON
async fn test_handler(Extension(ctx): Extension<SecurityCtx>) -> impl IntoResponse {
    let is_root = ctx.scope().is_root();
    let is_empty = ctx.scope().is_empty();
    let tenant_count = ctx.scope().tenant_ids().len();

    axum::Json(serde_json::json!({
        "is_root": is_root,
        "is_empty": is_empty,
        "tenant_count": tenant_count,
        "is_denied": ctx.is_denied()
    }))
}

/// Middleware that simulates `auth_disabled` mode by injecting root context
async fn inject_root_context(mut req: Request, next: Next) -> Response {
    #[allow(deprecated)]
    req.extensions_mut().insert(SecurityCtx::root_ctx());
    next.run(req).await
}

#[tokio::test]
async fn test_auth_disabled_injects_root_context() {
    // Build a router with the auth-disabled middleware
    let app = Router::new()
        .route("/test", get(test_handler))
        .layer(middleware::from_fn(inject_root_context));

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

    // Verify root context properties
    assert_eq!(json["is_root"], true, "SecurityCtx should have root scope");
    assert_eq!(json["is_empty"], false, "Root scope should not be empty");
    assert_eq!(
        json["tenant_count"], 0,
        "Root scope has no explicit tenant IDs"
    );
    assert_eq!(
        json["is_denied"], false,
        "Root context should not be denied"
    );
}

#[tokio::test]
async fn test_auth_disabled_allows_access_to_all_tenants() {
    // Handler that verifies root access bypasses tenant filtering
    async fn check_root_access(Extension(ctx): Extension<SecurityCtx>) -> impl IntoResponse {
        // In disabled mode, we should have root access
        assert!(
            ctx.scope().is_root(),
            "Should have root scope in disabled mode"
        );
        assert!(!ctx.is_denied(), "Root context should not be denied");

        // Root scope bypasses tenant filtering - it's marked as root, not as having all tenant IDs
        assert!(
            ctx.scope().tenant_ids().is_empty(),
            "Root scope uses flag, not tenant list"
        );

        axum::Json(serde_json::json!({"access": "granted", "mode": "root"}))
    }

    let app = Router::new()
        .route("/check", get(check_root_access))
        .layer(middleware::from_fn(inject_root_context));

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
    assert_eq!(json["mode"], "root");
}

#[tokio::test]
async fn test_auth_disabled_vs_normal_scope() {
    // Handler that distinguishes between root and normal scopes
    async fn scope_info(Extension(ctx): Extension<SecurityCtx>) -> impl IntoResponse {
        axum::Json(serde_json::json!({
            "is_root": ctx.scope().is_root(),
            "is_empty": ctx.scope().is_empty(),
            "has_tenants": ctx.scope().has_tenants(),
        }))
    }

    // Test 1: Root context (auth disabled)
    let app_root = Router::new()
        .route("/info", get(scope_info))
        .layer(middleware::from_fn(inject_root_context));

    let response = app_root
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

    assert_eq!(json["is_root"], true);
    assert_eq!(json["is_empty"], false);
    assert_eq!(json["has_tenants"], false); // Root doesn't use tenant list
}
