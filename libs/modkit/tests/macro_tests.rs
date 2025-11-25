//! Comprehensive tests for the #[module] macro with the new registry/builder

use anyhow::Result;
use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

use modkit::{
    config::ConfigProvider,
    contracts::{DbModule, Module, OpenApiRegistry, RestHostModule, RestfulModule, StatefulModule},
    module, ModuleCtx,
};
use std::sync::Arc;

// Helper for tests
struct EmptyConfigProvider;
impl ConfigProvider for EmptyConfigProvider {
    fn get_module_config(&self, _module_name: &str) -> Option<&serde_json::Value> {
        None
    }
}

fn test_module_ctx(cancel: tokio_util::sync::CancellationToken) -> ModuleCtx {
    ModuleCtx::new(
        "test",
        Arc::new(EmptyConfigProvider),
        Arc::new(modkit::client_hub::ClientHub::default()),
        cancel,
        None,
    )
}

/// Minimal OpenAPI registry mock
#[derive(Default)]
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

// ---------- Test modules (must be at module scope for `inventory`) ----------

#[derive(Default)]
#[module(name = "basic")]
struct BasicModule;

#[async_trait]
impl Module for BasicModule {
    async fn init(&self, _ctx: &modkit::context::ModuleCtx) -> Result<()> {
        Ok(())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Default)]
#[module(name = "full_featured", capabilities = [db, rest, stateful])]
struct FullFeaturedModule;

#[async_trait]
impl Module for FullFeaturedModule {
    async fn init(&self, _ctx: &modkit::context::ModuleCtx) -> Result<()> {
        Ok(())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
#[async_trait]
impl DbModule for FullFeaturedModule {
    async fn migrate(&self, _db: &modkit_db::DbHandle) -> Result<()> {
        Ok(())
    }
}
impl RestfulModule for FullFeaturedModule {
    fn register_rest(
        &self,
        _ctx: &modkit::context::ModuleCtx,
        router: axum::Router,
        _openapi: &dyn OpenApiRegistry,
    ) -> Result<axum::Router> {
        Ok(router)
    }
}
#[async_trait]
impl StatefulModule for FullFeaturedModule {
    async fn start(&self, _t: CancellationToken) -> Result<()> {
        Ok(())
    }
    async fn stop(&self, _t: CancellationToken) -> Result<()> {
        Ok(())
    }
}

#[derive(Default)]
#[module(name = "dependent", deps = ["basic", "full_featured"])]
struct DependentModule;

#[async_trait]
impl Module for DependentModule {
    async fn init(&self, _ctx: &modkit::context::ModuleCtx) -> Result<()> {
        Ok(())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Default)]
#[module(name = "custom_ctor", ctor = CustomCtorModule::create())]
struct CustomCtorModule {
    value: i32,
}

impl CustomCtorModule {
    fn create() -> Self {
        Self { value: 42 }
    }
}

#[async_trait]
impl Module for CustomCtorModule {
    async fn init(&self, _ctx: &modkit::context::ModuleCtx) -> Result<()> {
        Ok(())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Default)]
#[module(name = "db_only", capabilities = [db])]
struct DbOnlyModule;
#[async_trait]
impl Module for DbOnlyModule {
    async fn init(&self, _ctx: &modkit::context::ModuleCtx) -> Result<()> {
        Ok(())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
#[async_trait]
impl DbModule for DbOnlyModule {
    async fn migrate(&self, _db: &modkit_db::DbHandle) -> Result<()> {
        Ok(())
    }
}

#[derive(Default)]
#[module(name = "rest_only", capabilities = [rest])]
struct RestOnlyModule;
#[async_trait]
impl Module for RestOnlyModule {
    async fn init(&self, _ctx: &modkit::context::ModuleCtx) -> Result<()> {
        Ok(())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl RestfulModule for RestOnlyModule {
    fn register_rest(
        &self,
        _ctx: &modkit::context::ModuleCtx,
        router: axum::Router,
        _openapi: &dyn OpenApiRegistry,
    ) -> Result<axum::Router> {
        Ok(router)
    }
}

#[derive(Default)]
#[module(name = "rest_host", capabilities = [rest_host])]
struct TestRestHostModule {
    registry: TestOpenApiRegistry,
}

#[async_trait]
impl Module for TestRestHostModule {
    async fn init(&self, _ctx: &modkit::context::ModuleCtx) -> Result<()> {
        Ok(())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl RestHostModule for TestRestHostModule {
    fn rest_prepare(
        &self,
        _ctx: &modkit::context::ModuleCtx,
        router: axum::Router,
    ) -> anyhow::Result<axum::Router> {
        Ok(router)
    }

    fn rest_finalize(
        &self,
        _ctx: &modkit::context::ModuleCtx,
        router: axum::Router,
    ) -> anyhow::Result<axum::Router> {
        Ok(router)
    }

    fn as_registry(&self) -> &dyn OpenApiRegistry {
        &self.registry
    }
}

#[derive(Default)]
#[module(name = "stateful_only", capabilities = [stateful])]
struct StatefulOnlyModule;
#[async_trait]
impl Module for StatefulOnlyModule {
    async fn init(&self, _ctx: &modkit::context::ModuleCtx) -> Result<()> {
        Ok(())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
#[async_trait]
impl StatefulModule for StatefulOnlyModule {
    async fn start(&self, _t: CancellationToken) -> Result<()> {
        Ok(())
    }
    async fn stop(&self, _t: CancellationToken) -> Result<()> {
        Ok(())
    }
}

// ---------- Tests ----------

#[tokio::test]
async fn test_basic_macro_and_init() {
    assert_eq!(BasicModule::MODULE_NAME, "basic");
    let ctx = test_module_ctx(CancellationToken::new());
    BasicModule.init(&ctx).await.unwrap();
}

#[tokio::test]
async fn test_custom_ctor_name_and_value() {
    assert_eq!(CustomCtorModule::MODULE_NAME, "custom_ctor");
    let m = CustomCtorModule::create();
    assert_eq!(m.value, 42);
}

#[tokio::test]
async fn test_full_capabilities() {
    assert_eq!(FullFeaturedModule::MODULE_NAME, "full_featured");

    let ctx = test_module_ctx(CancellationToken::new());
    FullFeaturedModule.init(&ctx).await.unwrap();

    // REST sync phase
    let router = axum::Router::new();
    let oas = TestOpenApiRegistry;
    let _router = FullFeaturedModule
        .register_rest(&ctx, router, &oas)
        .unwrap();

    // Stateful
    let token = CancellationToken::new();
    FullFeaturedModule.start(token.clone()).await.unwrap();
    FullFeaturedModule.stop(token).await.unwrap();
}

#[test]
fn test_capability_trait_markers() {
    fn assert_module<T: Module>(_: &T) {}
    fn assert_db<T: DbModule>(_: &T) {}
    fn assert_rest<T: RestfulModule>(_: &T) {}
    fn assert_stateful<T: StatefulModule>(_: &T) {}

    assert_module(&BasicModule);
    assert_module(&DependentModule);
    assert_module(&CustomCtorModule::default());

    assert_db(&FullFeaturedModule);
    assert_db(&DbOnlyModule);

    assert_rest(&FullFeaturedModule);
    assert_rest(&RestOnlyModule);

    assert_stateful(&FullFeaturedModule);
    assert_stateful(&StatefulOnlyModule);
}
