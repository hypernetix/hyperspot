#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Comprehensive tests for the ModKit runner functionality
//!
//! Tests the core orchestration logic including lifecycle phases,
//! database strategies, shutdown options, and error handling.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

use modkit::{
    client_hub::ClientHub,
    config::ConfigProvider,
    contracts::{DbModule, Module, OpenApiRegistry, RestHostModule, RestfulModule, StatefulModule},
    registry::{ModuleRegistry, RegistryBuilder},
    runtime::{run, DbOptions, HostRuntime, RunOptions, ShutdownOptions},
    ModuleCtx,
};

// Probe state for runtime lifecycle test (inventory-discovered module)
#[derive(Debug, Default)]
struct ProbeState {
    init: AtomicBool,
    start: AtomicBool,
    stop: AtomicBool,
}

fn probe_state() -> Arc<ProbeState> {
    static STATE: std::sync::OnceLock<Arc<ProbeState>> = std::sync::OnceLock::new();
    STATE
        .get_or_init(|| Arc::new(ProbeState::default()))
        .clone()
}

#[derive(Clone)]
#[modkit::module(name = "runtime_lifecycle_probe", capabilities = [stateful])]
pub struct RuntimeLifecycleProbe {
    state: Arc<ProbeState>,
}

impl Default for RuntimeLifecycleProbe {
    fn default() -> Self {
        Self {
            state: probe_state(),
        }
    }
}

#[async_trait::async_trait]
impl Module for RuntimeLifecycleProbe {
    async fn init(&self, _ctx: &ModuleCtx) -> anyhow::Result<()> {
        self.state.init.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait::async_trait]
impl StatefulModule for RuntimeLifecycleProbe {
    async fn start(&self, cancel: CancellationToken) -> anyhow::Result<()> {
        self.state.start.store(true, Ordering::SeqCst);
        // Wait until the test cancels to exercise wait -> stop
        cancel.cancelled().await;
        Ok(())
    }

