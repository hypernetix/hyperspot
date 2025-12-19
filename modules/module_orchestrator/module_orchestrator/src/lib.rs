//! Module Orchestrator
//!
//! System module for service discovery.
//! This module provides `DirectoryService` for gRPC service registration and discovery.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use anyhow::Result;
use async_trait::async_trait;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

use modkit::context::ModuleCtx;
use modkit::contracts::{GrpcServiceModule, RegisterGrpcServiceFn, SystemModule};
use modkit::directory::LocalDirectoryApi;
use modkit::runtime::ModuleManager;
use modkit::DirectoryApi;

use module_orchestrator_grpc::DIRECTORY_SERVICE_NAME;

mod server;

pub use module_orchestrator_contracts::{
    RegisterInstanceInfo, ServiceEndpoint, ServiceInstanceInfo,
};
pub use module_orchestrator_grpc::DirectoryGrpcClient;

/// Configuration for the module orchestrator
#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct ModuleOrchestratorConfig;

/// Module Orchestrator - system module for service discovery
///
/// This module:
/// - Provides `DirectoryApi` to the `ClientHub` for in-process modules
/// - Exposes `DirectoryService` gRPC service via `grpc_hub`
/// - Tracks module instances and provides service resolution
#[modkit::module(
    name = "module_orchestrator",
    capabilities = [grpc, system],
    client = module_orchestrator_contracts::DirectoryApi
)]
pub struct ModuleOrchestrator {
    config: RwLock<ModuleOrchestratorConfig>,
    directory_api: OnceLock<Arc<dyn DirectoryApi>>,
    module_manager: OnceLock<Arc<ModuleManager>>,
}

impl Default for ModuleOrchestrator {
    fn default() -> Self {
        Self {
            config: RwLock::new(ModuleOrchestratorConfig),
            directory_api: OnceLock::new(),
            module_manager: OnceLock::new(),
        }
    }
}

#[async_trait]
impl SystemModule for ModuleOrchestrator {
    fn pre_init(&self, sys: &modkit::runtime::SystemContext) -> anyhow::Result<()> {
        self.module_manager
            .set(Arc::clone(&sys.module_manager))
            .map_err(|_| anyhow::anyhow!("ModuleManager already set (pre_init called twice?)"))?;
        Ok(())
    }
}

#[async_trait]
impl modkit::Module for ModuleOrchestrator {
    async fn init(&self, ctx: &ModuleCtx) -> Result<()> {
        // Load configuration if present
        let cfg = ctx.config::<ModuleOrchestratorConfig>().unwrap_or_default();
        *self.config.write().await = cfg;

        // Use the injected ModuleManager to create the LocalDirectoryApi
        let manager =
            self.module_manager.get().cloned().ok_or_else(|| {
                anyhow::anyhow!("ModuleManager not wired into ModuleOrchestrator")
            })?;

        let api_impl: Arc<dyn DirectoryApi> = Arc::new(LocalDirectoryApi::new(manager));

        // Register in ClientHub directly
        ctx.client_hub()
            .register::<dyn DirectoryApi>(api_impl.clone());

        self.directory_api
            .set(api_impl)
            .map_err(|_| anyhow::anyhow!("DirectoryApi already set (init called twice?)"))?;

        tracing::info!("ModuleOrchestrator initialized");

        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_system_module(&self) -> Option<&dyn SystemModule> {
        Some(self)
    }
}

/// Export gRPC services to `grpc_hub`
#[async_trait]
impl GrpcServiceModule for ModuleOrchestrator {
    async fn get_grpc_services(&self, _ctx: &ModuleCtx) -> Result<Vec<RegisterGrpcServiceFn>> {
        let api = self
            .directory_api
            .get()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("DirectoryApi not initialized"))?;

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
