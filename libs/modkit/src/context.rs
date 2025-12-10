use serde::de::DeserializeOwned;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

// Import configuration types from the config module
use crate::config::{module_config_or_default, ConfigError, ConfigProvider};

// Note: runtime-dependent features are conditionally compiled

/// Module execution context - the primary interface for modules to access runtime resources.
///
/// This context is passed to all module lifecycle methods (`init`, `register_rest`, etc.)
/// and provides access to:
/// - **Configuration**: Type-safe config loading via `config()`
/// - **Database**: Per-module DB handle via `db_required()` or `db_optional()`
/// - **Service Discovery**: ClientHub for registering/consuming other modules' APIs
/// - **Lifecycle**: Cancellation token for graceful shutdown coordination
///
/// # Lifecycle Flows
///
/// ## Flow A: Module Initialization (`Module::init`)
/// ```ignore
/// async fn init(&self, ctx: &ModuleCtx) -> Result<()> {
///     // 1. Load typed configuration
///     let cfg: MyConfig = ctx.config()?;
///     
///     // 2. Get database handle (fails if not configured)
///     let db = ctx.db_required()?;
///     
///     // 3. Register public API for other modules
///     ctx.client_hub().register(Arc::new(MyService::new(db)));
///     
///     // 4. Consume other modules' APIs
///     let other = ctx.client_hub().get::<dyn OtherApi>()?;
///     Ok(())
/// }
/// ```
///
/// ## Flow B: Background Tasks (`StatefulModule::start`)
/// ```ignore
/// async fn start(&self, cancel: CancellationToken) -> Result<()> {
///     // Use cancellation token from init to coordinate shutdown
///     tokio::select! {
///         _ = self.background_task() => {},
///         _ = cancel.cancelled() => {
///             // Clean shutdown
///         }
///     }
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct ModuleCtx {
    module_name: Arc<str>,
    config_provider: Arc<dyn ConfigProvider>,
    client_hub: Arc<crate::client_hub::ClientHub>,
    cancellation_token: CancellationToken,
    db_handle: Option<Arc<modkit_db::DbHandle>>,
}

