//! Tenant resolver module.

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use modkit::Module;
use modkit::context::ModuleCtx;
use tenant_resolver_sdk::{TenantResolverClient, TenantResolverPluginSpecV1};
use tracing::info;
use types_registry_sdk::TypesRegistryClient;

use crate::config::TenantResolverConfig;
use crate::domain::{Service, TenantResolverLocalClient};

/// Tenant Resolver module.
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
pub(crate) struct TenantResolver {
    service: OnceLock<Arc<Service>>,
}

impl Default for TenantResolver {
    fn default() -> Self {
        Self {
            service: OnceLock::new(),
        }
    }
}

#[async_trait]
impl Module for TenantResolver {
    #[tracing::instrument(skip_all, fields(vendor))]
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        let cfg: TenantResolverConfig = ctx.config()?;
        tracing::Span::current().record("vendor", cfg.vendor.as_str());
        info!(vendor = %cfg.vendor, "Initializing tenant_resolver module");

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

        // Register local client in ClientHub
        let api: Arc<dyn TenantResolverClient> =
            Arc::new(TenantResolverLocalClient::new(svc.clone()));
        ctx.client_hub().register::<dyn TenantResolverClient>(api);

        self.service
            .set(svc)
            .map_err(|_| anyhow::anyhow!("Service already initialized"))?;

        Ok(())
    }
}