    async fn stop(&self, _cancel: CancellationToken) -> anyhow::Result<()> {
        self.state.stop.store(true, Ordering::SeqCst);
        Ok(())
    }
}

// Test tracking infrastructure
#[allow(dead_code)]
type CallTracker = Arc<Mutex<Vec<String>>>;

#[derive(Default)]
#[allow(dead_code)]
struct TestOpenApiRegistry;

impl OpenApiRegistry for TestOpenApiRegistry {
    fn register_operation(&self, _spec: &modkit::api::OperationSpec) {}
    fn ensure_schema_raw(
        &self,
        root_name: &str,
        _schemas: Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) -> String {
        root_name.to_string()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Mock config provider for testing
#[derive(Clone)]
struct MockConfigProvider {
    configs: std::collections::HashMap<String, serde_json::Value>,
}

impl MockConfigProvider {
    fn new() -> Self {
        Self {
            configs: std::collections::HashMap::new(),
        }
    }

    fn with_config(mut self, module_name: &str, config: serde_json::Value) -> Self {
        self.configs.insert(module_name.to_string(), config);
        self
    }
}

impl ConfigProvider for MockConfigProvider {
    fn get_module_config(&self, module_name: &str) -> Option<&serde_json::Value> {
        self.configs.get(module_name)
    }
}

// Test trait to add pipe method for more readable code
#[allow(dead_code)]
trait Pipe<T> {
    fn pipe<U, F: FnOnce(T) -> U>(self, f: F) -> U;
}

impl<T> Pipe<T> for T {
    fn pipe<U, F: FnOnce(T) -> U>(self, f: F) -> U {
        f(self)
    }
}

// Test module implementations with lifecycle tracking
#[allow(dead_code)]
#[derive(Clone)]
struct TestModule {
    name: String,
    calls: CallTracker,
    should_fail_init: Arc<AtomicBool>,
    should_fail_db: Arc<AtomicBool>,
    should_fail_rest: Arc<AtomicBool>,
    should_fail_start: Arc<AtomicBool>,
    should_fail_stop: Arc<AtomicBool>,
}

#[allow(dead_code)]
impl TestModule {
    fn new(name: &str, calls: CallTracker) -> Self {
        Self {
            name: name.to_string(),
            calls,
            should_fail_init: Arc::new(AtomicBool::new(false)),
            should_fail_db: Arc::new(AtomicBool::new(false)),
            should_fail_rest: Arc::new(AtomicBool::new(false)),
            should_fail_start: Arc::new(AtomicBool::new(false)),
            should_fail_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    fn fail_init(self) -> Self {
        self.should_fail_init.store(true, Ordering::SeqCst);
        self
    }

    fn fail_db(self) -> Self {
        self.should_fail_db.store(true, Ordering::SeqCst);
        self
    }

    fn fail_rest(self) -> Self {
        self.should_fail_rest.store(true, Ordering::SeqCst);
        self
    }

    fn fail_start(self) -> Self {
        self.should_fail_start.store(true, Ordering::SeqCst);
        self
    }

    fn fail_stop(self) -> Self {
        self.should_fail_stop.store(true, Ordering::SeqCst);
        self
    }
}

#[async_trait::async_trait]
impl Module for TestModule {
    async fn init(&self, _ctx: &ModuleCtx) -> anyhow::Result<()> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("{}.init", self.name));
        if self.should_fail_init.load(Ordering::SeqCst) {
            anyhow::bail!("Init failed for module {}", self.name);
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait::async_trait]
impl DbModule for TestModule {
    async fn migrate(&self, _db: &modkit_db::DbHandle) -> anyhow::Result<()> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("{}.migrate", self.name));
        if self.should_fail_db.load(Ordering::SeqCst) {
            anyhow::bail!("DB migration failed for module {}", self.name);
        }
        Ok(())
    }
}

impl RestfulModule for TestModule {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        _openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("{}.register_rest", self.name));
        if self.should_fail_rest.load(Ordering::SeqCst) {
            anyhow::bail!("REST registration failed for module {}", self.name);
        }
        Ok(router)
    }
}

#[async_trait::async_trait]
impl StatefulModule for TestModule {
    async fn start(&self, _cancel: CancellationToken) -> anyhow::Result<()> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("{}.start", self.name));
        if self.should_fail_start.load(Ordering::SeqCst) {
            anyhow::bail!("Start failed for module {}", self.name);
        }
        Ok(())
    }

    async fn stop(&self, _cancel: CancellationToken) -> anyhow::Result<()> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("{}.stop", self.name));
        if self.should_fail_stop.load(Ordering::SeqCst) {
            anyhow::bail!("Stop failed for module {}", self.name);
        }
        Ok(())
    }
}

// Helper to create a registry with test modules
#[allow(dead_code)]
fn create_test_registry(modules: Vec<TestModule>) -> anyhow::Result<ModuleRegistry> {
    let mut builder = RegistryBuilder::default();

    for module in modules {
        let module_name = module.name.clone();
        let module_name_str: &'static str = Box::leak(module_name.into_boxed_str());
        let module = Arc::new(module);

        builder.register_core_with_meta(module_name_str, &[], module.clone() as Arc<dyn Module>);
        builder.register_db_with_meta(module_name_str, module.clone() as Arc<dyn DbModule>);
        builder.register_rest_with_meta(module_name_str, module.clone() as Arc<dyn RestfulModule>);
        builder.register_stateful_with_meta(
            module_name_str,
            module.clone() as Arc<dyn StatefulModule>,
        );
    }

    Ok(builder.build_topo_sorted()?)
}

// Helper to create a registry with test modules without REST capability
fn create_test_registry_no_rest(modules: Vec<TestModule>) -> anyhow::Result<ModuleRegistry> {
    let mut builder = RegistryBuilder::default();

    for module in modules {
        let module_name = module.name.clone();
        let module_name_str: &'static str = Box::leak(module_name.into_boxed_str());
        let module = Arc::new(module);

        builder.register_core_with_meta(module_name_str, &[], module.clone() as Arc<dyn Module>);
        builder.register_db_with_meta(module_name_str, module.clone() as Arc<dyn DbModule>);
        builder.register_stateful_with_meta(
            module_name_str,
            module.clone() as Arc<dyn StatefulModule>,
        );
    }

    Ok(builder.build_topo_sorted()?)
}

