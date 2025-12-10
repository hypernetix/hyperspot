use crate::{
    auth_mode::{AuthModeConfig, PluginRegistry},
    config_error::ConfigError,
    dispatcher::AuthDispatcher,
    plugins::{GenericOidcPlugin, KeycloakClaimsPlugin},
    providers::JwksKeyProvider,
    validation::ValidationConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Main authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Plugin to use (single mode only)
    #[serde(flatten)]
    pub mode: AuthModeConfig,

    /// Leeway in seconds for time-based validations (exp, nbf)
    #[serde(default = "default_leeway")]
    pub leeway_seconds: i64,

    /// Allowed issuers (if empty, any issuer is accepted)
    #[serde(default)]
    pub issuers: Vec<String>,

    /// Allowed audiences (if empty, any audience is accepted)
    #[serde(default)]
    pub audiences: Vec<String>,

    /// JWKS configuration
    #[serde(default)]
    pub jwks: Option<JwksConfig>,

    /// Available plugins (named configurations)
    #[serde(default)]
    pub plugins: HashMap<String, PluginConfig>,
}

fn default_leeway() -> i64 {
    60
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            mode: AuthModeConfig::default(),
            leeway_seconds: 60,
            issuers: Vec::new(),
            audiences: Vec::new(),
            jwks: None,
            plugins: HashMap::default(),
        }
    }
}

impl AuthConfig {
    /// Validate the configuration for consistency
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !self.plugins.contains_key(&self.mode.provider) {
            return Err(ConfigError::UnknownPlugin(self.mode.provider.clone()));
        }
        Ok(())
    }
}

/// JWKS endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwksConfig {
    /// JWKS endpoint URL
    pub uri: String,

    /// Refresh interval in seconds (default: 300 = 5 minutes)
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_seconds: u64,

    /// Maximum backoff in seconds (default: 3600 = 1 hour)
    #[serde(default = "default_max_backoff")]
    pub max_backoff_seconds: u64,
}

fn default_refresh_interval() -> u64 {
    300
}

fn default_max_backoff() -> u64 {
    3600
}

/// Plugin-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PluginConfig {
    Keycloak {
        /// Tenant claim field name
        #[serde(default = "default_tenant_claim")]
        tenant_claim: String,

        /// Client ID for resource_access roles
        client_roles: Option<String>,

        /// Role prefix to add to all roles
        role_prefix: Option<String>,
    },
    Oidc {
        /// Tenant claim field name
        #[serde(default = "default_tenant_claim")]
        tenant_claim: String,

        /// Roles claim field name
        #[serde(default = "default_roles_claim")]
        roles_claim: String,
    },
}

fn default_tenant_claim() -> String {
    "tenants".to_string()
}

fn default_roles_claim() -> String {
    "roles".to_string()
}

/// Build an AuthDispatcher from configuration
pub fn build_auth_dispatcher(config: &AuthConfig) -> Result<AuthDispatcher, ConfigError> {
    config.validate()?;

    let validation_config = ValidationConfig {
        allowed_issuers: config.issuers.clone(),
        allowed_audiences: config.audiences.clone(),
        leeway_seconds: config.leeway_seconds,
        require_uuid_subject: true,
        require_uuid_tenants: true,
    };

    let registry = config
        .plugins
        .iter()
        .map(|(name, plugin_config)| {
            let plugin: Arc<dyn crate::plugin_traits::ClaimsPlugin> = match plugin_config {
                PluginConfig::Keycloak {
                    tenant_claim,
                    client_roles,
                    role_prefix,
                } => Arc::new(KeycloakClaimsPlugin::new(
                    tenant_claim,
                    client_roles.clone(),
                    role_prefix.clone(),
                )),
                PluginConfig::Oidc {
                    tenant_claim,
                    roles_claim,
                } => Arc::new(GenericOidcPlugin::new(tenant_claim, roles_claim)),
            };

            tracing::debug!(
                plugin_name = %name,
                plugin_type = ?plugin_config,
                "Registered claims plugin"
            );

            (name, plugin)
        })
        .fold(PluginRegistry::default(), |mut registry, (name, plugin)| {
            registry.register(name, plugin);
            registry
        });

    let dispatcher = AuthDispatcher::new(validation_config, config, &registry)?;

    let dispatcher = if let Some(jwks_config) = &config.jwks {
        let provider = JwksKeyProvider::new(&jwks_config.uri)
            .with_refresh_interval(Duration::from_secs(jwks_config.refresh_interval_seconds))
            .with_max_backoff(Duration::from_secs(jwks_config.max_backoff_seconds));

        dispatcher.with_key_provider(Arc::new(provider))
    } else {
        dispatcher
    };

    tracing::info!(
        plugin = %config.mode.provider,
        "Authentication dispatcher initialized (single mode)"
    );

    Ok(dispatcher)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AuthConfig::default();
        assert_eq!(config.leeway_seconds, 60);
        assert!(config.issuers.is_empty());
        assert!(config.audiences.is_empty());
    }

    #[test]
    fn test_single_mode_config() {
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
            issuers: vec!["https://auth.example.com".to_string()],
            audiences: vec!["api".to_string()],
            jwks: None,
            plugins,
        };

        // Should validate successfully
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_single_mode_unknown_plugin() {
        let config = AuthConfig {
            mode: AuthModeConfig {
                provider: "unknown".to_string(),
            },
            plugins: HashMap::new(),
            ..Default::default()
        };

        // Should fail validation
        let result = config.validate();
        assert!(matches!(result, Err(ConfigError::UnknownPlugin(_))));
    }

    #[test]
    fn test_config_serialization() {
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

        let json = serde_json::to_string_pretty(&config).unwrap();
        println!("{}", json);

        let deserialized: AuthConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.leeway_seconds, 120);
        assert_eq!(deserialized.issuers.len(), 1);
    }

    #[test]
    fn test_build_dispatcher_with_jwks() {
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
            leeway_seconds: 60,
            issuers: vec!["https://auth.example.com".to_string()],
            audiences: vec!["api".to_string()],
            jwks: Some(JwksConfig {
                uri: "https://auth.example.com/.well-known/jwks.json".to_string(),
                refresh_interval_seconds: 300,
                max_backoff_seconds: 3600,
            }),
            plugins,
        };

        let dispatcher = build_auth_dispatcher(&config).unwrap();
        assert_eq!(
            dispatcher.validation_config().allowed_issuers,
            vec!["https://auth.example.com"]
        );
    }
}
