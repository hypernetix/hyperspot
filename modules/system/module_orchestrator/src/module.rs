//! Module definition for `ModuleOrchestrator`

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

use modkit::DirectoryClient;
use modkit::context::ModuleCtx;
use modkit::contracts::{
    GrpcServiceCapability, OpenApiRegistry, RegisterGrpcServiceFn, RestApiCapability,
    SystemCapability,
};
use modkit::directory::LocalDirectoryClient;
use modkit::registry::ModuleRegistrySnapshot;
use modkit::runtime::ModuleManager;

use cf_system_sdks::directory::DIRECTORY_SERVICE_NAME;

use crate::domain::service::ModulesService;
use crate::server;

/// Configuration for the module orchestrator
#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct ModuleOrchestratorConfig;

/// Module Orchestrator - system module for service discovery
///
/// This module:
/// - Provides `DirectoryClient` to the `ClientHub` for in-process modules
/// - Exposes `DirectoryService` gRPC service via `grpc_hub`
/// - Tracks module instances and provides service resolution
/// - Exposes REST API to list all registered modules
#[modkit::module(
    name = "module_orchestrator",
    capabilities = [grpc, system, rest],
    client = cf_system_sdks::directory::DirectoryClient
)]
pub struct ModuleOrchestrator {
    config: RwLock<ModuleOrchestratorConfig>,
    directory_api: OnceLock<Arc<dyn DirectoryClient>>,
    module_manager: OnceLock<Arc<ModuleManager>>,
    registry_snapshot: OnceLock<Arc<ModuleRegistrySnapshot>>,
    oop_module_names: OnceLock<Arc<HashSet<String>>>,
    modules_service: arc_swap::ArcSwapOption<ModulesService>,
}

impl Default for ModuleOrchestrator {
    fn default() -> Self {
        Self {
            config: RwLock::new(ModuleOrchestratorConfig),
            directory_api: OnceLock::new(),
            module_manager: OnceLock::new(),
            registry_snapshot: OnceLock::new(),
            oop_module_names: OnceLock::new(),
            modules_service: arc_swap::ArcSwapOption::empty(),
        }
    }
}

#[async_trait]
impl SystemCapability for ModuleOrchestrator {
    fn pre_init(&self, sys: &modkit::runtime::SystemContext) -> anyhow::Result<()> {
        self.module_manager
            .set(Arc::clone(&sys.module_manager))
            .map_err(|_| anyhow::anyhow!("ModuleManager already set (pre_init called twice?)"))?;
        self.registry_snapshot
            .set(Arc::clone(&sys.registry_snapshot))
            .map_err(|_| {
                anyhow::anyhow!("RegistrySnapshot already set (pre_init called twice?)")
            })?;
        self.oop_module_names
            .set(Arc::clone(&sys.oop_module_names))
            .map_err(|_| {
                anyhow::anyhow!("OoP module names already set (pre_init called twice?)")
            })?;
        Ok(())
    }
}

#[async_trait]
impl modkit::Module for ModuleOrchestrator {
    async fn init(&self, ctx: &ModuleCtx) -> Result<()> {
        // Load configuration if present
        let cfg = ctx.config::<ModuleOrchestratorConfig>().unwrap_or_default();
        *self.config.write().await = cfg;

        // Use the injected ModuleManager to create the DirectoryClient
        let manager =
            self.module_manager.get().cloned().ok_or_else(|| {
                anyhow::anyhow!("ModuleManager not wired into ModuleOrchestrator")
            })?;

        let api_impl: Arc<dyn DirectoryClient> =
            Arc::new(LocalDirectoryClient::new(manager.clone()));

        // Register in ClientHub directly
        ctx.client_hub()
            .register::<dyn DirectoryClient>(api_impl.clone());

        self.directory_api
            .set(api_impl)
            .map_err(|_| anyhow::anyhow!("DirectoryClient already set (init called twice?)"))?;

        // Create the ModulesService for the REST endpoint
        let registry_snapshot = self
            .registry_snapshot
            .get()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("RegistrySnapshot not wired"))?;
        let oop_module_names = self
            .oop_module_names
            .get()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("OoP module names not wired"))?;

        if self.modules_service.load().is_some() {
            return Err(anyhow::anyhow!(
                "ModulesService already initialized (init called twice?)"
            ));
        }
        let modules_service = ModulesService::new(registry_snapshot, manager, oop_module_names);
        self.modules_service.store(Some(Arc::new(modules_service)));

        tracing::info!("ModuleOrchestrator initialized");

        Ok(())
    }
}

impl RestApiCapability for ModuleOrchestrator {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> Result<axum::Router> {
        let service = self
            .modules_service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("ModulesService not initialized"))?
            .clone();

        let router = crate::api::rest::routes::register_routes(router, openapi, service);

        tracing::info!("ModuleOrchestrator REST routes registered");
        Ok(router)
    }
}

/// Export gRPC services to `grpc_hub`
#[async_trait]
impl GrpcServiceCapability for ModuleOrchestrator {
    async fn get_grpc_services(&self, _ctx: &ModuleCtx) -> Result<Vec<RegisterGrpcServiceFn>> {
        let api = self
            .directory_api
            .get()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("DirectoryClient not initialized"))?;

        // Build DirectoryService
        let directory_svc = server::make_directory_service(api);

        Ok(vec![RegisterGrpcServiceFn {
            service_name: DIRECTORY_SERVICE_NAME,
            register: Box::new(move |routes| {
                routes.add_service(directory_svc.clone());
            }),
        }])
    }
}
