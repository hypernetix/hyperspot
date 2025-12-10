use crate::{config_error::ConfigError, plugin_traits::ClaimsPlugin};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

/// Configuration for authentication - simplified to single plugin only
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthModeConfig {
    /// Name of the plugin to use
    pub provider: String,
}

impl Default for AuthModeConfig {
    fn default() -> Self {
        AuthModeConfig {
            provider: "default".to_string(),
        }
    }
}

/// Registry of available claims plugins
#[derive(Default)]
pub struct PluginRegistry {
    plugins: HashMap<String, Arc<dyn ClaimsPlugin>>,
}

impl PluginRegistry {
    /// Register a plugin with a name
    pub fn register(&mut self, name: impl Into<String>, plugin: Arc<dyn ClaimsPlugin>) {
        self.plugins.insert(name.into(), plugin);
    }

    /// Get a plugin by name
    pub fn get(&self, name: &str) -> Result<&Arc<dyn ClaimsPlugin>, ConfigError> {
        self.plugins
            .get(name)
            .ok_or_else(|| ConfigError::UnknownPlugin(name.to_string()))
    }

    /// Check if a plugin exists
    pub fn contains(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    /// Get all plugin names
    pub fn plugin_names(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Number of registered plugins
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::{claims::Claims, claims_error::ClaimsError};
    use serde_json::Value;

    struct MockPlugin;

    impl ClaimsPlugin for MockPlugin {
        fn name(&self) -> &'static str {
            "mock"
        }

        fn normalize(&self, _raw: &Value) -> Result<Claims, ClaimsError> {
            unimplemented!()
        }
    }

    #[test]
    fn test_registry_basic_operations() {
        let mut registry = PluginRegistry::default();
        assert!(registry.is_empty());

        registry.register("mock", Arc::new(MockPlugin));
        assert_eq!(registry.len(), 1);
        assert!(registry.contains("mock"));
        assert!(!registry.contains("other"));

        let plugin = registry.get("mock").unwrap();
        assert_eq!(plugin.name(), "mock");

        let result = registry.get("unknown");
        assert!(matches!(result, Err(ConfigError::UnknownPlugin(_))));
    }

    #[test]
    fn test_auth_mode_config_serialization() {
        let config = AuthModeConfig {
            provider: "keycloak".to_string(),
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"provider\":\"keycloak\""));

        // Test deserialization
        let deserialized: AuthModeConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.provider, "keycloak");
    }

    #[test]
    fn test_auth_mode_config_default() {
        let config = AuthModeConfig::default();
        assert_eq!(config.provider, "default");
    }
}
