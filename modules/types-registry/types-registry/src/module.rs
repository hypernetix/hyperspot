//! Module declaration for the Types Registry module.

use std::sync::Arc;

use async_trait::async_trait;
use modkit::api::OpenApiRegistry;
use modkit::contracts::SystemModule;
use modkit::{Module, ModuleCtx, RestfulModule};
use tracing::{debug, info};
use types_registry_sdk::TypesRegistryApi;

use crate::config::TypesRegistryConfig;
use crate::domain::service::TypesRegistryService;
use crate::infra::InMemoryGtsRepository;
use crate::local_client::TypesRegistryLocalClient;

/// Types Registry module.
///
/// Provides GTS entity registration, storage, validation, and REST API endpoints.
///
/// ## Capabilities
///
/// - `system` — Core infrastructure module, initialized early in startup
/// - `rest` — Exposes REST API endpoints
#[modkit::module(
    name = "types_registry",
    capabilities = [system, rest]
)]
pub struct TypesRegistryModule {
    service: arc_swap::ArcSwapOption<TypesRegistryService>,
}

impl Default for TypesRegistryModule {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::from(None),
        }
    }
}

impl Clone for TypesRegistryModule {
    fn clone(&self) -> Self {
        Self {
            service: arc_swap::ArcSwapOption::new(self.service.load().as_ref().map(Clone::clone)),
        }
    }
}

#[async_trait]
impl Module for TypesRegistryModule {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing types_registry module");

        let cfg: TypesRegistryConfig = ctx.config()?;
        debug!(
            "Loaded types_registry config: entity_id_fields={:?}, schema_id_fields={:?}",
            cfg.entity_id_fields, cfg.schema_id_fields
        );

        let gts_config = cfg.to_gts_config();
        let repo = Arc::new(InMemoryGtsRepository::new(gts_config));
        let service = Arc::new(TypesRegistryService::new(repo));

        self.service.store(Some(service.clone()));

        let api: Arc<dyn TypesRegistryApi> = Arc::new(TypesRegistryLocalClient::new(service));
        ctx.client_hub().register::<dyn TypesRegistryApi>(api);

        info!("Types registry module initialized");
        Ok(())
    }
}

#[async_trait]
impl SystemModule for TypesRegistryModule {
    /// Post-init hook: switches the registry to ready mode.
    ///
    /// This runs AFTER `init()` has completed for ALL modules.
    /// At this point, all modules have had a chance to register their types,
    /// so we can safely validate and switch to ready mode.
    async fn post_init(&self, _sys: &modkit::runtime::SystemContext) -> anyhow::Result<()> {
        info!("types_registry post_init: switching to ready mode");

        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();

        service
            .switch_to_ready()
            .map_err(|e| anyhow::anyhow!("Failed to switch to ready mode: {e}"))?;

        info!("types_registry switched to ready mode successfully");
        Ok(())
    }
}

impl RestfulModule for TypesRegistryModule {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        info!("Registering types_registry REST routes");

        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();

        let router = crate::api::rest::routes::register_routes(router, openapi, service);

        info!("Types registry REST routes registered successfully");
        Ok(router)
    }
}
