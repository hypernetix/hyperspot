use std::sync::Arc;

use async_trait::async_trait;
use modkit::api::OpenApiRegistry;
use modkit::context::ModuleCtx;
use modkit::contracts::RestfulModule;
use modkit::Module;
use tenant_resolver_sdk::TenantResolverClient;
use tracing::info;

use crate::config::TenantResolverConfig;
use crate::domain::service::Service;
use crate::local_client::TenantResolverGwClient;

/// Tenant Resolver Gateway module.
///
/// This module discovers plugin instances via `types_registry` and routes
/// requests to the selected plugin based on vendor configuration.
///
/// Plugin discovery is **lazy**: it happens on the first API call,
/// after `types_registry` has switched to ready mode in `post_init`.
#[modkit::module(
    name = "tenant_resolver_gateway",
    deps = ["types_registry", "contoso_tr_plugin", "fabrikam_tr_plugin"],
    capabilities = [rest]
)]
pub struct TenantResolverGateway {
    service: arc_swap::ArcSwapOption<Service>,
}

impl Default for TenantResolverGateway {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::empty(),
        }
    }
}

#[async_trait]
impl Module for TenantResolverGateway {
    #[tracing::instrument(skip_all, fields(vendor))]
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        let cfg: TenantResolverConfig = ctx.config()?;
        tracing::Span::current().record("vendor", cfg.vendor.as_str());
        info!(vendor = %cfg.vendor, "Initializing tenant_resolver_gateway (lazy plugin discovery)");

        let hub = ctx.client_hub();
        let svc = Arc::new(Service::new(hub, cfg.vendor));

        // Register gateway client into ClientHub for other modules
        let api: Arc<dyn TenantResolverClient> = Arc::new(TenantResolverGwClient::new(svc.clone()));
        ctx.client_hub().register::<dyn TenantResolverClient>(api);

        self.service.store(Some(svc));

        Ok(())
    }
}

impl RestfulModule for TenantResolverGateway {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        let svc = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();

        Ok(crate::api::rest::routes::register_routes(
            router, openapi, svc,
        ))
    }
}
