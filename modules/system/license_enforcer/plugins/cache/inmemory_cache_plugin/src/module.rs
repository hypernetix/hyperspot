//! In-memory cache plugin module.

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use license_enforcer_sdk::{CachePluginClient, LicenseCachePluginSpecV1};
use modkit::Module;
use modkit::client_hub::ClientScope;
use modkit::context::ModuleCtx;
use modkit::gts::BaseModkitPluginV1;
use tracing::info;
use types_registry_sdk::TypesRegistryClient;

use crate::config::InMemoryCachePluginConfig;
use crate::domain::{Client, Service};

/// In-memory cache plugin module.
///
/// Provides TTL-based in-memory caching for tenant-scoped enabled global feature sets.
#[modkit::module(
    name = "inmemory_cache_plugin",
    deps = ["types_registry", "license_enforcer_gateway"]
)]
pub struct InMemoryCachePlugin {
    service: OnceLock<Arc<Service>>,
}

impl Default for InMemoryCachePlugin {
    fn default() -> Self {
        Self {
            service: OnceLock::new(),
        }
    }
}

#[async_trait]
impl Module for InMemoryCachePlugin {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing inmemory_cache_plugin");

        // Load configuration
        let cfg: InMemoryCachePluginConfig = ctx.config()?;
        info!(
            vendor = %cfg.vendor,
            priority = cfg.priority,
            ttl_secs = cfg.ttl.as_secs(),
            max_entries = cfg.max_entries,
            "Loaded plugin configuration"
        );

        // Generate plugin instance ID
        let instance_id = LicenseCachePluginSpecV1::gts_make_instance_id(
            "hyperspot.builtin.inmemory_cache.plugin.v1",
        );

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

        // Create service with configured TTL and max_entries
        let service = Arc::new(Service::new(cfg.ttl, cfg.max_entries));
        self.service
            .set(service.clone())
            .map_err(|_| anyhow::anyhow!("Service already initialized"))?;

        // Register scoped client in ClientHub
        let client = Arc::new(Client::new(service));
        let api: Arc<dyn CachePluginClient> = client;
        ctx.client_hub()
            .register_scoped::<dyn CachePluginClient>(ClientScope::gts_id(&instance_id), api);

        info!(instance_id = %instance_id, "In-memory cache plugin initialized");
        Ok(())
    }
}
