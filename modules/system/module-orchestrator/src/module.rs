//! Module definition for `ModuleOrchestrator`

use anyhow::Result;
use async_trait::async_trait;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

use modkit::DirectoryClient;
use modkit::context::ModuleCtx;
use modkit::contracts::{GrpcServiceCapability, RegisterGrpcServiceFn, SystemCapability};
use modkit::directory::LocalDirectoryClient;
use modkit::runtime::ModuleManager;

use cf_system_sdks::directory::DIRECTORY_SERVICE_NAME;

use crate::server;

/// Configuration for the module orchestrator
#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct ModuleOrchestratorConfig;

/// Module Orchestrator - system module for service discovery
///
/// This module:
/// - Provides `DirectoryClient` to the `ClientHub` for in-process modules
/// - Exposes `DirectoryService` gRPC service via `grpc-hub`
/// - Tracks module instances and provides service resolution
#[modkit::module(
    name = "module-orchestrator",
    capabilities = [grpc, system],
    client = cf_system_sdks::directory::DirectoryClient
)]
pub struct ModuleOrchestrator {
    config: RwLock<ModuleOrchestratorConfig>,
    directory_api: OnceLock<Arc<dyn DirectoryClient>>,
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
impl SystemCapability for ModuleOrchestrator {
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

        // Use the injected ModuleManager to create the DirectoryClient
        let manager =
            self.module_manager.get().cloned().ok_or_else(|| {
                anyhow::anyhow!("ModuleManager not wired into ModuleOrchestrator")
            })?;

        let api_impl: Arc<dyn DirectoryClient> = Arc::new(LocalDirectoryClient::new(manager));

        // Register in ClientHub directly
        ctx.client_hub()
            .register::<dyn DirectoryClient>(api_impl.clone());

        self.directory_api
            .set(api_impl)
            .map_err(|_| anyhow::anyhow!("DirectoryClient already set (init called twice?)"))?;

        tracing::info!("ModuleOrchestrator initialized");

        Ok(())
    }
}

/// Export gRPC services to `grpc-hub`
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
