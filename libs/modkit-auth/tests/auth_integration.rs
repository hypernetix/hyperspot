#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Integration tests for the unified authentication system
//!
//! These tests verify end-to-end behavior with a real Axum Router

use axum::{
    Router,
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::get,
};
use modkit_auth::{
    Claims,
    axum_ext::AuthPolicyLayer,
    errors::AuthError,
    traits::{PrimaryAuthorizer, TokenValidator},
    types::{AuthRequirement, RoutePolicy, SecRequirement},
};
use modkit_security::SecurityContext;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tower::ServiceExt;
use uuid::Uuid;

/// Fake `TokenValidator` for integration testing
struct IntegrationValidator {
    should_succeed: AtomicBool,
    claims: Claims,
}

impl IntegrationValidator {
    fn new_ok(claims: Claims) -> Self {
        Self {
            should_succeed: AtomicBool::new(true),
            claims,
        }
    }

    fn new_err() -> Self {
        Self {
            should_succeed: AtomicBool::new(false),
            claims: fake_claims(Uuid::new_v4()),
        }
    }
}

#[async_trait::async_trait]
impl TokenValidator for IntegrationValidator {
    async fn validate_and_parse(&self, _token: &str) -> Result<Claims, AuthError> {
        if self.should_succeed.load(Ordering::SeqCst) {
            Ok(self.claims.clone())
        } else {
            Err(AuthError::Unauthenticated)
        }
    }
}

/// Fake Authorizer for integration testing
struct IntegrationAuthorizer {
    should_succeed: AtomicBool,
}

impl IntegrationAuthorizer {
    fn new_ok() -> Self {
        Self {
            should_succeed: AtomicBool::new(true),
        }
    }

    fn new_err() -> Self {
        Self {
            should_succeed: AtomicBool::new(false),
        }
    }
}

#[async_trait::async_trait]
impl PrimaryAuthorizer for IntegrationAuthorizer {
    async fn check(
        &self,
        _claims: &Claims,
        _requirement: &SecRequirement,
    ) -> Result<(), AuthError> {
        if self.should_succeed.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(AuthError::Forbidden)
        }
    }
}

/// Helper to create fake Claims
fn fake_claims(sub_id: Uuid) -> Claims {
    Claims {
        issuer: "test-issuer".to_owned(),
        subject: sub_id,
        audiences: vec!["test-api".to_owned()],
        expires_at: None,
        not_before: None,
        issued_at: None,
        jwt_id: None,
        tenant_id: Uuid::new_v4(),
        permissions: vec![],
        extras: serde_json::Map::new(),
    }
}

/// Handler that returns `SecurityContext` information for anonymous routes
async fn test_handler(ctx: axum::Extension<SecurityContext>) -> impl IntoResponse {
    let ctx = ctx.0;
    // Check if it's an anonymous context by checking for default subject_id
    if ctx.subject_id() == Uuid::default() {
        format!("anonymous:{}", ctx.subject_id())
    } else {
        format!("user:{}", ctx.subject_id())
    }
}

/// Build a test router with auth middleware
fn build_test_router(
    policy: Arc<dyn RoutePolicy>,
    validator: Arc<dyn TokenValidator>,
    authorizer: Arc<dyn PrimaryAuthorizer>,
) -> Router {
    Router::new()
        .route("/secured", get(test_handler))
        .route("/public", get(test_handler))
        .route("/optional", get(test_handler))
        .layer(AuthPolicyLayer::new(validator, authorizer, policy))
}

