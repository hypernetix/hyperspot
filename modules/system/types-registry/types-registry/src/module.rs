//! Module declaration for the Types Registry module.

use std::sync::Arc;

use async_trait::async_trait;
use modkit::api::OpenApiRegistry;
use modkit::contracts::SystemCapability;
use modkit::{Module, ModuleCtx, RestApiCapability};
use tracing::{debug, info};
use types_registry_sdk::TypesRegistryClient;

use crate::config::TypesRegistryConfig;
use crate::domain::local_client::TypesRegistryLocalClient;
use crate::domain::service::TypesRegistryService;
use crate::infra::InMemoryGtsRepository;

/// Types Registry module.
///
/// Provides GTS entity registration, storage, validation, and REST API endpoints.
///
/// ## Capabilities
///
/// - `system` — Core infrastructure module, initialized early in startup
/// - `rest` — Exposes REST API endpoints
///
/// ## Note
///
/// Core GTS types (like `BaseModkitPluginV1`) are now registered by the
/// `types` module (`modules/system/types`), not here. This maintains proper
/// separation of concerns and avoids circular dependencies.
#[modkit::module(
    name = "types-registry",
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
        let service = Arc::new(TypesRegistryService::new(repo, cfg));

        self.service.store(Some(service.clone()));

        let api: Arc<dyn TypesRegistryClient> = Arc::new(TypesRegistryLocalClient::new(service));

        ctx.client_hub().register::<dyn TypesRegistryClient>(api);

        info!("Types registry module initialized");
        Ok(())
    }
}

#[async_trait]
impl SystemCapability for TypesRegistryModule {
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

        service.switch_to_ready().map_err(|e| {
            if let Some(errors) = e.validation_errors() {
                for err in errors {
                    // Try to get the entity content for debugging
                    let entity_content = match service.get(&err.gts_id) {
                        Ok(entity) => serde_json::to_string_pretty(&entity.content)
                            .unwrap_or_else(|_| "Failed to serialize".to_owned()),
                        _ => "Entity not found or failed to retrieve".to_owned(),
                    };

                    tracing::error!(
                        gts_id = %err.gts_id,
                        message = %err.message,
                        entity_content = %entity_content,
                        "GTS validation error"
                    );
                }
            }
            anyhow::anyhow!("Failed to switch to ready mode: {e}")
        })?;

        info!("types_registry switched to ready mode successfully");
        Ok(())
    }
}

impl RestApiCapability for TypesRegistryModule {
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
