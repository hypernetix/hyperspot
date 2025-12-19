#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use modkit_auth::axum_ext::AuthPolicyLayer;
use modkit_auth::{
    authorizer::RoleAuthorizer,
    axum_ext::Authz,
    build_auth_dispatcher,
    scope_builder::SimpleScopeBuilder,
    traits::{PrimaryAuthorizer, ScopeBuilder, TokenValidator},
    types::{AuthRequirement, RoutePolicy},
    AuthConfig, AuthModeConfig, Claims, PluginConfig, ValidationConfig,
};
use std::collections::HashMap;
use std::sync::Arc;
use tower::ServiceExt;

/// Static policy that always requires auth
#[derive(Clone)]
struct AlwaysRequiredPolicy;

#[async_trait::async_trait]
impl RoutePolicy for AlwaysRequiredPolicy {
    async fn resolve(&self, _method: &axum::http::Method, _path: &str) -> AuthRequirement {
        AuthRequirement::Required(None)
    }
}

/// Static policy for optional auth
#[derive(Clone)]
struct AlwaysOptionalPolicy;

#[async_trait::async_trait]
impl RoutePolicy for AlwaysOptionalPolicy {
    async fn resolve(&self, _method: &axum::http::Method, _path: &str) -> AuthRequirement {
        AuthRequirement::Optional
    }
}

/// Helper to create a minimal test configuration (single mode)
fn create_test_config() -> AuthConfig {
    let mut plugins = HashMap::new();
    plugins.insert(
        "test-oidc".to_owned(),
        PluginConfig::Oidc {
            tenant_claim: "tenants".to_owned(),
            roles_claim: "roles".to_owned(),
        },
    );

    AuthConfig {
        mode: AuthModeConfig {
            provider: "test-oidc".to_owned(),
        },
        leeway_seconds: 60,
        issuers: vec!["https://test.example.com".to_owned()],
        audiences: vec!["test-api".to_owned()],
        jwks: None,
        plugins,
    }
}

async fn protected_handler(Authz(ctx): Authz) -> impl IntoResponse {
    format!("Protected: {}", ctx.subject_id())
}

async fn optional_handler() -> impl IntoResponse {
    "OK"
}

fn create_app(
    policy: Arc<dyn RoutePolicy>,
    validator: Arc<dyn TokenValidator>,
    scope_builder: Arc<dyn ScopeBuilder>,
    authorizer: Arc<dyn PrimaryAuthorizer>,
) -> Router {
    Router::new()
        .route("/protected", get(protected_handler))
        .layer(AuthPolicyLayer::new(
            validator,
            scope_builder,
            authorizer,
            policy,
        ))
}

