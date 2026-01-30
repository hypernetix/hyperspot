//! No-cache plugin module.

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use license_enforcer_sdk::{CachePluginClient, LicenseCachePluginSpecV1};
use modkit::Module;
use modkit::client_hub::ClientScope;
use modkit::context::ModuleCtx;
use modkit::gts::BaseModkitPluginV1;
use tracing::info;
use types_registry_sdk::TypesRegistryClient;

use crate::config::NoCachePluginConfig;
use crate::domain::{Client, Service};

/// No-cache plugin module.
///
/// Provides no-op caching (always cache miss).
///
/// **Plugin registration pattern:**
/// - Gateway registers the plugin schema (GTS type definition)
/// - This plugin registers its instance (implementation metadata)
/// - This plugin registers its scoped client (implementation in `ClientHub`)
#[modkit::module(
    name = "nocache_plugin",
    deps = ["types_registry", "license_enforcer_gateway"]
)]
pub struct NoCachePlugin {
    service: OnceLock<Arc<Service>>,
}

impl Default for NoCachePlugin {
    fn default() -> Self {
        Self {
            service: OnceLock::new(),
        }
    }
}

#[async_trait]
impl Module for NoCachePlugin {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing nocache_plugin");

        // Load configuration
        let cfg: NoCachePluginConfig = ctx.config()?;
        info!(
            vendor = %cfg.vendor,
            priority = cfg.priority,
            "Loaded plugin configuration"
        );

        // Generate plugin instance ID
        let instance_id =
            LicenseCachePluginSpecV1::gts_make_instance_id("hyperspot.builtin.nocache.plugin.v1");

        // Register plugin instance in types-registry
        let registry = ctx.client_hub().get::<dyn TypesRegistryClient>()?;
        let instance = BaseModkitPluginV1::<LicenseCachePluginSpecV1> {
            id: instance_id.clone(),
            vendor: cfg.vendor.clone(),
            priority: cfg.priority,
            properties: LicenseCachePluginSpecV1,
        };
        let instance_json = serde_json::to_value(&instance)?;

        let _ = registry.register(vec![instance_json]).await?;

        // Create service
        let service = Arc::new(Service::new());
        self.service
            .set(service.clone())
            .map_err(|_| anyhow::anyhow!("Service already initialized"))?;

        // Register scoped client in ClientHub
        let client = Arc::new(Client::new(service));
        let api: Arc<dyn CachePluginClient> = client;
        ctx.client_hub()
            .register_scoped::<dyn CachePluginClient>(ClientScope::gts_id(&instance_id), api);

        info!(instance_id = %instance_id, "No-cache plugin initialized");
        Ok(())
    }
}
