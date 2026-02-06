//! Single-tenant resolver plugin module.

use std::sync::Arc;

use async_trait::async_trait;
use modkit::Module;
use modkit::client_hub::ClientScope;
use modkit::context::ModuleCtx;
use modkit::gts::BaseModkitPluginV1;
use tenant_resolver_sdk::{TenantResolverPluginClient, TenantResolverPluginSpecV1};
use tracing::info;
use types_registry_sdk::TypesRegistryClient;

use crate::domain::Service;

/// Hardcoded vendor name for GTS instance registration.
const VENDOR: &str = "hyperspot";

/// Hardcoded priority (higher value = lower priority).
/// Set to 1000 so `static_tr_plugin` (priority 100) wins when both are enabled.
const PRIORITY: i16 = 1000;

/// Single-tenant resolver plugin module.
///
/// Zero-configuration plugin for single-tenant deployments.
/// Returns the tenant from security context as the only accessible tenant.
#[modkit::module(
    name = "single-tenant-tr-plugin",
    deps = ["types-registry"]
)]
pub struct SingleTenantTrPlugin;

impl Default for SingleTenantTrPlugin {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Module for SingleTenantTrPlugin {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing single_tenant_tr_plugin");

        // Generate plugin instance ID
        let instance_id = TenantResolverPluginSpecV1::gts_make_instance_id(
            "hyperspot.builtin.single_tenant_resolver.plugin.v1",
        );

        // Register plugin instance in types-registry
        let registry = ctx.client_hub().get::<dyn TypesRegistryClient>()?;
        let instance = BaseModkitPluginV1::<TenantResolverPluginSpecV1> {
            id: instance_id.clone(),
            vendor: VENDOR.to_owned(),
            priority: PRIORITY,
            properties: TenantResolverPluginSpecV1,
        };
        let instance_json = serde_json::to_value(&instance)?;

        let _ = registry.register(vec![instance_json]).await?;

        // Create service and register scoped client in ClientHub
        let service = Arc::new(Service);
        let api: Arc<dyn TenantResolverPluginClient> = service;
        ctx.client_hub()
            .register_scoped::<dyn TenantResolverPluginClient>(
                ClientScope::gts_id(&instance_id),
                api,
            );

        info!(
            instance_id = %instance_id,
            vendor = VENDOR,
            priority = PRIORITY,
            "Single-tenant plugin initialized"
        );
        Ok(())
    }
}
