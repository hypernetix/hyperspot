use serde::de::DeserializeOwned;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

// Note: runtime-dependent features are conditionally compiled

/// Provider of module-specific configuration (raw JSON sections only).
pub trait ConfigProvider: Send + Sync {
    /// Returns raw JSON section for the module, if any.
    fn get_module_config(&self, module_name: &str) -> Option<&serde_json::Value>;
}

#[derive(Clone)]
pub struct ModuleCtx {
    pub(crate) db: Option<Arc<db::DbHandle>>,
    pub(crate) config_provider: Option<Arc<dyn ConfigProvider>>,
    pub(crate) client_hub: Arc<crate::client_hub::ClientHub>,
    pub(crate) cancellation_token: CancellationToken,
    pub(crate) module_name: Option<Arc<str>>,
}

// ---- construction/scoping (crate-private) ----
// TODO: make it private (but need to fix test crate visibility issues)
pub struct ModuleCtxBuilder {
    inner: ModuleCtx,
}

impl ModuleCtxBuilder {
    pub fn new(token: CancellationToken) -> Self {
        Self {
            inner: ModuleCtx::from_token(token),
        }
    }
    pub fn with_db(mut self, db: Arc<db::DbHandle>) -> Self {
        self.inner.db = Some(db);
        self
    }
    pub fn with_config_provider(mut self, p: Arc<dyn ConfigProvider>) -> Self {
        self.inner.config_provider = Some(p);
        self
    }
    pub(crate) fn with_client_hub(mut self, hub: Arc<crate::client_hub::ClientHub>) -> Self {
        self.inner.client_hub = hub;
        self
    }
    pub fn build(self) -> ModuleCtx {
        self.inner
    }
}

impl ModuleCtx {
    pub(crate) fn from_token(token: CancellationToken) -> Self {
        Self {
            db: None,
            config_provider: None,
            client_hub: Arc::new(crate::client_hub::ClientHub::default()),
            cancellation_token: token,
            module_name: None,
        }
    }

    /// Scope context to a specific module name (used by the registry).
    pub(crate) fn for_module(mut self, name: &str) -> Self {
        self.module_name = Some(Arc::<str>::from(name));
        self
    }

    // ---- public read-only API for modules ----
    pub fn db(&self) -> Option<&db::DbHandle> {
        self.db.as_deref()
    }

    pub fn db_required(&self) -> db::Result<&db::DbHandle> {
        self.db
            .as_deref()
            .ok_or_else(|| db::DbError::FeatureDisabled("Database not configured for this module"))
    }

    pub fn client_hub(&self) -> Arc<crate::client_hub::ClientHub> {
        self.client_hub.clone()
    }

    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }

    pub fn current_module(&self) -> Option<&str> {
        self.module_name.as_deref()
    }

    /// Deserialize the module's config section into T.
    /// Extracts the 'config' field from the module entry: modules.<name> = { database: ..., config: ... }
    pub fn config<T: DeserializeOwned>(&self) -> anyhow::Result<T> {
        let name = self
            .module_name
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("context is not scoped to a module"))?;

        let prov = self
            .config_provider
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no ConfigProvider"))?;

        let module_raw = prov
            .get_module_config(name)
            .ok_or_else(|| anyhow::anyhow!("missing module config: {name}"))?;

        // Extract config section from: modules.<name> = { database: ..., config: ... }
        let obj = module_raw
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("module config for '{name}' must be an object"))?;

        let config_section = obj
            .get("config")
            .ok_or_else(|| anyhow::anyhow!("missing 'config' section in module '{name}'"))?;

        let config: T = serde_json::from_value(config_section.clone())
            .map_err(|e| anyhow::anyhow!("invalid {name} config section: {}", e))?;

        Ok(config)
    }

    /// Get the raw JSON value of the module's config section.
    /// Returns the 'config' field from: modules.<name> = { database: ..., config: ... }
    pub fn raw_config(&self) -> &serde_json::Value {
        use std::sync::LazyLock;

        static EMPTY: LazyLock<serde_json::Value> =
            LazyLock::new(|| serde_json::Value::Object(serde_json::Map::new()));

        match (&self.module_name, &self.config_provider) {
            (Some(name), Some(prov)) => {
                if let Some(module_raw) = prov.get_module_config(name) {
                    // Try new structure first: modules.<name> = { database: ..., config: ... }
                    if let Some(obj) = module_raw.as_object() {
                        if let Some(config_section) = obj.get("config") {
                            return config_section;
                        }
                    }
                }
                &EMPTY
            }
            _ => &EMPTY,
        }
    }

    /// Create a derivative context with the same references but a different DB handle.
    /// This allows reusing the stable base context while providing per-module DB access.
    pub fn with_db(&self, db: Arc<db::DbHandle>) -> ModuleCtx {
        ModuleCtx {
            db: Some(db),
            config_provider: self.config_provider.clone(),
            client_hub: self.client_hub.clone(),
            cancellation_token: self.cancellation_token.clone(),
            module_name: self.module_name.clone(),
        }
    }

    /// Create a derivative context with the same references but no DB handle.
    /// Useful for modules that don't require database access.
    pub fn without_db(&self) -> ModuleCtx {
        ModuleCtx {
            db: None,
            config_provider: self.config_provider.clone(),
            client_hub: self.client_hub.clone(),
            cancellation_token: self.cancellation_token.clone(),
            module_name: self.module_name.clone(),
        }
    }
}