// Helper function to create a mock DbManager for testing
fn create_mock_db_manager() -> Arc<modkit_db::DbManager> {
    use figment::{providers::Serialized, Figment};

    // Create a simple figment with mock database configuration
    let figment = Figment::new().merge(Serialized::defaults(serde_json::json!({
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

    Arc::new(modkit_db::DbManager::from_figment(figment, home_dir).unwrap())
}

#[tokio::test]
async fn shutdown_options_token() {
    let cancel = CancellationToken::new();

    let opts = RunOptions {
        modules_cfg: Arc::new(MockConfigProvider::new()),
        db: DbOptions::None,
        shutdown: ShutdownOptions::Token(cancel.clone()),
    };

    // Start the runner in a background task
    let runner_handle = tokio::spawn(run(opts));

    // Give it a moment to start
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Cancel it
    cancel.cancel();

    // Should complete quickly
    let result = timeout(Duration::from_millis(100), runner_handle).await;
    assert!(result.is_ok());
    let run_result = result.unwrap().unwrap();
    assert!(run_result.is_ok());
}

#[tokio::test]
async fn shutdown_options_future() {
    let (tx, rx) = tokio::sync::oneshot::channel();

    let opts = RunOptions {
        modules_cfg: Arc::new(MockConfigProvider::new()),
        db: DbOptions::None,
        shutdown: ShutdownOptions::Future(Box::pin(async move {
            let _ = rx.await;
        })),
    };

    // Start the runner in a background task
    let runner_handle = tokio::spawn(run(opts));

    // Give it a moment to start
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Trigger shutdown via the future
    let _ = tx.send(());

    // Should complete quickly
    let result = timeout(Duration::from_millis(100), runner_handle).await;
    assert!(result.is_ok());
    let run_result = result.unwrap().unwrap();
    assert!(run_result.is_ok());
}

#[tokio::test]
async fn runner_with_config_provider() {
    let cancel = CancellationToken::new();
    cancel.cancel(); // Immediate shutdown

    let config_provider = MockConfigProvider::new().with_config(
        "test_module",
        serde_json::json!({
            "setting1": "value1",
            "setting2": 42
        }),
    );

    let opts = RunOptions {
        modules_cfg: Arc::new(config_provider),
        db: DbOptions::None,
        shutdown: ShutdownOptions::Token(cancel),
    };

    let result = timeout(Duration::from_millis(100), run(opts)).await;
    assert!(result.is_ok());
}

// Integration test for complete lifecycle (will work once we have proper module discovery mock)
#[tokio::test]
async fn complete_lifecycle_success() {
    // This test is a placeholder for when we can properly mock the module discovery
    // For now, we test that the runner doesn't panic with minimal setup
    let cancel = CancellationToken::new();
    cancel.cancel(); // Immediate shutdown

    let opts = RunOptions {
        modules_cfg: Arc::new(MockConfigProvider::new()),
        db: DbOptions::None,
        shutdown: ShutdownOptions::Token(cancel),
    };

    let result = run(opts).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn cancellation_during_startup() {
    let cancel = CancellationToken::new();

    let opts = RunOptions {
        modules_cfg: Arc::new(MockConfigProvider::new()),
        db: DbOptions::None,
        shutdown: ShutdownOptions::Token(cancel.clone()),
    };

    // Start the runner in a background task
    let runner_handle = tokio::spawn(run(opts));

    // Cancel immediately to test cancellation handling
    cancel.cancel();

    // Should complete quickly due to cancellation
    let result = timeout(Duration::from_millis(100), runner_handle).await;
    assert!(
        result.is_ok(),
        "Runner should complete quickly when cancelled"
    );

    let run_result = result.unwrap().unwrap();
    assert!(
        run_result.is_ok(),
        "Runner should handle cancellation gracefully"
    );
}

#[tokio::test]
async fn multiple_config_provider_scenarios() {
    let cancel = CancellationToken::new();
    cancel.cancel(); // Immediate shutdown

    // Test with empty config
    let empty_config = MockConfigProvider::new();
    let opts = RunOptions {
        modules_cfg: Arc::new(empty_config),
        db: DbOptions::None,
        shutdown: ShutdownOptions::Token(cancel.clone()),
    };

    let result = run(opts).await;
    assert!(result.is_ok(), "Should handle empty config");

    // Test with complex config
    let complex_config = MockConfigProvider::new()
        .with_config(
            "module1",
            serde_json::json!({
                "setting1": "value1",
                "nested": {
                    "setting2": 42,
                    "setting3": true
                }
            }),
        )
        .with_config(
            "module2",
            serde_json::json!({
                "array_setting": [1, 2, 3],
                "string_setting": "test"
            }),
        );

    let cancel2 = CancellationToken::new();
    cancel2.cancel();

    let opts2 = RunOptions {
        modules_cfg: Arc::new(complex_config),
        db: DbOptions::None,
        shutdown: ShutdownOptions::Token(cancel2),
    };

    let result2 = run(opts2).await;
    assert!(result2.is_ok(), "Should handle complex config");
}

#[tokio::test]
async fn db_options_none_skips_migrations() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let test_module = TestModule::new("test_module", calls.clone());

    let registry = create_test_registry_no_rest(vec![test_module]).unwrap();

    let cancel = CancellationToken::new();
    let host = HostRuntime::new(
        registry,
        Arc::new(MockConfigProvider::new()),
        DbOptions::None,
        Arc::new(ClientHub::default()),
        cancel.clone(),
    );

    // Cancel immediately to prevent waiting
    cancel.cancel();

    let result = host.run_full_cycle().await;
    assert!(
        result.is_ok(),
        "Lifecycle should complete successfully: {:?}",
        result.err()
    );

    let call_log = calls.lock().unwrap();

    assert!(
        call_log.contains(&"test_module.init".to_string()),
        "Module init should be called even with DbOptions::None"
    );
    assert!(
        !call_log.contains(&"test_module.migrate".to_string()),
        "Module migrate should NOT be called with DbOptions::None"
    );
    assert!(
        call_log.contains(&"test_module.start".to_string()),
        "Module start should be called"
    );
}

#[tokio::test]
async fn db_options_manager_calls_migrations() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let test_module = TestModule::new("test_module", calls.clone());

    let registry = create_test_registry_no_rest(vec![test_module]).unwrap();

    let config_provider = MockConfigProvider::new().with_config(
        "test_module",
        serde_json::json!({
            "database": {
                "dsn": "sqlite::memory:",
                "params": {
                    "journal_mode": "WAL"
                }
            }
        }),
    );

    let cancel = CancellationToken::new();
    let db_manager = create_mock_db_manager();
    let host = HostRuntime::new(
        registry,
        Arc::new(config_provider),
        DbOptions::Manager(db_manager),
        Arc::new(ClientHub::default()),
        cancel.clone(),
    );

    // Cancel immediately to prevent waiting
    cancel.cancel();

    let result = host.run_full_cycle().await;
    assert!(
        result.is_ok(),
        "Lifecycle should complete successfully: {:?}",
        result.err()
    );

    let call_log = calls.lock().unwrap();
    assert!(
        call_log.contains(&"test_module.init".to_string()),
        "Module init should be called"
    );
    assert!(
        call_log.contains(&"test_module.migrate".to_string()),
        "Module migrate SHOULD be called with DbOptions::Manager"
    );
    assert!(
        call_log.contains(&"test_module.start".to_string()),
        "Module start should be called"
    );
}

#[tokio::test]
async fn db_options_manager_skips_modules_without_db_config() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let test_module = TestModule::new("module_without_db_config", calls.clone());

    let registry = create_test_registry_no_rest(vec![test_module]).unwrap();

    let cancel = CancellationToken::new();
    let host = HostRuntime::new(
        registry,
        Arc::new(MockConfigProvider::new()),
        DbOptions::Manager(create_mock_db_manager()),
        Arc::new(ClientHub::default()),
        cancel.clone(),
    );

    cancel.cancel();

    let result = host.run_full_cycle().await;
    assert!(
        result.is_ok(),
        "Lifecycle should complete successfully when DB config is missing: {:?}",
        result.err()
    );

    let call_log = calls.lock().unwrap();
    assert!(
        call_log.contains(&"module_without_db_config.init".to_string()),
        "Module init should run even when db config is absent"
    );
    assert!(
        !call_log.contains(&"module_without_db_config.migrate".to_string()),
        "Module migrate should be skipped when no db config exists"
    );
    assert!(
        call_log.contains(&"module_without_db_config.start".to_string()),
        "Module start should still execute"
    );
}

#[tokio::test]
async fn db_options_manager_propagates_migration_errors() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let test_module = TestModule::new("test_module", calls.clone()).fail_db();

    let registry = create_test_registry_no_rest(vec![test_module]).unwrap();

    let config_provider = MockConfigProvider::new().with_config(
        "test_module",
        serde_json::json!({
            "database": {
                "dsn": "sqlite::memory:",
                "params": {
                    "journal_mode": "WAL"
                }
            }
        }),
    );

    let host = HostRuntime::new(
        registry,
        Arc::new(config_provider),
        DbOptions::Manager(create_mock_db_manager()),
        Arc::new(ClientHub::default()),
        CancellationToken::new(),
    );

    let err = host
        .run_full_cycle()
        .await
        .expect_err("Migration failure should propagate to caller");

    let call_log = calls.lock().unwrap();
    assert!(
        call_log.contains(&"test_module.migrate".to_string()),
        "Migration attempt should be recorded"
    );
    assert!(
        !call_log.contains(&"test_module.start".to_string()),
        "Start phase must not run after migration failure"
    );

    let err_str = format!("{err:#?}");
    assert!(
        err_str.contains("test_module"),
        "Error should mention the failing module: {err_str}"
    );
}

// Mock REST host module for testing REST phase
#[derive(Clone)]
struct MockRestHost {
    calls: CallTracker,
}

impl MockRestHost {
    fn new(calls: CallTracker) -> Self {
        Self { calls }
    }
}

impl RestHostModule for MockRestHost {
    fn rest_prepare(&self, _ctx: &ModuleCtx, router: axum::Router) -> anyhow::Result<axum::Router> {
        self.calls
            .lock()
            .unwrap()
            .push("rest_host.prepare".to_string());
        Ok(router)
    }

    fn rest_finalize(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
    ) -> anyhow::Result<axum::Router> {
        self.calls
            .lock()
            .unwrap()
            .push("rest_host.finalize".to_string());
        Ok(router)
    }

    fn as_registry(&self) -> &dyn OpenApiRegistry {
        &TestOpenApiRegistry
    }
}

// Helper to create a registry with REST modules and a REST host
fn create_test_registry_with_rest_host(
    modules: Vec<TestModule>,
    host: MockRestHost,
) -> anyhow::Result<ModuleRegistry> {
    let mut builder = RegistryBuilder::default();

    let host_name: &'static str = Box::leak("rest_host".to_string().into_boxed_str());
    let host_arc = Arc::new(host);
    #[derive(Clone)]
    struct RestHostAsModule {
        calls: CallTracker,
    }

    #[async_trait::async_trait]
    impl Module for RestHostAsModule {
        async fn init(&self, _ctx: &ModuleCtx) -> anyhow::Result<()> {
            self.calls
                .lock()
                .unwrap()
                .push("rest_host.init".to_string());
            Ok(())
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    let host_core = Arc::new(RestHostAsModule {
        calls: host_arc.calls.clone(),
    });
    builder.register_core_with_meta(host_name, &[], host_core);
    builder.register_rest_host_with_meta(host_name, host_arc);

    for module in modules {
        let module_name = module.name.clone();
        let module_name_str: &'static str = Box::leak(module_name.into_boxed_str());
        let module = Arc::new(module);

        builder.register_core_with_meta(module_name_str, &[], module.clone() as Arc<dyn Module>);
        builder.register_rest_with_meta(module_name_str, module.clone() as Arc<dyn RestfulModule>);
        builder.register_stateful_with_meta(
            module_name_str,
            module.clone() as Arc<dyn StatefulModule>,
        );
    }

    Ok(builder.build_topo_sorted()?)
}

#[tokio::test]
async fn rest_phase_with_rest_modules() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let test_module = TestModule::new("test_module", calls.clone());
    let rest_host = MockRestHost::new(calls.clone());

    let registry = create_test_registry_with_rest_host(vec![test_module], rest_host).unwrap();

    let cancel = CancellationToken::new();
    let host = HostRuntime::new(
        registry,
        Arc::new(MockConfigProvider::new()),
        DbOptions::None,
        Arc::new(ClientHub::default()),
        cancel.clone(),
    );

    // Cancel immediately to prevent waiting
    cancel.cancel();

    let result = host.run_full_cycle().await;
    assert!(
        result.is_ok(),
        "Lifecycle should complete successfully: {:?}",
        result.err()
    );

    let call_log = calls.lock().unwrap();

    assert!(
        call_log.contains(&"rest_host.init".to_string()),
        "REST host init should be called"
    );
    assert!(
        call_log.contains(&"rest_host.prepare".to_string()),
        "REST host prepare should be called"
    );
    assert!(
        call_log.contains(&"test_module.register_rest".to_string()),
        "Module register_rest should be called"
    );
    assert!(
        call_log.contains(&"rest_host.finalize".to_string()),
        "REST host finalize should be called"
    );

    // Verify order: prepare -> register -> finalize
    let prepare_idx = call_log
        .iter()
        .position(|s| s == "rest_host.prepare")
        .unwrap();
    let register_idx = call_log
        .iter()
        .position(|s| s == "test_module.register_rest")
        .unwrap();
    let finalize_idx = call_log
        .iter()
        .position(|s| s == "rest_host.finalize")
        .unwrap();

    assert!(
        prepare_idx < register_idx,
        "REST host prepare should come before module registration"
    );
    assert!(
        register_idx < finalize_idx,
        "Module registration should come before REST host finalize"
    );
}

// Mock gRPC service module for testing gRPC phase
#[derive(Clone)]
struct MockGrpcServiceModule {
    calls: CallTracker,
    service_name: &'static str,
}

impl MockGrpcServiceModule {
    fn new(calls: CallTracker, service_name: &'static str) -> Self {
        Self {
            calls,
            service_name,
        }
    }
}

#[async_trait::async_trait]
impl modkit::contracts::GrpcServiceModule for MockGrpcServiceModule {
    async fn get_grpc_services(
        &self,
        _ctx: &ModuleCtx,
    ) -> anyhow::Result<Vec<modkit::contracts::RegisterGrpcServiceFn>> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("{}.get_grpc_services", self.service_name));

        Ok(vec![modkit::contracts::RegisterGrpcServiceFn {
            service_name: self.service_name,
            register: Box::new(|_routes| {
                // Mock registration - no actual service added
            }),
        }])
    }
}

