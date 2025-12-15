use async_trait::async_trait;
use jsonwebtoken::Header;
use modkit_auth::plugin_traits::{ClaimsPlugin, KeyProvider};
use modkit_auth::validation::{
    extract_audiences, extract_string, parse_timestamp, parse_uuid_array_from_value,
    parse_uuid_from_value,
};
use modkit_auth::{
    AuthConfig, AuthDispatcher, AuthModeConfig, Claims, ClaimsError, PluginConfig, PluginRegistry,
    ValidationConfig,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// Minimal claims plugin that converts raw JSON into strongly typed `Claims`.
struct DemoClaimsPlugin;

impl ClaimsPlugin for DemoClaimsPlugin {
    fn name(&self) -> &'static str {
        "demo"
    }

    fn normalize(&self, raw: &Value) -> Result<Claims, ClaimsError> {
        let issuer_value = raw
            .get("iss")
            .ok_or_else(|| ClaimsError::MissingClaim("iss".to_string()))?;
        let issuer = extract_string(issuer_value, "iss")?;

        let sub_value = raw
            .get("sub")
            .ok_or_else(|| ClaimsError::MissingClaim("sub".to_string()))?;
        let sub = parse_uuid_from_value(sub_value, "sub")?;

        let audiences = raw.get("aud").map(extract_audiences).unwrap_or_default();

        let expires_at = raw
            .get("exp")
            .map(|value| parse_timestamp(value, "exp"))
            .transpose()?;

        let not_before = raw
            .get("nbf")
            .map(|value| parse_timestamp(value, "nbf"))
            .transpose()?;

        let tenants = raw
            .get("tenants")
            .map(|value| parse_uuid_array_from_value(value, "tenants"))
            .transpose()?
            .unwrap_or_default();

        let roles = raw
            .get("roles")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|value| value.as_str().map(ToString::to_string))
                    .collect()
            })
            .unwrap_or_default();

        Ok(Claims {
            sub,
            issuer,
            audiences,
            expires_at,
            not_before,
            tenants,
            roles,
            extras: serde_json::Map::new(),
        })
    }
}

/// Static key provider that skips signature validation for demonstration purposes.
struct StaticKeyProvider {
    claims: Value,
}

impl StaticKeyProvider {
    fn new(claims: Value) -> Self {
        Self { claims }
    }
}

#[async_trait]
impl KeyProvider for StaticKeyProvider {
    fn name(&self) -> &'static str {
        "static"
    }

    async fn validate_and_decode(&self, _token: &str) -> Result<(Header, Value), ClaimsError> {
        Ok((Header::default(), self.claims.clone()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut plugins = PluginRegistry::default();
    plugins.register("demo", Arc::new(DemoClaimsPlugin));

    let mut plugin_configs = HashMap::new();
    plugin_configs.insert(
        "demo".to_string(),
        PluginConfig::Oidc {
            tenant_claim: "tenants".to_string(),
            roles_claim: "roles".to_string(),
        },
    );

    let config = AuthConfig {
        mode: AuthModeConfig {
            provider: "demo".to_string(),
        },
        issuers: vec!["https://issuer.local".to_string()],
        audiences: vec!["demo-api".to_string()],
        plugins: plugin_configs,
        ..AuthConfig::default()
    };

    let validation = ValidationConfig {
        allowed_issuers: config.issuers.clone(),
        allowed_audiences: config.audiences.clone(),
        leeway_seconds: config.leeway_seconds,
        require_uuid_subject: true,
        require_uuid_tenants: true,
    };

    let subject = Uuid::new_v4();
    let tenant = Uuid::new_v4();
    let expires_at = OffsetDateTime::now_utc() + Duration::minutes(15);

    let raw_claims = serde_json::json!({
        "iss": "https://issuer.local",
        "sub": subject.to_string(),
        "aud": ["demo-api"],
        "exp": expires_at.unix_timestamp(),
        "tenants": [tenant.to_string()],
        "roles": ["viewer"]
    });

    let dispatcher = AuthDispatcher::new(validation, &config, &plugins)?
        .with_key_provider(Arc::new(StaticKeyProvider::new(raw_claims)));

    let claims = dispatcher.validate_jwt("demo-token").await?;
    let role_list = if claims.roles.is_empty() {
        "none".to_string()
    } else {
        claims.roles.join(", ")
    };
    println!(
        "Validated token for subject {} with roles {}",
        claims.sub, role_list
    );

    Ok(())
}
