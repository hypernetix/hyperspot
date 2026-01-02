//! Fabrikam tenant resolver plugin module.

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use modkit::client_hub::ClientScope;
use modkit::context::ModuleCtx;
use modkit::gts::BaseModkitPluginV1;
use modkit::Module;
use modkit_security::SecurityCtx;
use tenant_resolver_sdk::{TenantResolverPluginClient, TenantResolverPluginSpecV1};
use tracing::info;
use types_registry_sdk::TypesRegistryApi;

use crate::config::FabrikamPluginConfig;
use crate::domain::Service;

/// Fabrikam tenant resolver plugin module.
///
/// **Plugin registration pattern:**
/// - The gateway module registers the plugin **schema** (GTS type definition)
/// - This plugin registers its **instance** (specific implementation metadata)
/// - This plugin registers its **scoped client** (implementation in `ClientHub`)
#[modkit::module(
    name = "fabrikam_tr_plugin",
    deps = ["types_registry"],
)]
pub struct FabrikamTrPlugin {
    /// Service instance, initialized once during `init()`.
    service: OnceLock<Arc<Service>>,
}

impl Default for FabrikamTrPlugin {
    fn default() -> Self {
        Self {
            service: OnceLock::new(),
        }
    }
}

#[async_trait]
impl Module for FabrikamTrPlugin {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing fabrikam_tr_plugin");

        // Load configuration
        let cfg: FabrikamPluginConfig = ctx.config()?;

        // Generate plugin instance ID
        let instance_id = TenantResolverPluginSpecV1::gts_make_instance_id(
            "fabrikam.plugins.tenant_resolver.plugin.v1",
        );

        // === INSTANCE REGISTRATION ===
        // Register the plugin INSTANCE in types-registry.
        // Note: The plugin SCHEMA is registered by the gateway module.
        let registry = ctx.client_hub().get::<dyn TypesRegistryApi>()?;
        let vendor_clone = cfg.vendor.clone();
        let instance = BaseModkitPluginV1::<TenantResolverPluginSpecV1> {
            id: instance_id.clone(),
            vendor: cfg.vendor,
            priority: cfg.priority,
            properties: TenantResolverPluginSpecV1,
        };
        let instance_json = serde_json::to_value(&instance)?;

        let _ = registry
            .register(&SecurityCtx::root_ctx(), vec![instance_json])
            .await?;

        // Create service with tenant tree from config
        let domain_service = Arc::new(
            Service::new(&cfg.tenants)
                .map_err(|e| anyhow::anyhow!("invalid Fabrikam tenant tree configuration: {e}"))?,
        );
        self.service
            .set(domain_service.clone())
            .map_err(|_| anyhow::anyhow!("Service already initialized"))?;

        // Register scoped client in ClientHub
        let api: Arc<dyn TenantResolverPluginClient> = domain_service;
        ctx.client_hub()
            .register_scoped::<dyn TenantResolverPluginClient>(
                ClientScope::gts_id(&instance_id),
                api,
            );

        info!(
            instance_id = %instance_id,
            vendor = %vendor_clone,
            tenant_count = cfg.tenants.len(),
            "Fabrikam plugin initialized with tenant tree"
        );
        Ok(())
    }
}