#[tokio::test]
async fn test_middleware_returns_401_for_missing_token() {
    let config = create_test_config();
    let dispatcher = Arc::new(build_auth_dispatcher(&config).unwrap());

    let validator: Arc<dyn TokenValidator> = dispatcher;
    let scope_builder: Arc<dyn ScopeBuilder> = Arc::new(SimpleScopeBuilder);
    let authorizer: Arc<dyn PrimaryAuthorizer> = Arc::new(RoleAuthorizer);
    let policy: Arc<dyn RoutePolicy> = Arc::new(AlwaysRequiredPolicy);

    let app = create_app(policy, validator, scope_builder, authorizer);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_middleware_returns_401_for_invalid_token() {
    let config = create_test_config();
    let dispatcher = Arc::new(build_auth_dispatcher(&config).unwrap());

    let validator: Arc<dyn TokenValidator> = dispatcher;
    let scope_builder: Arc<dyn ScopeBuilder> = Arc::new(SimpleScopeBuilder);
    let authorizer: Arc<dyn PrimaryAuthorizer> = Arc::new(RoleAuthorizer);
    let policy: Arc<dyn RoutePolicy> = Arc::new(AlwaysRequiredPolicy);

    let app = create_app(policy, validator, scope_builder, authorizer);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .header("Authorization", "Bearer invalid-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_middleware_allows_options_preflight() {
    let config = create_test_config();
    let dispatcher = Arc::new(build_auth_dispatcher(&config).unwrap());

    let validator: Arc<dyn TokenValidator> = dispatcher;
    let scope_builder: Arc<dyn ScopeBuilder> = Arc::new(SimpleScopeBuilder);
    let authorizer: Arc<dyn PrimaryAuthorizer> = Arc::new(RoleAuthorizer);
    let policy: Arc<dyn RoutePolicy> = Arc::new(AlwaysRequiredPolicy);

    let app = create_app(policy, validator, scope_builder, authorizer);

    // OPTIONS request with CORS headers
    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/protected")
                .header("Origin", "https://example.com")
                .header("Access-Control-Request-Method", "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should not return 401 - CORS preflight should bypass auth
    // Note: We get 405 Method Not Allowed because we only defined GET route
    // But it shouldn't be 401
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_optional_auth_inserts_anonymous_context() {
    let config = create_test_config();
    let dispatcher = Arc::new(build_auth_dispatcher(&config).unwrap());

    let validator: Arc<dyn TokenValidator> = dispatcher;
    let scope_builder: Arc<dyn ScopeBuilder> = Arc::new(SimpleScopeBuilder);
    let authorizer: Arc<dyn PrimaryAuthorizer> = Arc::new(RoleAuthorizer);
    let policy: Arc<dyn RoutePolicy> = Arc::new(AlwaysOptionalPolicy);

    let app = Router::new()
        .route("/optional", get(optional_handler))
        .layer(AuthPolicyLayer::new(
            validator,
            scope_builder,
            authorizer,
            policy,
        ));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/optional")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_single_mode_dispatcher_created() {
    let mut plugins = HashMap::new();
    plugins.insert(
        "plugin-a".to_owned(),
        PluginConfig::Oidc {
            tenant_claim: "tenants".to_owned(),
            roles_claim: "roles".to_owned(),
        },
    );

    let config = AuthConfig {
        mode: AuthModeConfig {
            provider: "plugin-a".to_owned(),
        },
        leeway_seconds: 60,
        issuers: vec!["https://test.example.com".to_owned()],
        audiences: vec!["api".to_owned()],
        jwks: None,
        plugins,
    };

    let dispatcher = build_auth_dispatcher(&config).unwrap();

    // Should build successfully
    assert_eq!(
        dispatcher.validation_config().allowed_issuers,
        vec!["https://test.example.com"]
    );
}

#[tokio::test]
async fn test_claims_uuid_validation() {
    use uuid::Uuid;

    let config = ValidationConfig {
        allowed_issuers: vec!["https://test.example.com".to_owned()],
        allowed_audiences: vec!["api".to_owned()],
        leeway_seconds: 60,
        require_uuid_subject: true,
        require_uuid_tenants: true,
    };

    let claims = Claims {
        issuer: "https://test.example.com".to_owned(),
        subject: Uuid::new_v4(),
        audiences: vec!["api".to_owned()],
        expires_at: Some(time::OffsetDateTime::now_utc() + time::Duration::hours(1)),
        not_before: None,
        issued_at: None,
        jwt_id: None,
        tenant_id: Uuid::new_v4(),
        permissions: vec![],
        extras: serde_json::Map::new(),
    };

    let result = modkit_auth::validation::validate_claims(&claims, &config);
    assert!(result.is_ok(), "Valid claims should pass validation");
}

#[tokio::test]
async fn test_expired_token_fails_validation() {
    use uuid::Uuid;

    let config = ValidationConfig {
        allowed_issuers: vec!["https://test.example.com".to_owned()],
        allowed_audiences: vec!["api".to_owned()],
        leeway_seconds: 5, // Short leeway
        require_uuid_subject: true,
        require_uuid_tenants: true,
    };

    let claims = Claims {
        issuer: "https://test.example.com".to_owned(),
        subject: Uuid::new_v4(),
        audiences: vec!["api".to_owned()],
        expires_at: Some(time::OffsetDateTime::now_utc() - time::Duration::hours(1)),
        not_before: None,
        issued_at: None,
        jwt_id: None,
        tenant_id: Uuid::new_v4(),
        permissions: vec![],
        extras: serde_json::Map::new(),
    };

    let result = modkit_auth::validation::validate_claims(&claims, &config);
    assert!(result.is_err(), "Expired claims should fail validation");
}

#[tokio::test]
async fn test_invalid_issuer_fails_validation() {
    use uuid::Uuid;

    let config = ValidationConfig {
        allowed_issuers: vec!["https://allowed.example.com".to_owned()],
        allowed_audiences: vec!["api".to_owned()],
        leeway_seconds: 60,
        require_uuid_subject: true,
        require_uuid_tenants: true,
    };

    let claims = Claims {
        issuer: "https://invalid.example.com".to_owned(),
        subject: Uuid::new_v4(),
        audiences: vec!["api".to_owned()],
        expires_at: Some(time::OffsetDateTime::now_utc() + time::Duration::hours(1)),
        not_before: None,
        issued_at: None,
        jwt_id: None,
        tenant_id: Uuid::new_v4(),
        permissions: vec![],
        extras: serde_json::Map::new(),
    };

    let result = modkit_auth::validation::validate_claims(&claims, &config);
    assert!(result.is_err(), "Invalid issuer should fail validation");
}
