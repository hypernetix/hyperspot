//! Core types registration module implementation.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use async_trait::async_trait;
use modkit::contracts::SystemCapability;
use modkit::gts::get_core_gts_schemas;
use modkit::{Module, ModuleCtx};
use tracing::{debug, info};
use types_registry_sdk::TypesRegistryClient;
use types_sdk::TypesClient;

use crate::domain::TypesLocalClient;

/// Core types registration module.
///
/// This system module is responsible for registering core GTS types that are used
/// throughout the framework (e.g., `BaseModkitPluginV1` for plugin systems).
///
/// ## Initialization Order
///
/// This module must initialize after `types_registry` but before any modules that
/// use plugin systems or other core GTS types.
///
/// Dependency chain: `types_registry` → `types` → plugin modules
///
/// ## Core Types Registered
///
/// - `BaseModkitPluginV1` - Base schema for all plugin instances
/// - Any future core framework types
#[modkit::module(
    name = "types",
    deps = ["types_registry"],
    capabilities = [system]
)]
pub struct Types {
    ready: Arc<AtomicBool>,
}

impl Default for Types {
    fn default() -> Self {
        Self {
            ready: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Clone for Types {
    fn clone(&self) -> Self {
        Self {
            ready: Arc::clone(&self.ready),
        }
    }
}

#[async_trait]
impl Module for Types {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing types module");

        // Get the types registry client
        let registry = ctx.client_hub().get::<dyn TypesRegistryClient>()?;

        // Register core GTS types that other modules depend on
        // This must happen before any module registers derived schemas/instances
        debug!("Registering core GTS schemas");
        let core_schemas = get_core_gts_schemas()?;

        registry
            .register(core_schemas)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to register core GTS schemas: {e}"))?;

        info!("Core GTS schemas registered successfully");

        // Create and register the local client
        let client = TypesLocalClient::new(Arc::clone(&self.ready));
        client.set_ready();

        let api: Arc<dyn TypesClient> = Arc::new(client);
        ctx.client_hub().register::<dyn TypesClient>(api);

        info!("Types module initialized");

        Ok(())
    }
}

#[async_trait]
impl SystemCapability for Types {}
