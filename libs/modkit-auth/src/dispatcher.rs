use crate::{
    auth_mode::PluginRegistry,
    claims::Claims,
    claims_error::ClaimsError,
    config::AuthConfig,
    config_error::ConfigError,
    plugin_traits::{ClaimsPlugin, IntrospectionProvider, KeyProvider},
    validation::{validate_claims, ValidationConfig},
};
use std::sync::Arc;
use uuid::Uuid;

/// Truncate UUID to first 8 characters for safe logging
fn truncate_uuid(uuid: &Uuid) -> String {
    let s = uuid.to_string();
    s.chars().take(8).collect()
}

/// Central dispatcher for JWT and opaque token validation
///
/// Orchestrates key providers and claims plugins to validate tokens
/// using a single configured plugin.
pub struct AuthDispatcher {
    /// Registered key providers (JWKS, etc.)
    key_providers: Vec<Arc<dyn KeyProvider>>,

    /// Registered introspection providers (for opaque tokens)
    introspection_providers: Vec<Arc<dyn IntrospectionProvider>>,

    /// The authentication plugin to use for claims normalization
    plugin: Arc<dyn ClaimsPlugin>,

    /// Common validation configuration
    validation_config: ValidationConfig,
}

impl AuthDispatcher {
    /// Create a new dispatcher with validation config and plugin
    pub fn new(
        validation_config: ValidationConfig,
        config: &AuthConfig,
        registry: &PluginRegistry,
    ) -> Result<Self, ConfigError> {
        // Get the configured plugin
        let plugin = registry.get(&config.mode.provider)?.clone();

        Ok(Self {
            key_providers: Vec::new(),
            introspection_providers: Vec::new(),
            plugin,
            validation_config,
        })
    }

    /// Add a key provider
    pub fn with_key_provider(mut self, provider: Arc<dyn KeyProvider>) -> Self {
        self.key_providers.push(provider);
        self
    }

    /// Add an introspection provider
    pub fn with_introspection_provider(mut self, provider: Arc<dyn IntrospectionProvider>) -> Self {
        self.introspection_providers.push(provider);
        self
    }

    /// Validate a JWT token
    ///
    /// Workflow:
    /// 1. Try each KeyProvider until one successfully validates the signature
    /// 2. Extract issuer from token
    /// 3. Use the configured plugin to normalize claims
    /// 4. Run common validation (issuer, audience, exp, nbf, UUIDs)
    /// 5. Return normalized claims
    pub async fn validate_jwt(&self, token: &str) -> Result<Claims, ClaimsError> {
        // Step 1: Try to validate signature with each key provider
        let (header, raw_claims) = {
            let mut last_error = None;
            let mut result = None;

            for provider in &self.key_providers {
                match provider.validate_and_decode(token).await {
                    Ok(r) => {
                        tracing::debug!(
                            provider = provider.name(),
                            kid = ?r.0.kid,
                            "Successfully validated token signature"
                        );
                        result = Some(r);
                        break;
                    }
                    Err(e) => {
                        tracing::debug!(
                            provider = provider.name(),
                            error = %e,
                            "Provider failed to validate token"
                        );
                        last_error = Some(e);
                    }
                }
            }

            result.ok_or_else(|| last_error.unwrap_or(ClaimsError::NoMatchingProvider))?
        };

        // Step 2: Extract issuer for logging
        let issuer = raw_claims
            .get("iss")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ClaimsError::Malformed("missing iss claim".into()))?;

        // Step 3: Use the configured plugin
        let plugin = &self.plugin;

        tracing::debug!(
            plugin = plugin.name(),
            issuer = issuer,
            "Using configured plugin"
        );

        // Step 4: Normalize claims
        let normalized = plugin.normalize(&raw_claims).map_err(|e| {
            tracing::error!(
                plugin = plugin.name(),
                error = %e,
                issuer = issuer,
                "Failed to normalize claims"
            );
            e
        })?;

        // Step 5: Run common validation
        validate_claims(&normalized, &self.validation_config).map_err(|e| {
            tracing::warn!(
                error = %e,
                sub_prefix = %truncate_uuid(&normalized.sub),
                issuer = %normalized.issuer,
                "Common validation failed"
            );
            e
        })?;