/// Helper to build a request
fn build_request(method: Method, path: &str, token: Option<&str>) -> Request {
    let mut builder = axum::http::Request::builder().method(method).uri(path);

    if let Some(t) = token {
        builder = builder.header("Authorization", format!("Bearer {t}"));
    }

    builder.body(Body::empty()).unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn secured_route_without_token_returns_401() {
    let validator = Arc::new(IntegrationValidator::new_ok(fake_claims(Uuid::new_v4())));
    let authorizer = Arc::new(IntegrationAuthorizer::new_ok());
    let policy = Arc::new(AuthRequirement::Required(None));

    let app = build_test_router(policy, validator, authorizer);

    let request = build_request(Method::GET, "/secured", None);
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test(flavor = "multi_thread")]
async fn secured_route_with_valid_token_returns_ok() {
    let sub_id = Uuid::new_v4();
    let validator = Arc::new(IntegrationValidator::new_ok(fake_claims(sub_id)));
    let authorizer = Arc::new(IntegrationAuthorizer::new_ok());
    let policy = Arc::new(AuthRequirement::Required(None));

    let app = build_test_router(policy, validator, authorizer);

    let request = build_request(Method::GET, "/secured", Some("valid-token"));
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(body.starts_with("user:"));
    assert!(body.contains(&sub_id.to_string()));
}

#[tokio::test(flavor = "multi_thread")]
async fn public_route_always_returns_ok_with_anonymous() {
    let validator = Arc::new(IntegrationValidator::new_ok(fake_claims(Uuid::new_v4())));
    let authorizer = Arc::new(IntegrationAuthorizer::new_ok());
    let policy = Arc::new(AuthRequirement::None);

    let app = build_test_router(policy, validator, authorizer);

    let request = build_request(Method::GET, "/public", None);
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(body.starts_with("anonymous:"));
}

#[tokio::test(flavor = "multi_thread")]
async fn optional_route_with_valid_token_returns_ok_authenticated() {
    let sub_id = Uuid::new_v4();
    let validator = Arc::new(IntegrationValidator::new_ok(fake_claims(sub_id)));
    let authorizer = Arc::new(IntegrationAuthorizer::new_ok());
    let policy = Arc::new(AuthRequirement::Optional);

    let app = build_test_router(policy, validator, authorizer);

    let request = build_request(Method::GET, "/optional", Some("valid-token"));
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(body.starts_with("user:"));
    assert!(body.contains(&sub_id.to_string()));
}

#[tokio::test(flavor = "multi_thread")]
async fn optional_route_without_token_returns_ok_anonymous() {
    let validator = Arc::new(IntegrationValidator::new_ok(fake_claims(Uuid::new_v4())));
    let authorizer = Arc::new(IntegrationAuthorizer::new_ok());
    let policy = Arc::new(AuthRequirement::Optional);

    let app = build_test_router(policy, validator, authorizer);

    let request = build_request(Method::GET, "/optional", None);
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(body.starts_with("anonymous:"));
}

#[tokio::test(flavor = "multi_thread")]
async fn cors_preflight_bypasses_auth_logic() {
    let validator = Arc::new(IntegrationValidator::new_ok(fake_claims(Uuid::new_v4())));
    let authorizer = Arc::new(IntegrationAuthorizer::new_ok());
    let policy = Arc::new(AuthRequirement::Required(None));

    let app = build_test_router(policy, validator, authorizer);

    let mut request = build_request(Method::OPTIONS, "/secured", None);
    request
        .headers_mut()
        .insert("Origin", "https://example.com".parse().unwrap());
    request
        .headers_mut()
        .insert("Access-Control-Request-Method", "GET".parse().unwrap());

    let response = app.oneshot(request).await.unwrap();

    // Should not be 401 (auth bypassed)
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test(flavor = "multi_thread")]
async fn secured_route_with_invalid_token_returns_401() {
    let validator = Arc::new(IntegrationValidator::new_err());
    let authorizer = Arc::new(IntegrationAuthorizer::new_ok());
    let policy = Arc::new(AuthRequirement::Required(None));

    let app = build_test_router(policy, validator, authorizer);

    let request = build_request(Method::GET, "/secured", Some("invalid-token"));
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test(flavor = "multi_thread")]
async fn secured_route_with_sec_requirement_denied_returns_403() {
    let validator = Arc::new(IntegrationValidator::new_ok(fake_claims(Uuid::new_v4())));
    let authorizer = Arc::new(IntegrationAuthorizer::new_err());
    let sec_req = SecRequirement::new("admin", "access");
    let policy = Arc::new(AuthRequirement::Required(Some(sec_req)));

    let app = build_test_router(policy, validator, authorizer);

    let request = build_request(Method::GET, "/secured", Some("valid-token"));
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "multi_thread")]
async fn secured_route_with_sec_requirement_allowed_returns_ok() {
    let sub_id = Uuid::new_v4();
    let validator = Arc::new(IntegrationValidator::new_ok(fake_claims(sub_id)));
    let authorizer = Arc::new(IntegrationAuthorizer::new_ok());
    let sec_req = SecRequirement::new("admin", "access");
    let policy = Arc::new(AuthRequirement::Required(Some(sec_req)));

    let app = build_test_router(policy, validator, authorizer);

    let request = build_request(Method::GET, "/secured", Some("valid-token"));
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(body.starts_with("user:"));
}
