#![allow(clippy::unwrap_used, clippy::expect_used)]

use modkit_auth::claims::Permission;
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
        "keycloak".to_owned(),
        PluginConfig::Keycloak {
            tenant_claim: "tenant".to_owned(),
            client_roles: Some("modkit-api".to_owned()),
            role_prefix: None,
        },
    );

    let config = AuthConfig {
        mode: AuthModeConfig {
            provider: "keycloak".to_owned(),
        },
        leeway_seconds: 60,
        issuers: vec!["https://keycloak.example.com/realms/test".to_owned()],
        audiences: vec!["modkit-api".to_owned()],
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
            provider: "unknown".to_owned(),
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
        subject: user_id,
        issuer: "https://auth.example.com".to_owned(),
        audiences: vec!["api".to_owned()],
        expires_at: Some(time::OffsetDateTime::now_utc() + time::Duration::hours(1)),
        not_before: None,
        issued_at: None,
        jwt_id: None,
        tenant_id,
        permissions: vec![
            Permission::new("resource", "read"),
            Permission::new("resource", "write"),
        ],
        extras: {
            let mut map = serde_json::Map::new();
            map.insert("email".to_owned(), json!("test@example.com"));
            map
        },
    };

    // Verify normalized claims structure
    assert_eq!(normalized.subject, user_id);
    assert_eq!(normalized.issuer, "https://auth.example.com");
    assert_eq!(normalized.tenant_id, tenant_id);
    assert_eq!(normalized.permissions.len(), 2);
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
        subject: user_id,
        issuer: "https://auth.example.com".to_owned(),
        audiences: vec!["api".to_owned()],
        expires_at: Some(time::OffsetDateTime::now_utc() - time::Duration::hours(1)),
        not_before: None,
        issued_at: None,
        jwt_id: None,
        tenant_id: Uuid::new_v4(),
        permissions: vec![],
        extras: serde_json::Map::new(),
    };

    assert!(expired.is_expired());

    // Test not yet valid
    let future = Claims {
        subject: user_id,
        issuer: "https://auth.example.com".to_owned(),
        audiences: vec!["api".to_owned()],
        expires_at: None,
        not_before: Some(time::OffsetDateTime::now_utc() + time::Duration::hours(1)),
        issued_at: None,
        jwt_id: None,
        tenant_id: Uuid::new_v4(),
        permissions: vec![],
        extras: serde_json::Map::new(),
    };

    assert!(!future.is_valid_yet());
}

#[test]
fn test_config_serialization_roundtrip() {
    let mut plugins = HashMap::new();
    plugins.insert(
        "keycloak".to_owned(),
        PluginConfig::Keycloak {
            tenant_claim: "tenants".to_owned(),
            client_roles: Some("modkit-api".to_owned()),
            role_prefix: Some("kc".to_owned()),
        },
    );

    let config = AuthConfig {
        mode: AuthModeConfig {
            provider: "keycloak".to_owned(),
        },
        leeway_seconds: 120,
        issuers: vec!["https://auth.example.com".to_owned()],
        audiences: vec!["api".to_owned()],
        jwks: Some(JwksConfig {
            uri: "https://auth.example.com/.well-known/jwks.json".to_owned(),
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
        expected: vec!["https://expected.com".to_owned()],
        actual: "https://actual.com".to_owned(),
    };
    assert!(err.to_string().contains("expected"));
    assert!(err.to_string().contains("actual"));

    let err = ClaimsError::Expired;
    assert_eq!(err.to_string(), "Token expired");

    let err = ClaimsError::MissingClaim("sub".to_owned());
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
        "oidc".to_owned(),
        PluginConfig::Oidc {
            tenant_claim: "tenant".to_owned(),
            roles_claim: "roles".to_owned(),
        },
    );

    let config = AuthConfig {
        mode: AuthModeConfig {
            provider: "oidc".to_owned(),
        },
        plugins,
        ..Default::default()
    };

    let dispatcher = build_auth_dispatcher(&config).unwrap();

    // Should succeed even with no key providers
    let result = dispatcher.refresh_keys().await;
    assert!(result.is_ok());
}