        tracing::debug!(
            sub_prefix = %truncate_uuid(&normalized.sub),
            issuer = %normalized.issuer,
            plugin = plugin.name(),
            kid = ?header.kid,
            num_roles = normalized.roles.len(),
            num_tenants = normalized.tenants.len(),
            "Token validation successful"
        );

        Ok(normalized)
    }

    /// Validate an opaque token via introspection
    ///
    /// Workflow:
    /// 1. Try each IntrospectionProvider until one succeeds
    /// 2. Extract issuer from introspection response
    /// 3. Use the configured plugin to normalize claims
    /// 4. Run common validation
    /// 5. Return normalized claims
    pub async fn validate_opaque(&self, token: &str) -> Result<Claims, ClaimsError> {
        // Step 1: Try to introspect with each provider
        let introspection_result = {
            let mut last_error = None;
            let mut result = None;

            for provider in &self.introspection_providers {
                match provider.introspect(token).await {
                    Ok(r) => {
                        tracing::debug!(
                            provider = provider.name(),
                            "Successfully introspected token"
                        );
                        result = Some(r);
                        break;
                    }
                    Err(e) => {
                        tracing::debug!(
                            provider = provider.name(),
                            error = %e,
                            "Provider failed to introspect token"
                        );
                        last_error = Some(e);
                    }
                }
            }

            result.ok_or_else(|| {
                last_error.unwrap_or_else(|| {
                    ClaimsError::Provider("No introspection provider available".into())
                })
            })?
        };

        // Check if token is active
        if let Some(active) = introspection_result.get("active").and_then(|v| v.as_bool()) {
            if !active {
                return Err(ClaimsError::IntrospectionDenied);
            }
        }

        // Step 2: Extract issuer for logging
        let issuer = introspection_result
            .get("iss")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ClaimsError::Malformed("missing iss claim".into()))?;

        // Step 3: Use the configured plugin
        let plugin = &self.plugin;

        tracing::debug!(
            plugin = plugin.name(),
            issuer = issuer,
            "Using configured plugin for introspection"
        );

        // Step 4: Normalize claims
        let normalized = plugin.normalize(&introspection_result).map_err(|e| {
            tracing::error!(
                plugin = plugin.name(),
                error = %e,
                issuer = issuer,
                "Failed to normalize introspection response"
            );
            e
        })?;

        // Step 5: Run common validation
        validate_claims(&normalized, &self.validation_config).map_err(|e| {
            tracing::warn!(
                error = %e,
                sub_prefix = %truncate_uuid(&normalized.sub),
                issuer = %normalized.issuer,
                "Common validation failed"
            );
            e
        })?;

        tracing::debug!(
            sub_prefix = %truncate_uuid(&normalized.sub),
            issuer = %normalized.issuer,
            plugin = plugin.name(),
            num_roles = normalized.roles.len(),
            num_tenants = normalized.tenants.len(),
            "Opaque token validation successful"
        );

        Ok(normalized)
    }

    /// Get validation config (for inspection/testing)
    pub fn validation_config(&self) -> &ValidationConfig {
        &self.validation_config
    }

    /// Get the configured authentication plugin (for inspection/testing)
    pub fn plugin(&self) -> &Arc<dyn ClaimsPlugin> {
        &self.plugin
    }

    /// Trigger key refresh for all key providers
    pub async fn refresh_keys(&self) -> Result<(), Vec<ClaimsError>> {
        let mut errors = Vec::new();

        for provider in &self.key_providers {
            if let Err(e) = provider.refresh_keys().await {
                tracing::warn!(
                    provider = provider.name(),
                    error = %e,
                    "Key refresh failed"
                );
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth_mode::AuthModeConfig;
    use crate::config::PluginConfig;
    use std::collections::HashMap;

    #[test]
    fn test_dispatcher_creation() {
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

        // This would need a real plugin in registry to work
        // Just testing that the structure compiles
        let _ = &config;
    }
}
