//! Tenant resolver gateway module.

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use modkit::Module;
use modkit::context::ModuleCtx;
use tenant_resolver_sdk::{TenantResolverGatewayClient, TenantResolverPluginSpecV1};
use tracing::info;
use types_registry_sdk::TypesRegistryClient;

use crate::config::TenantResolverGwConfig;
use crate::domain::{Service, TenantResolverGwLocalClient};

/// Tenant Resolver Gateway module.
///
/// This module:
/// 1. Registers the plugin schema in types-registry
/// 2. Discovers plugin instances via types-registry
/// 3. Routes requests to the selected plugin based on vendor configuration
///
/// Plugin discovery is lazy: happens on first API call after types-registry
/// is ready.
#[modkit::module(
    name = "tenant-resolver",
    deps = ["types-registry"],
    capabilities = []
)]
pub(crate) struct TenantResolverGateway {
    service: OnceLock<Arc<Service>>,
}

impl Default for TenantResolverGateway {
    fn default() -> Self {
        Self {
            service: OnceLock::new(),
        }
    }
}

#[async_trait]
impl Module for TenantResolverGateway {
    #[tracing::instrument(skip_all, fields(vendor))]
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        let cfg: TenantResolverGwConfig = ctx.config()?;
        tracing::Span::current().record("vendor", cfg.vendor.as_str());
        info!(vendor = %cfg.vendor, "Initializing tenant_resolver gateway");

        // Register plugin schema in types-registry
        let registry = ctx.client_hub().get::<dyn TypesRegistryClient>()?;
        let schema_str = TenantResolverPluginSpecV1::gts_schema_with_refs_as_string();
        let schema_json: serde_json::Value = serde_json::from_str(&schema_str)?;
        let _ = registry.register(vec![schema_json]).await?;
        info!(
            schema_id = %TenantResolverPluginSpecV1::gts_schema_id(),
            "Registered plugin schema in types-registry"
        );

        // Create service
        let hub = ctx.client_hub();
        let svc = Arc::new(Service::new(hub, cfg.vendor));

        // Register gateway client in ClientHub
        let api: Arc<dyn TenantResolverGatewayClient> =
            Arc::new(TenantResolverGwLocalClient::new(svc.clone()));
        ctx.client_hub()
            .register::<dyn TenantResolverGatewayClient>(api);

        self.service
            .set(svc)
            .map_err(|_| anyhow::anyhow!("Service already initialized"))?;

        Ok(())
    }
}