// Helper to create a registry with gRPC modules
fn create_test_registry_with_grpc(
    grpc_modules: Vec<(&'static str, MockGrpcServiceModule)>,
) -> anyhow::Result<ModuleRegistry> {
    let mut builder = RegistryBuilder::default();

    // Register a mock grpc_hub
    let hub_name: &'static str = "grpc_hub";

    #[derive(Clone)]
    struct MockGrpcHub;

    #[async_trait::async_trait]
    impl Module for MockGrpcHub {
        async fn init(&self, _ctx: &ModuleCtx) -> anyhow::Result<()> {
            Ok(())
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    impl modkit::contracts::GrpcHubModule for MockGrpcHub {}

    let hub_core = Arc::new(MockGrpcHub);
    builder.register_core_with_meta(hub_name, &[], hub_core);
    builder.register_grpc_hub_with_meta(hub_name);

    // Register grpc service modules
    for (module_name, grpc_module) in grpc_modules {
        #[derive(Clone)]
        struct GrpcModuleCore;

        #[async_trait::async_trait]
        impl Module for GrpcModuleCore {
            async fn init(&self, _ctx: &ModuleCtx) -> anyhow::Result<()> {
                Ok(())
            }
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }

        let core = Arc::new(GrpcModuleCore);

        builder.register_core_with_meta(module_name, &[], core);
        builder.register_grpc_service_with_meta(module_name, Arc::new(grpc_module));
    }

    Ok(builder.build_topo_sorted()?)
}

#[tokio::test]
async fn grpc_phase_with_grpc_modules() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let grpc_module_a = MockGrpcServiceModule::new(calls.clone(), "test.ServiceA");
    let grpc_module_b = MockGrpcServiceModule::new(calls.clone(), "test.ServiceB");

    let registry = create_test_registry_with_grpc(vec![
        ("grpc_module_a", grpc_module_a),
        ("grpc_module_b", grpc_module_b),
    ])
    .unwrap();

    let cancel = CancellationToken::new();
    let host = HostRuntime::new(
        registry,
        Arc::new(MockConfigProvider::new()),
        DbOptions::None,
        Arc::new(ClientHub::default()),
        cancel.clone(),
    );

    // Cancel immediately to prevent waiting
    cancel.cancel();

    let result = host.run_full_cycle().await;
    assert!(
        result.is_ok(),
        "Lifecycle should complete successfully: {:?}",
        result.err()
    );

    let call_log = calls.lock().unwrap();

    assert!(
        call_log.contains(&"test.ServiceA.get_grpc_services".to_string()),
        "Module A get_grpc_services should be called"
    );
    assert!(
        call_log.contains(&"test.ServiceB.get_grpc_services".to_string()),
        "Module B get_grpc_services should be called"
    );
}

#[tokio::test]
async fn runner_timeout_scenarios() {
    // Test that runner doesn't hang indefinitely
    let cancel = CancellationToken::new();

    let opts = RunOptions {
        modules_cfg: Arc::new(MockConfigProvider::new()),
        db: DbOptions::None,
        shutdown: ShutdownOptions::Token(cancel.clone()),
    };

    let runner_handle = tokio::spawn(run(opts));

    // Give it some time to start up
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Cancel after a short delay
    cancel.cancel();

    // Should complete within a reasonable time
    let result = timeout(Duration::from_millis(200), runner_handle).await;
    assert!(result.is_ok(), "Runner should complete within timeout");

    let run_result = result.unwrap().unwrap();
    assert!(run_result.is_ok(), "Runner should complete successfully");
}

// Test configuration scenarios
#[test]
fn config_provider_edge_cases() {
    let provider = MockConfigProvider::new()
        .with_config("test", serde_json::json!(null))
        .with_config("empty", serde_json::json!({}))
        .with_config(
            "complex",
            serde_json::json!({
                "a": {
                    "b": {
                        "c": "deep_value"
                    }
                }
            }),
        );

    // Test null config
    let null_config = provider.get_module_config("test");
    assert!(null_config.is_some());
    assert!(null_config.unwrap().is_null());

    // Test empty config
    let empty_config = provider.get_module_config("empty");
    assert!(empty_config.is_some());
    assert!(empty_config.unwrap().is_object());

    // Test complex config
    let complex_config = provider.get_module_config("complex");
    assert!(complex_config.is_some());
    assert!(complex_config.unwrap()["a"]["b"]["c"] == "deep_value");

    // Test non-existent config
    let missing_config = provider.get_module_config("nonexistent");
    assert!(missing_config.is_none());
}

#[tokio::test]
async fn run_drives_full_lifecycle_for_stateful_module() {
    let state = probe_state();
    state.init.store(false, Ordering::SeqCst);
    state.start.store(false, Ordering::SeqCst);
    state.stop.store(false, Ordering::SeqCst);

    let cancel = CancellationToken::new();

    let opts = RunOptions {
        modules_cfg: Arc::new(MockConfigProvider::new()),
        db: DbOptions::None,
        shutdown: ShutdownOptions::Token(cancel.clone()),
    };

    let runner = tokio::spawn(run(opts));

    let test_timeout = Duration::from_millis(200);

    let started = timeout(test_timeout, async {
        while !state.start.load(Ordering::SeqCst) {
            tokio::task::yield_now().await;
        }
    })
    .await;

    assert!(started.is_ok(), "start should be observed");

    // Trigger shutdown and wait for completion
    cancel.cancel();

    let run_result = timeout(Duration::from_secs(2), runner)
        .await
        .expect("runner should finish")
        .expect("runner task should not panic");

    assert!(run_result.is_ok(), "run should complete successfully");

    // THEN: all lifecycle flags should be flipped
    assert!(state.init.load(Ordering::SeqCst), "init should run");
    assert!(state.start.load(Ordering::SeqCst), "start should run");
    assert!(state.stop.load(Ordering::SeqCst), "stop should run");
}
