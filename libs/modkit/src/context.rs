use serde::de::DeserializeOwned;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

// Import configuration types from the config module
use crate::config::{module_config_or_default, ConfigError, ConfigProvider};

// Note: runtime-dependent features are conditionally compiled

#[derive(Clone)]
pub struct ModuleCtx {
    module_name: Arc<str>,
    config_provider: Arc<dyn ConfigProvider>,
    client_hub: Arc<crate::client_hub::ClientHub>,
    cancellation_token: CancellationToken,
    db_handle: Option<Arc<modkit_db::DbHandle>>,
}

/// Builder for creating module-scoped contexts with resolved database handles.
///
/// This builder internally uses DbManager to resolve per-module DbHandle instances
/// at build time, ensuring ModuleCtx contains only the final, ready-to-use handle.
pub struct ModuleContextBuilder {
    config_provider: Arc<dyn ConfigProvider>,
    client_hub: Arc<crate::client_hub::ClientHub>,
    root_token: CancellationToken,
    db_manager: Option<Arc<modkit_db::DbManager>>, // internal only, never exposed to modules
}

impl ModuleContextBuilder {
    pub fn new(
        config_provider: Arc<dyn ConfigProvider>,
        client_hub: Arc<crate::client_hub::ClientHub>,
        root_token: CancellationToken,
        db_manager: Option<Arc<modkit_db::DbManager>>,
    ) -> Self {
        Self {
            config_provider,
            client_hub,
            root_token,
            db_manager,
        }
    }

    /// Build a module-scoped context, resolving the DbHandle for the given module.
    pub async fn for_module(&self, module_name: &str) -> anyhow::Result<ModuleCtx> {
        let db_handle = if let Some(mgr) = &self.db_manager {
            mgr.get(module_name).await?
        } else {
            None
        };

        Ok(ModuleCtx::new(
            Arc::<str>::from(module_name),
            self.config_provider.clone(),
            self.client_hub.clone(),
            self.root_token.child_token(),
            db_handle,
        ))
    }
}

impl ModuleCtx {
    /// Create a new module-scoped context with all required fields.
    pub fn new(
        module_name: impl Into<Arc<str>>,
        config_provider: Arc<dyn ConfigProvider>,
        client_hub: Arc<crate::client_hub::ClientHub>,
        cancellation_token: CancellationToken,
        db_handle: Option<Arc<modkit_db::DbHandle>>,
    ) -> Self {
        Self {
            module_name: module_name.into(),
            config_provider,
            client_hub,
            cancellation_token,
            db_handle,
        }
    }

    // ---- public read-only API for modules ----

    #[inline]
    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    #[inline]
    pub fn config_provider(&self) -> &dyn ConfigProvider {
        &*self.config_provider
    }

    #[inline]
    pub fn client_hub(&self) -> &crate::client_hub::ClientHub {
        &self.client_hub
    }

    #[inline]
    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }

    pub fn db_optional(&self) -> Option<Arc<modkit_db::DbHandle>> {
        self.db_handle.clone()
    }

    pub fn db_required(&self) -> anyhow::Result<Arc<modkit_db::DbHandle>> {
        self.db_handle.clone().ok_or_else(|| {
            anyhow::anyhow!(
                "Database is not configured for module '{}'",
                self.module_name
            )
        })
    }

    pub fn current_module(&self) -> Option<&str> {
        Some(&self.module_name)
    }

    /// Deserialize the module's config section into T, or use defaults if missing.
    ///
    /// This method uses lenient configuration loading: if the module is not present in config,
    /// has no config section, or the module entry is not an object, it returns `T::default()`.
    /// This allows modules to exist without configuration sections in the main config file.
    ///
    /// It extracts the 'config' field from: `modules.<name> = { database: ..., config: ... }`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[derive(serde::Deserialize, Default)]
    /// struct MyConfig {
    ///     api_key: String,
    ///     timeout_ms: u64,
    /// }
    ///
    /// let config: MyConfig = ctx.config()?;
    /// ```
    pub fn config<T: DeserializeOwned + Default>(&self) -> Result<T, ConfigError> {
        module_config_or_default(self.config_provider.as_ref(), &self.module_name)
    }

    /// Get the raw JSON value of the module's config section.
    /// Returns the 'config' field from: modules.<name> = { database: ..., config: ... }
    pub fn raw_config(&self) -> &serde_json::Value {
        use std::sync::LazyLock;

        static EMPTY: LazyLock<serde_json::Value> =
            LazyLock::new(|| serde_json::Value::Object(serde_json::Map::new()));

        if let Some(module_raw) = self.config_provider.get_module_config(&self.module_name) {
            // Try new structure first: modules.<name> = { database: ..., config: ... }
            if let Some(obj) = module_raw.as_object() {
                if let Some(config_section) = obj.get("config") {
                    return config_section;
                }
            }
        }
        &EMPTY
    }

    /// Create a derivative context with the same references but a different DB handle.
    /// This allows reusing the stable base context while providing per-module DB access.
    pub fn with_db(&self, db: Arc<modkit_db::DbHandle>) -> ModuleCtx {
        ModuleCtx {
            module_name: self.module_name.clone(),
            config_provider: self.config_provider.clone(),
            client_hub: self.client_hub.clone(),
            cancellation_token: self.cancellation_token.clone(),
            db_handle: Some(db),
        }
    }

    /// Create a derivative context with the same references but no DB handle.
    /// Useful for modules that don't require database access.
    pub fn without_db(&self) -> ModuleCtx {
        ModuleCtx {
            module_name: self.module_name.clone(),
            config_provider: self.config_provider.clone(),
            client_hub: self.client_hub.clone(),
            cancellation_token: self.cancellation_token.clone(),
            db_handle: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;
    use std::collections::HashMap;

    #[derive(Debug, PartialEq, Deserialize, Default)]
    struct TestConfig {
        #[serde(default)]
        api_key: String,
        #[serde(default)]
        timeout_ms: u64,
        #[serde(default)]
        enabled: bool,
    }

    struct MockConfigProvider {
        modules: HashMap<String, serde_json::Value>,
    }

    impl MockConfigProvider {
        fn new() -> Self {
            let mut modules = HashMap::new();

            // Valid module config
            modules.insert(
                "test_module".to_string(),
                json!({
                    "database": {
                        "url": "postgres://localhost/test"
                    },
                    "config": {
                        "api_key": "secret123",
                        "timeout_ms": 5000,
                        "enabled": true
                    }
                }),
            );

            Self { modules }
        }
    }

    impl ConfigProvider for MockConfigProvider {
        fn get_module_config(&self, module_name: &str) -> Option<&serde_json::Value> {
            self.modules.get(module_name)
        }
    }

    #[test]
    fn test_module_ctx_config_with_valid_config() {
        let provider = Arc::new(MockConfigProvider::new());
        let ctx = ModuleCtx::new(
            "test_module",
            provider,
            Arc::new(crate::client_hub::ClientHub::default()),
            CancellationToken::new(),
            None,
        );

        let result: Result<TestConfig, ConfigError> = ctx.config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.api_key, "secret123");
        assert_eq!(config.timeout_ms, 5000);
        assert!(config.enabled);
    }

    #[test]
    fn test_module_ctx_config_returns_default_for_missing_module() {
        let provider = Arc::new(MockConfigProvider::new());
        let ctx = ModuleCtx::new(
            "nonexistent_module",
            provider,
            Arc::new(crate::client_hub::ClientHub::default()),
            CancellationToken::new(),
            None,
        );

        let result: Result<TestConfig, ConfigError> = ctx.config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config, TestConfig::default());
    }
}
