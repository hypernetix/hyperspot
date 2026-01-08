//! `ModKit` Framework module implementation.

use async_trait::async_trait;
use modkit::contracts::SystemModule;
use modkit::gts::get_core_gts_schemas;
use modkit::{Module, ModuleCtx};
use modkit_security::SecurityCtx;
use tracing::{debug, info};
use types_registry_sdk::TypesRegistryApi;

/// `ModKit` Framework module.
///
/// This system module is responsible for registering core GTS types that are used
/// throughout the `ModKit` framework (e.g., `BaseModkitPluginV1` for plugin systems).
///
/// ## Initialization Order
///
/// This module must initialize after `types_registry` but before any modules that
/// use plugin systems or other core `ModKit` GTS types.
///
/// Dependency chain: `types_registry` → `modkit_framework` → plugin modules
///
/// ## Core Types Registered
///
/// - `BaseModkitPluginV1` - Base schema for all plugin instances
/// - Any future core `ModKit` framework types
#[modkit::module(
    name = "modkit_framework",
    deps = ["types_registry"],
    capabilities = [system]
)]
pub struct ModKitFramework;

impl Default for ModKitFramework {
    fn default() -> Self {
        Self
    }
}

impl Clone for ModKitFramework {
    fn clone(&self) -> Self {
        Self
    }
}

#[async_trait]
impl Module for ModKitFramework {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing modkit_framework module");

        // Get the types registry client
        let registry = ctx.client_hub().get::<dyn TypesRegistryApi>()?;

        // Register core GTS types that other modules depend on
        // This must happen before any module registers derived schemas/instances
        debug!("Registering core ModKit GTS schemas");
        let core_schemas = get_core_gts_schemas()?;

        registry
            .register(&SecurityCtx::root_ctx(), core_schemas)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to register core GTS schemas: {e}"))?;

        info!("Core ModKit GTS schemas registered successfully");
        info!("ModKit framework module initialized");

        Ok(())
    }
}

#[async_trait]
impl SystemModule for ModKitFramework {}
