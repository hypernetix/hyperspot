use crate::config::AppConfig;
use std::sync::Arc;

/// Configuration provider trait for modules
pub trait ConfigProvider: Send + Sync {
    /// Get the configuration for a specific module
    fn get_module_config(&self, module_name: &str) -> Option<&serde_json::Value>;

    /// Get a specific config value by key
    fn get_config_raw(&self, key: &str) -> Option<serde_json::Value>;
}

/// Implementation of ConfigProvider that uses AppConfig
pub struct AppConfigProvider(Arc<AppConfig>);

impl AppConfigProvider {
    pub fn new(config: AppConfig) -> Self {
        Self(Arc::new(config))
    }

    pub fn from_arc(config: Arc<AppConfig>) -> Self {
        Self(config)
    }

    pub fn inner(&self) -> &AppConfig {
        &self.0
    }
}

impl ConfigProvider for AppConfigProvider {
    fn get_module_config(&self, module_name: &str) -> Option<&serde_json::Value> {
        self.0.modules.get(module_name)
    }

    fn get_config_raw(&self, key: &str) -> Option<serde_json::Value> {
        match key {
            "server" => serde_json::to_value(&self.0.server).ok(),
            "database" => self
                .0
                .database
                .as_ref()
                .and_then(|v| serde_json::to_value(v).ok()),
            "logging" => self
                .0
                .logging
                .as_ref()
                .and_then(|v| serde_json::to_value(v).ok()),
            _ => None,
        }
    }
}
