use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware,
    response::IntoResponse,
    routing::get,
    Router,
};
use modkit_auth::{
    axum_ext::{auth_optional, auth_required, Authz},
    build_auth_dispatcher, AuthConfig, AuthModeConfig, Claims, PluginConfig, ValidationConfig,
};
use std::collections::HashMap;
use std::sync::Arc;
use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

/// Helper to create a minimal test configuration (single mode)
fn create_test_config() -> AuthConfig {
    let mut plugins = HashMap::new();
    plugins.insert(
        "test-oidc".to_string(),
        PluginConfig::Oidc {
            tenant_claim: "tenants".to_string(),
            roles_claim: "roles".to_string(),
        },
    );

    AuthConfig {
        mode: AuthModeConfig {
            provider: "test-oidc".to_string(),
        },
        leeway_seconds: 60,
        issuers: vec!["https://test.example.com".to_string()],
        audiences: vec!["test-api".to_string()],
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

#[tokio::test]
async fn test_middleware_returns_401_for_missing_token() {
    let config = create_test_config();
    let dispatcher = Arc::new(build_auth_dispatcher(&config).unwrap());

    let app = Router::new()
        .route("/protected", get(protected_handler))
        .layer(middleware::from_fn_with_state(
            dispatcher.clone(),
            auth_required,
        ));

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

    let app = Router::new()
        .route("/protected", get(protected_handler))
        .layer(middleware::from_fn_with_state(
            dispatcher.clone(),
            auth_required,
        ));

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

    let app = Router::new()
        .route("/protected", get(protected_handler))
        .layer(middleware::from_fn_with_state(
            dispatcher.clone(),
            auth_required,
        ));

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

    let app = Router::new()
        .route("/optional", get(optional_handler))
        .layer(middleware::from_fn_with_state(
            dispatcher.clone(),
            auth_optional,
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
        "plugin-a".to_string(),
        PluginConfig::Oidc {
            tenant_claim: "tenants".to_string(),
            roles_claim: "roles".to_string(),
        },
    );

    let config = AuthConfig {
        mode: AuthModeConfig {
            provider: "plugin-a".to_string(),
        },
        leeway_seconds: 60,
        issuers: vec!["https://test.example.com".to_string()],
        audiences: vec!["api".to_string()],
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
        allowed_issuers: vec!["https://test.example.com".to_string()],
        allowed_audiences: vec!["api".to_string()],
        leeway_seconds: 60,
        require_uuid_subject: true,
        require_uuid_tenants: true,
    };

    let claims = Claims {
        sub: Uuid::new_v4(),
        issuer: "https://test.example.com".to_string(),
        audiences: vec!["api".to_string()],
        expires_at: Some(time::OffsetDateTime::now_utc() + time::Duration::hours(1)),
        not_before: None,
        tenants: vec![Uuid::new_v4()],
        roles: vec!["user".to_string()],
        extras: serde_json::Map::new(),
    };

    let result = modkit_auth::validation::validate_claims(&claims, &config);
    assert!(result.is_ok(), "Valid claims should pass validation");
}

#[tokio::test]
async fn test_expired_token_fails_validation() {
    use uuid::Uuid;

    let config = ValidationConfig {
        allowed_issuers: vec!["https://test.example.com".to_string()],
        allowed_audiences: vec!["api".to_string()],
        leeway_seconds: 5, // Short leeway
        require_uuid_subject: true,
        require_uuid_tenants: true,
    };

    let claims = Claims {
        sub: Uuid::new_v4(),
        issuer: "https://test.example.com".to_string(),
        audiences: vec!["api".to_string()],
        expires_at: Some(time::OffsetDateTime::now_utc() - time::Duration::hours(1)),
        not_before: None,
        tenants: vec![],
        roles: vec![],
        extras: serde_json::Map::new(),
    };

    let result = modkit_auth::validation::validate_claims(&claims, &config);
    assert!(result.is_err(), "Expired claims should fail validation");
}

#[tokio::test]
async fn test_invalid_issuer_fails_validation() {
    use uuid::Uuid;

    let config = ValidationConfig {
        allowed_issuers: vec!["https://allowed.example.com".to_string()],
        allowed_audiences: vec!["api".to_string()],
        leeway_seconds: 60,
        require_uuid_subject: true,
        require_uuid_tenants: true,
    };

    let claims = Claims {
        sub: Uuid::new_v4(),
        issuer: "https://invalid.example.com".to_string(),
        audiences: vec!["api".to_string()],
        expires_at: Some(time::OffsetDateTime::now_utc() + time::Duration::hours(1)),
        not_before: None,
        tenants: vec![],
        roles: vec![],
        extras: serde_json::Map::new(),
    };

    let result = modkit_auth::validation::validate_claims(&claims, &config);
    assert!(result.is_err(), "Invalid issuer should fail validation");
}
