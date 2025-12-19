use async_trait::async_trait;
use jsonwebtoken::Header;
use modkit_auth::claims::Permission;
use modkit_auth::plugin_traits::{ClaimsPlugin, KeyProvider};
use modkit_auth::validation::{
    extract_audiences, extract_string, parse_timestamp, parse_uuid_from_value,
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
            .ok_or_else(|| ClaimsError::MissingClaim("iss".to_owned()))?;
        let issuer = extract_string(issuer_value, "iss")?;

        let sub_value = raw
            .get("sub")
            .ok_or_else(|| ClaimsError::MissingClaim("sub".to_owned()))?;
        let subject = parse_uuid_from_value(sub_value, "sub")?;

        let audiences = raw.get("aud").map(extract_audiences).unwrap_or_default();

        let expires_at = raw
            .get("exp")
            .map(|value| parse_timestamp(value, "exp"))
            .transpose()?;

        let not_before = raw
            .get("nbf")
            .map(|value| parse_timestamp(value, "nbf"))
            .transpose()?;

        let issued_at = raw
            .get("iat")
            .map(|value| parse_timestamp(value, "iat"))
            .transpose()?;

        let jwt_id = raw
            .get("jti")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        let tenant_id_value = raw
            .get("tenant_id")
            .ok_or_else(|| ClaimsError::MissingClaim("tenant_id".to_owned()))?;
        let tenant_id = parse_uuid_from_value(tenant_id_value, "tenant_id")?;

        let permissions: Vec<Permission> = raw
            .get("roles")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|value| value.as_str())
                    .filter_map(|role| {
                        if let Some(pos) = role.rfind(':') {
                            Permission::builder()
                                .resource_pattern(&role[..pos])
                                .action(&role[pos + 1..])
                                .build()
                                .ok()
                        } else {
                            Permission::builder()
                                .resource_pattern(role)
                                .action("*")
                                .build()
                                .ok()
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(Claims {
            issuer,
            subject,
            audiences,
            expires_at,
            not_before,
            issued_at,
            jwt_id,
            tenant_id,
            permissions,
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
        "demo".to_owned(),
        PluginConfig::Oidc {
            tenant_claim: "tenants".to_owned(),
            roles_claim: "roles".to_owned(),
        },
    );

    let config = AuthConfig {
        mode: AuthModeConfig {
            provider: "demo".to_owned(),
        },
        issuers: vec!["https://issuer.local".to_owned()],
        audiences: vec!["demo-api".to_owned()],
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
        "tenant_id": tenant.to_string(),
        "roles": ["viewer:read"]
    });

    let dispatcher = AuthDispatcher::new(validation, &config, &plugins)?
        .with_key_provider(Arc::new(StaticKeyProvider::new(raw_claims)));

    let claims = dispatcher.validate_jwt("demo-token").await?;
    let perm_list = if claims.permissions.is_empty() {
        "none".to_owned()
    } else {
        claims
            .permissions
            .iter()
            .map(|p| format!("{}:{}", p.resource_pattern(), p.action()))
            .collect::<Vec<_>>()
            .join(", ")
    };
    println!(
        "Validated token for subject {} with permissions {}",
        claims.subject, perm_list
    );

    Ok(())
}
