//! Static `AuthN` resolver plugin module.

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use authn_resolver_sdk::{AuthNResolverPluginClient, AuthNResolverPluginSpecV1};
use modkit::Module;
use modkit::client_hub::ClientScope;
use modkit::context::ModuleCtx;
use modkit::gts::BaseModkitPluginV1;
use tracing::info;
use types_registry_sdk::TypesRegistryClient;

use crate::config::StaticAuthNPluginConfig;
use crate::domain::Service;

/// Static `AuthN` resolver plugin module.
///
/// Provides token-to-identity mapping from configuration.
///
/// **Plugin registration pattern:**
/// - Gateway registers the plugin schema (GTS type definition)
/// - This plugin registers its instance (implementation metadata)
/// - This plugin registers its scoped client (implementation in `ClientHub`)
#[modkit::module(
    name = "static-authn-plugin",
    deps = ["types-registry"]
)]
pub struct StaticAuthNPlugin {
    service: OnceLock<Arc<Service>>,
}

impl Default for StaticAuthNPlugin {
    fn default() -> Self {
        Self {
            service: OnceLock::new(),
        }
    }
}

#[async_trait]
impl Module for StaticAuthNPlugin {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing static_authn_plugin");

        // Load configuration
        let cfg: StaticAuthNPluginConfig = ctx.config()?;
        info!(
            vendor = %cfg.vendor,
            priority = cfg.priority,
            mode = ?cfg.mode,
            token_count = cfg.tokens.len(),
            "Loaded plugin configuration"
        );

        // Generate plugin instance ID
        let instance_id = AuthNResolverPluginSpecV1::gts_make_instance_id(
            "hyperspot.builtin.static_authn_resolver.plugin.v1",
        );

        // Register plugin instance in types-registry
        let registry = ctx.client_hub().get::<dyn TypesRegistryClient>()?;
        let instance = BaseModkitPluginV1::<AuthNResolverPluginSpecV1> {
            id: instance_id.clone(),
            vendor: cfg.vendor.clone(),
            priority: cfg.priority,
            properties: AuthNResolverPluginSpecV1,
        };
        let instance_json = serde_json::to_value(&instance)?;

        let _ = registry.register(vec![instance_json]).await?;

        // Create service from config
        let service = Arc::new(Service::from_config(&cfg));
        self.service
            .set(service.clone())
            .map_err(|_| anyhow::anyhow!("Service already initialized"))?;

        // Register scoped client in ClientHub
        let api: Arc<dyn AuthNResolverPluginClient> = service;
        ctx.client_hub()
            .register_scoped::<dyn AuthNResolverPluginClient>(
                ClientScope::gts_id(&instance_id),
                api,
            );

        info!(instance_id = %instance_id, "Static authn plugin initialized");
        Ok(())
    }
}