/// Factory for creating per-module execution contexts.
///
/// **Internal use only** - created by `HostRuntime` during startup with global singletons.
///
/// # Responsibility
/// Resolves module-specific resources (especially DB handles) at context creation time,
/// ensuring each `ModuleCtx` contains only ready-to-use resources without exposing the
/// underlying `DbManager` to modules.
///
/// # Usage Pattern
/// ```ignore
/// // Runtime creates builder once at startup
/// let builder = ModuleContextBuilder::new(config, client_hub, cancel, db_manager);
///
/// // Before each lifecycle phase, resolve context for that module
/// let ctx = builder.for_module("users_info").await?;
/// module.init(&ctx).await?;
/// ```
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

    /// Resolve a module-scoped context with DB handle (if configured).
    ///
    /// Queries `DbManager` for this module's database configuration and creates
    /// a ready-to-use `ModuleCtx`. If the module has no DB config, `db_optional()` returns `None`.
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

    /// Access the service registry for inter-module communication.
    ///
    /// **Register** your module's public API during `init()` so other modules can discover it:
    /// ```ignore
    /// ctx.client_hub().register(Arc::new(MyApiImpl));
    /// ```
    ///
    /// **Consume** other modules' APIs:
    /// ```ignore
    /// let other_api = ctx.client_hub().get::<dyn OtherModuleApi>()?;
    /// ```
    #[inline]
    pub fn client_hub(&self) -> &crate::client_hub::ClientHub {
        &self.client_hub
    }

    /// Get the cancellation token for graceful shutdown coordination.
    ///
    /// Store this during `init()` and use it in `StatefulModule::start()` to detect when
    /// the runtime is shutting down:
    /// ```ignore
    /// tokio::select! {
    ///     _ = my_loop() => {},
    ///     _ = cancel.cancelled() => { /* cleanup */ }
    /// }
    /// ```
    #[inline]
    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }

    /// Get database handle if configured for this module.
    ///
    /// Returns `None` if:
    /// - Module has no `database` section in config
    /// - Runtime was started with `DbOptions::None`
    pub fn db_optional(&self) -> Option<Arc<modkit_db::DbHandle>> {
        self.db_handle.clone()
    }

    /// Get database handle or fail if not configured.
    ///
    /// Use this in modules that declare `capabilities = [db]` and cannot function without a database.
    /// Returns error if module has no `database` config section or runtime has `DbOptions::None`.
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
    ///
    /// Use this when you need dynamic config inspection or when `config::<T>()` deserialization
    /// is not suitable. Prefer the typed `config()` method for normal use cases.
    ///
    /// Returns the 'config' field from: `modules.<name> = { database: ..., config: ... }`
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

    /// Create a derivative context with a different DB handle.
    ///
    /// Useful for testing or when a module needs to access another module's database.
    /// All other references (config, client_hub, cancellation) are shallow-cloned.
    pub fn with_db(&self, db: Arc<modkit_db::DbHandle>) -> ModuleCtx {
        ModuleCtx {
            module_name: self.module_name.clone(),
            config_provider: self.config_provider.clone(),
            client_hub: self.client_hub.clone(),
            cancellation_token: self.cancellation_token.clone(),
            db_handle: Some(db),
        }
    }

    /// Create a derivative context without a DB handle.
    ///
    /// Primarily used in testing scenarios where database access should be explicitly unavailable.
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
#[cfg_attr(coverage_nightly, coverage(off))]
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
    fn module_ctx_config_with_valid_config() {
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
    fn module_ctx_config_returns_default_for_missing_module() {
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

    // Database Access Flows

    #[test]
    fn db_required_returns_handle_when_present() {
        // Create a minimal mock DB handle - we just need the type, not a real connection
        let db_manager = create_mock_db_manager();
        let provider = Arc::new(MockConfigProvider::new());
        let ctx = ModuleCtx::new(
            "test_module",
            provider,
            Arc::new(crate::client_hub::ClientHub::default()),
            CancellationToken::new(),
            Some(db_manager.clone()),
        );

        let result = ctx.db_required();
        assert!(
            result.is_ok(),
            "db_required should succeed when DB is present"
        );
        assert!(Arc::ptr_eq(&result.unwrap(), &db_manager));
    }

    #[test]
    fn db_required_fails_when_missing() {
        let ctx = ModuleCtx::new(
            "no_db_module",
            Arc::new(MockConfigProvider::new()),
            Arc::new(crate::client_hub::ClientHub::default()),
            CancellationToken::new(),
            None,
        );

        let err = ctx.db_required().unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("Database is not configured"),
            "Error should mention database is not configured, got: {err_msg}"
        );
        assert!(
            err_msg.contains("no_db_module"),
            "Error should mention module name, got: {err_msg}"
        );
    }

    #[test]
    fn db_optional_returns_some_when_present() {
        let db_manager = create_mock_db_manager();
        let ctx = ModuleCtx::new(
            "test_module",
            Arc::new(MockConfigProvider::new()),
            Arc::new(crate::client_hub::ClientHub::default()),
            CancellationToken::new(),
            Some(db_manager.clone()),
        );

        let result = ctx.db_optional();
        assert!(
            result.is_some(),
            "db_optional should return Some when DB is present"
        );
        // Verify it's the same Arc instance
        assert!(Arc::ptr_eq(&result.unwrap(), &db_manager));
    }

    #[test]
    fn db_optional_returns_none_when_missing() {
        let ctx = ModuleCtx::new(
            "test_module",
            Arc::new(MockConfigProvider::new()),
            Arc::new(crate::client_hub::ClientHub::default()),
            CancellationToken::new(),
            None,
        );

        let result = ctx.db_optional();
        assert!(
            result.is_none(),
            "db_optional should return None when DB is absent"
        );
    }

    // Derivative Context Flows

    #[test]
    fn with_db_creates_new_context_with_db() {
        let original_ctx = ModuleCtx::new(
            "test_module",
            Arc::new(MockConfigProvider::new()),
            Arc::new(crate::client_hub::ClientHub::default()),
            CancellationToken::new(),
            None,
        );

        assert!(
            original_ctx.db_optional().is_none(),
            "Original should not have DB"
        );

        let new_db = create_mock_db_manager();
        let new_ctx = original_ctx.with_db(new_db.clone());

        // New context should have the DB
        assert!(
            new_ctx.db_optional().is_some(),
            "New context should have DB"
        );
        assert!(Arc::ptr_eq(&new_ctx.db_optional().unwrap(), &new_db));

        // Original should be unchanged (immutability)
        assert!(
            original_ctx.db_optional().is_none(),
            "Original context should remain unchanged"
        );

        // Verify other fields are preserved
        assert_eq!(new_ctx.module_name(), original_ctx.module_name());
    }

    #[test]
    fn without_db_removes_db_handle() {
        let db_manager = create_mock_db_manager();
        let original_ctx = ModuleCtx::new(
            "test_module",
            Arc::new(MockConfigProvider::new()),
            Arc::new(crate::client_hub::ClientHub::default()),
            CancellationToken::new(),
            Some(db_manager),
        );

        assert!(
            original_ctx.db_optional().is_some(),
            "Original should have DB"
        );

        let new_ctx = original_ctx.without_db();

        // New context should not have DB
        assert!(
            new_ctx.db_optional().is_none(),
            "New context should not have DB"
        );

        // Original should be unchanged (immutability)
        assert!(
            original_ctx.db_optional().is_some(),
            "Original context should remain unchanged"
        );

        // Verify other fields are preserved
        assert_eq!(new_ctx.module_name(), original_ctx.module_name());
    }

    // Context Accessors

    #[test]
    fn module_name_accessor() {
        let ctx = ModuleCtx::new(
            "my_test_module",
            Arc::new(MockConfigProvider::new()),
            Arc::new(crate::client_hub::ClientHub::default()),
            CancellationToken::new(),
            None,
        );

        assert_eq!(ctx.module_name(), "my_test_module");
    }

    #[test]
    fn client_hub_returns_correct_instance() {
        let hub = Arc::new(crate::client_hub::ClientHub::default());
        let hub_ptr = Arc::as_ptr(&hub);

        let ctx = ModuleCtx::new(
            "test_module",
            Arc::new(MockConfigProvider::new()),
            hub.clone(),
            CancellationToken::new(),
            None,
        );

        // Verify we get the same ClientHub instance back
        let returned_hub = ctx.client_hub();
        assert_eq!(returned_hub as *const _ as *const (), hub_ptr as *const ());
    }

    #[test]
    fn cancellation_token_propagation() {
        let token = CancellationToken::new();
        let ctx = ModuleCtx::new(
            "test_module",
            Arc::new(MockConfigProvider::new()),
            Arc::new(crate::client_hub::ClientHub::default()),
            token.clone(),
            None,
        );

        let ctx_token = ctx.cancellation_token();
        assert!(
            !ctx_token.is_cancelled(),
            "Token should not be cancelled initially"
        );

        // Cancel the original token
        token.cancel();

        // Context's token should also be cancelled
        assert!(
            ctx_token.is_cancelled(),
            "Context token should be cancelled when original is cancelled"
        );
    }

    #[test]
    fn current_module_returns_module_name() {
        let ctx = ModuleCtx::new(
            "test_module",
            Arc::new(MockConfigProvider::new()),
            Arc::new(crate::client_hub::ClientHub::default()),
            CancellationToken::new(),
            None,
        );

        assert_eq!(ctx.current_module(), Some("test_module"));
    }

    // ModuleContextBuilder flows

    #[test]
    fn context_builder_without_db_manager_disables_db_access() {
        let config_provider: Arc<dyn ConfigProvider> = Arc::new(MockConfigProvider::new());
        let client_hub = Arc::new(crate::client_hub::ClientHub::default());
        let builder =
            ModuleContextBuilder::new(config_provider, client_hub, CancellationToken::new(), None);

        let rt = tokio::runtime::Runtime::new().unwrap();
        let ctx = rt.block_on(async { builder.for_module("test_module").await.unwrap() });

        assert!(ctx.db_optional().is_none());
    }

    #[test]
    fn context_builder_with_db_manager_resolves_db_handle() {
        let config_provider: Arc<dyn ConfigProvider> = Arc::new(MockConfigProvider::new());
        let client_hub = Arc::new(crate::client_hub::ClientHub::default());
        let db_manager = create_test_db_manager();

        let builder = ModuleContextBuilder::new(
            config_provider,
            client_hub,
            CancellationToken::new(),
            Some(db_manager),
        );

        let rt = tokio::runtime::Runtime::new().unwrap();
        let ctx = rt.block_on(async { builder.for_module("test_module").await.unwrap() });

        let db = ctx
            .db_required()
            .expect("Builder should attach DB handle when manager exists");
        assert_eq!(db.engine(), modkit_db::DbEngine::Sqlite);
        // DSN might be normalized (e.g. "sqlite://:memory:" vs "sqlite::memory:"), so we check key components
        assert_eq!(db.dsn(), "sqlite://:memory:");
    }

    // Raw Config Access

    #[test]
    fn raw_config_returns_config_section() {
        let provider = Arc::new(MockConfigProvider::new());
        let ctx = ModuleCtx::new(
            "test_module",
            provider,
            Arc::new(crate::client_hub::ClientHub::default()),
            CancellationToken::new(),
            None,
        );

        let raw = ctx.raw_config();
        assert!(raw.is_object(), "raw_config should return an object");
        assert_eq!(raw["api_key"], "secret123");
        assert_eq!(raw["timeout_ms"], 5000);
    }

    #[test]
    fn raw_config_returns_empty_for_missing_module() {
        let ctx = ModuleCtx::new(
            "nonexistent",
            Arc::new(MockConfigProvider::new()),
            Arc::new(crate::client_hub::ClientHub::default()),
            CancellationToken::new(),
            None,
        );

        let raw = ctx.raw_config();
        assert!(raw.is_object(), "raw_config should return an empty object");
        assert_eq!(raw.as_object().unwrap().len(), 0);
    }

    // Helper function to create a mock DB handle for testing
    fn create_mock_db_manager() -> Arc<modkit_db::DbHandle> {
        use figment::{providers::Serialized, Figment};

        let figment = Figment::new().merge(Serialized::defaults(json!({
            "modules": {
                "test_module": {
                    "database": {
                        "dsn": "sqlite::memory:",
                        "params": {
                            "journal_mode": "WAL"
                        }
                    }
                }
            }
        })));

        let home_dir = std::path::PathBuf::from("/tmp/test");
        let manager = modkit_db::DbManager::from_figment(figment, home_dir).unwrap();

        // Create a simple blocking runtime for test
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async { manager.get("test_module").await.unwrap().unwrap() })
    }

    fn create_test_db_manager() -> Arc<modkit_db::DbManager> {
        use figment::{providers::Serialized, Figment};

        let figment = Figment::new().merge(Serialized::defaults(json!({
            "modules": {
                "test_module": {
                    "database": {
                        "dsn": "sqlite::memory:"
                    }
                }
            }
        })));

        let home_dir = std::path::PathBuf::from("/tmp/test");
        Arc::new(modkit_db::DbManager::from_figment(figment, home_dir).unwrap())
    }
}
