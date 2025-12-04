#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

//! Directory Service - gRPC service for module instance discovery

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock as BlockingRwLock;
use std::sync::Arc;
use tokio::sync::RwLock;

use modkit::context::ModuleCtx;
use modkit::contracts::SystemModule;
use modkit::directory::LocalDirectoryApi;
use modkit::runtime::ModuleManager;
use modkit::DirectoryApi;

mod config;
mod server;

use config::DirectoryServiceConfig;
use server::make_directory_service;

/// Directory service module - exports a gRPC DirectoryService to the process grpc_hub
#[modkit::module(
    name = "directory_service",
    capabilities = [grpc, system],
    client = modkit::DirectoryApi
)]
pub struct DirectoryServiceModule {
    config: RwLock<DirectoryServiceConfig>,
    directory_api: RwLock<Option<Arc<dyn DirectoryApi>>>,
    module_manager: BlockingRwLock<Option<Arc<ModuleManager>>>,
}

impl Default for DirectoryServiceModule {
    fn default() -> Self {
        Self {
            config: RwLock::new(DirectoryServiceConfig::default()),
            directory_api: RwLock::new(None),
            module_manager: BlockingRwLock::new(None),
        }
    }
}

impl SystemModule for DirectoryServiceModule {
    fn wire_system(&self, sys: &modkit::runtime::SystemContext) {
        let mut guard = self.module_manager.write();
        *guard = Some(Arc::clone(&sys.module_manager));
    }
}

#[async_trait]
impl modkit::Module for DirectoryServiceModule {
    async fn init(&self, ctx: &ModuleCtx) -> Result<()> {
        let cfg = ctx.config::<DirectoryServiceConfig>()?;
        *self.config.write().await = cfg;

        // Use the injected ModuleManager instead of a global singleton
        let manager = self
            .module_manager
            .read()
            .as_ref()
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!("ModuleManager not wired into DirectoryServiceModule")
            })?;

        let api_impl: Arc<dyn DirectoryApi> = Arc::new(LocalDirectoryApi::new(manager));

        // Register in ClientHub using the generated helper function
        expose_directory_service_client(ctx, &api_impl)?;

        *self.directory_api.write().await = Some(api_impl);

        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_system_module(&self) -> Option<&dyn SystemModule> {
        Some(self)
    }
}

/// Export gRPC services to grpc_hub
#[async_trait]
impl modkit::contracts::GrpcServiceModule for DirectoryServiceModule {
    async fn get_grpc_services(
        &self,
        _ctx: &ModuleCtx,
    ) -> Result<Vec<modkit::contracts::RegisterGrpcServiceFn>> {
        let api = self
            .directory_api
            .read()
            .await
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("DirectoryApi not initialized"))?;

        // Build tonic service once and move it into the installer closure
        let svc = make_directory_service(api);
        let installer = modkit::contracts::RegisterGrpcServiceFn {
            service_name: server::SERVICE_NAME,
            register: Box::new(move |routes| {
                // The service implements Service<Request<Body>> + NamedService
                routes.add_service(svc.clone());
            }),
        };

        Ok(vec![installer])
    }
}
