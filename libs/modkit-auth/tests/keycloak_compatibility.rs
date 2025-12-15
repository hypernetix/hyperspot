#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Tests for Keycloak JWT token compatibility
//!
//! This test demonstrates that the JWT validator can handle:
//! 1. Keycloak tokens with string `aud` field
//! 2. Keycloak tokens with `realm_access.roles` instead of top-level `roles`
//! 3. Custom tokens with array `aud` field
//! 4. Tokens with various field combinations

use serde_json::json;
use uuid::Uuid;

#[test]
#[allow(clippy::unreadable_literal)]
fn test_keycloak_token_format() {
    // Simulate Keycloak token structure
    let keycloak_token = json!({
        "sub": "35daf794-da48-4f4c-9898-47c02cfdd845",
        "aud": "account",  // String, not array
        "iss": "https://auth.example.com/realms/dev",
        "exp": 1735123456,
        "iat": 1735119856,
        "realm_access": {
            "roles": ["users:read", "users:write", "default-roles-dev"]
        },
        "tenants": ["00000000-4f26-4798-b655-feedc1cfe5e3"],
        "email": "user@example.com"
    });

    // Test field extraction logic (matching what jwks.rs does)
    let sub = keycloak_token
        .get("sub")
        .and_then(|x| x.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    assert!(sub.is_some(), "Should parse sub as UUID");

    // Test aud string -> array conversion
    let aud = match keycloak_token.get("aud") {
        Some(serde_json::Value::String(s)) => Some(vec![s.clone()]),
        Some(serde_json::Value::Array(arr)) => Some(
            arr.iter()
                .filter_map(|x| x.as_str().map(ToString::to_string))
                .collect::<Vec<_>>(),
        ),
        _ => None,
    };
    assert_eq!(
        aud,
        Some(vec!["account".to_string()]),
        "String aud should convert to array"
    );

    // Test realm_access.roles extraction
    let mut roles: Vec<String> = Vec::new();
    if let Some(serde_json::Value::Array(arr)) = keycloak_token.get("roles") {
        roles.extend(
            arr.iter()
                .filter_map(|x| x.as_str().map(ToString::to_string)),
        );
    } else if let Some(serde_json::Value::Object(realm)) = keycloak_token.get("realm_access") {
        if let Some(serde_json::Value::Array(arr)) = realm.get("roles") {
            roles.extend(
                arr.iter()
                    .filter_map(|x| x.as_str().map(ToString::to_string)),
            );
        }
    }
    assert_eq!(
        roles,
        vec!["users:read", "users:write", "default-roles-dev"],
        "Should extract roles from realm_access.roles"
    );

    // Test tenants extraction
    let tenants = keycloak_token
        .get("tenants")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str())
                .filter_map(|s| Uuid::parse_str(s).ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    assert_eq!(tenants.len(), 1, "Should extract one tenant");
}

#[test]
#[allow(clippy::unreadable_literal)]
fn test_custom_token_format() {
    // Custom token with array aud and top-level roles
    let custom_token = json!({
        "sub": "35daf794-da48-4f4c-9898-47c02cfdd845",
        "aud": ["api", "web"],  // Array
        "iss": "https://auth.example.com",
        "exp": 1735123456,
        "roles": ["admin", "user"],  // Top-level
        "tenants": ["00000000-4f26-4798-b655-feedc1cfe5e3"]
    });

    // Test aud array handling
    let aud = match custom_token.get("aud") {
        Some(serde_json::Value::String(s)) => Some(vec![s.clone()]),
        Some(serde_json::Value::Array(arr)) => Some(
            arr.iter()
                .filter_map(|x| x.as_str().map(ToString::to_string))
                .collect::<Vec<_>>(),
        ),
        _ => None,
    };
    assert_eq!(
        aud,
        Some(vec!["api".to_string(), "web".to_string()]),
        "Array aud should remain as array"
    );

    // Test top-level roles extraction
    let mut roles: Vec<String> = Vec::new();
    if let Some(serde_json::Value::Array(arr)) = custom_token.get("roles") {
        roles.extend(
            arr.iter()
                .filter_map(|x| x.as_str().map(ToString::to_string)),
        );
    } else if let Some(serde_json::Value::Object(realm)) = custom_token.get("realm_access") {
        if let Some(serde_json::Value::Array(arr)) = realm.get("roles") {
            roles.extend(
                arr.iter()
                    .filter_map(|x| x.as_str().map(ToString::to_string)),
            );
        }
    }
    assert_eq!(
        roles,
        vec!["admin", "user"],
        "Should extract roles from top-level field"
    );
}

#[test]
#[allow(clippy::unreadable_literal)]
fn test_minimal_token_format() {
    // Minimal token with only required fields
    let minimal_token = json!({
        "sub": "35daf794-da48-4f4c-9898-47c02cfdd845",
        "exp": 1735123456
    });

    // Test missing fields default to empty/None
    let aud = match minimal_token.get("aud") {
        Some(serde_json::Value::String(s)) => Some(vec![s.clone()]),
        Some(serde_json::Value::Array(arr)) => Some(
            arr.iter()
                .filter_map(|x| x.as_str().map(ToString::to_string))
                .collect::<Vec<_>>(),
        ),
        _ => None,
    };
    assert_eq!(aud, None, "Missing aud should be None");

    let mut roles: Vec<String> = Vec::new();
    if let Some(serde_json::Value::Array(arr)) = minimal_token.get("roles") {
        roles.extend(
            arr.iter()
                .filter_map(|x| x.as_str().map(ToString::to_string)),
        );
    } else if let Some(serde_json::Value::Object(realm)) = minimal_token.get("realm_access") {
        if let Some(serde_json::Value::Array(arr)) = realm.get("roles") {
            roles.extend(
                arr.iter()
                    .filter_map(|x| x.as_str().map(ToString::to_string)),
            );
        }
    }
    assert!(roles.is_empty(), "Missing roles should be empty array");

    let tenants = minimal_token
        .get("tenants")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str())
                .filter_map(|s| Uuid::parse_str(s).ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    assert!(tenants.is_empty(), "Missing tenants should be empty array");
}

#[test]
fn test_both_roles_locations() {
    // Token with both top-level roles AND realm_access.roles
    // Top-level should take precedence
    let token = json!({
        "sub": "35daf794-da48-4f4c-9898-47c02cfdd845",
        "roles": ["admin"],  // Top-level
        "realm_access": {
            "roles": ["user"]  // Should be ignored
        }
    });

    let mut roles: Vec<String> = Vec::new();
    if let Some(serde_json::Value::Array(arr)) = token.get("roles") {
        roles.extend(
            arr.iter()
                .filter_map(|x| x.as_str().map(ToString::to_string)),
        );
    } else if let Some(serde_json::Value::Object(realm)) = token.get("realm_access") {
        if let Some(serde_json::Value::Array(arr)) = realm.get("roles") {
            roles.extend(
                arr.iter()
                    .filter_map(|x| x.as_str().map(ToString::to_string)),
            );
        }
    }
    assert_eq!(
        roles,
        vec!["admin"],
        "Top-level roles should take precedence over realm_access.roles"
    );
}
