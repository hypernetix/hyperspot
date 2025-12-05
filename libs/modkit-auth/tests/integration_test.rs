#![allow(clippy::unwrap_used, clippy::expect_used)]

use modkit_auth::{
    build_auth_dispatcher, AuthConfig, AuthModeConfig, Claims, ClaimsError, JwksConfig,
    PluginConfig,
};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_dispatcher_single_mode() {
    // Setup config with single mode
    let mut plugins = HashMap::new();
    plugins.insert(
        "keycloak".to_string(),
        PluginConfig::Keycloak {
            tenant_claim: "tenants".to_string(),
            client_roles: Some("modkit-api".to_string()),
            role_prefix: None,
        },
    );

    let config = AuthConfig {
        mode: AuthModeConfig {
            provider: "keycloak".to_string(),
        },
        leeway_seconds: 60,
        issuers: vec!["https://keycloak.example.com/realms/test".to_string()],
        audiences: vec!["modkit-api".to_string()],
        jwks: None,
        plugins,
    };

    let dispatcher = build_auth_dispatcher(&config).unwrap();

    // Verify dispatcher was created successfully
    assert_eq!(
        dispatcher.validation_config().allowed_issuers,
        vec!["https://keycloak.example.com/realms/test"]
    );
}

#[test]
fn test_config_validation_unknown_plugin() {
    let config = AuthConfig {
        mode: AuthModeConfig {
            provider: "unknown".to_string(),
        },
        plugins: HashMap::new(),
        ..Default::default()
    };

    let result = config.validate();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        modkit_auth::ConfigError::UnknownPlugin(_)
    ));
}

#[test]
fn test_claims_structure() {
    let user_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();

    let normalized = Claims {
        sub: user_id,
        issuer: "https://auth.example.com".to_string(),
        audiences: vec!["api".to_string()],
        expires_at: Some(time::OffsetDateTime::now_utc() + time::Duration::hours(1)),
        not_before: None,
        tenants: vec![tenant_id],
        roles: vec!["admin".to_string(), "user".to_string()],
        extras: {
            let mut map = serde_json::Map::new();
            map.insert("email".to_string(), json!("test@example.com"));
            map
        },
    };

    // Verify normalized claims structure
    assert_eq!(normalized.sub, user_id);
    assert_eq!(normalized.issuer, "https://auth.example.com");
    assert_eq!(normalized.tenants, vec![tenant_id]);
    assert_eq!(normalized.roles.len(), 2);
    assert_eq!(
        normalized.extras.get("email").and_then(|v| v.as_str()),
        Some("test@example.com")
    );
}

#[test]
fn test_claims_validation() {
    let user_id = Uuid::new_v4();

    // Test expired token
    let expired = Claims {
        sub: user_id,
        issuer: "https://auth.example.com".to_string(),
        audiences: vec!["api".to_string()],
        expires_at: Some(time::OffsetDateTime::now_utc() - time::Duration::hours(1)),
        not_before: None,
        tenants: vec![],
        roles: vec![],
        extras: serde_json::Map::new(),
    };

    assert!(expired.is_expired());

    // Test not yet valid
    let future = Claims {
        sub: user_id,
        issuer: "https://auth.example.com".to_string(),
        audiences: vec!["api".to_string()],
        expires_at: None,
        not_before: Some(time::OffsetDateTime::now_utc() + time::Duration::hours(1)),
        tenants: vec![],
        roles: vec![],
        extras: serde_json::Map::new(),
    };

    assert!(!future.is_valid_yet());
}

#[test]
fn test_config_serialization_roundtrip() {
    let mut plugins = HashMap::new();
    plugins.insert(
        "keycloak".to_string(),
        PluginConfig::Keycloak {
            tenant_claim: "tenants".to_string(),
            client_roles: Some("modkit-api".to_string()),
            role_prefix: Some("kc".to_string()),
        },
    );

    let config = AuthConfig {
        mode: AuthModeConfig {
            provider: "keycloak".to_string(),
        },
        leeway_seconds: 120,
        issuers: vec!["https://auth.example.com".to_string()],
        audiences: vec!["api".to_string()],
        jwks: Some(JwksConfig {
            uri: "https://auth.example.com/.well-known/jwks.json".to_string(),
            refresh_interval_seconds: 300,
            max_backoff_seconds: 3600,
        }),
        plugins,
    };

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&config).unwrap();

    // Deserialize back
    let deserialized: AuthConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.leeway_seconds, 120);
    assert_eq!(deserialized.issuers.len(), 1);
    assert_eq!(deserialized.audiences.len(), 1);
    assert!(deserialized.jwks.is_some());
    assert_eq!(deserialized.plugins.len(), 1);
}

#[test]
fn test_claims_error_types() {
    let err = ClaimsError::InvalidIssuer {
        expected: vec!["https://expected.com".to_string()],
        actual: "https://actual.com".to_string(),
    };
    assert!(err.to_string().contains("expected"));
    assert!(err.to_string().contains("actual"));

    let err = ClaimsError::Expired;
    assert_eq!(err.to_string(), "Token expired");

    let err = ClaimsError::MissingClaim("sub".to_string());
    assert!(err.to_string().contains("sub"));

    // Test new error variants
    let err = ClaimsError::NoMatchingPlugin;
    assert!(err.to_string().contains("No matching plugin"));

    let err = ClaimsError::IntrospectionDenied;
    assert!(err.to_string().contains("Introspection denied"));

    let err = ClaimsError::UnknownKidAfterRefresh;
    assert!(err.to_string().contains("Unknown key ID after refresh"));
}

#[tokio::test]
async fn test_dispatcher_refresh_keys_with_no_providers() {
    let mut plugins = HashMap::new();
    plugins.insert(
        "oidc".to_string(),
        PluginConfig::Oidc {
            tenant_claim: "tenants".to_string(),
            roles_claim: "roles".to_string(),
        },
    );

    let config = AuthConfig {
        mode: AuthModeConfig {
            provider: "oidc".to_string(),
        },
        plugins,
        ..Default::default()
    };

    let dispatcher = build_auth_dispatcher(&config).unwrap();

    // Should succeed even with no key providers
    let result = dispatcher.refresh_keys().await;
    assert!(result.is_ok());
}
