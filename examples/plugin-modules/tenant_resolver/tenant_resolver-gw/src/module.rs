use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use modkit::api::OpenApiRegistry;
use modkit::context::ModuleCtx;
use modkit::contracts::RestfulModule;
use modkit::Module;
use modkit_security::SecurityContext;
use tenant_resolver_sdk::{TenantResolverClient, TenantResolverPluginSpecV1};
use tracing::info;
use types_registry_sdk::TypesRegistryApi;

use crate::config::TenantResolverConfig;
use crate::domain::service::Service;
use crate::local_client::TenantResolverGwClient;

/// Tenant Resolver Gateway module.
///
/// This module:
/// 1. Registers the **plugin schema** in types-registry (once, for all plugins)
/// 2. Discovers plugin instances via `types_registry`
/// 3. Routes requests to the selected plugin based on vendor configuration
///
/// **Plugin registration pattern:**
/// - Gateway registers the **schema** (GTS type definition)
/// - Plugins register their **instances** (specific plugin implementations)
///
/// Plugin discovery is **lazy**: it happens on the first API call,
/// after `types_registry` has switched to ready mode in `post_init`.
///
/// **Note on plugin dependencies:**
/// The gateway does NOT declare hard dependencies on specific plugins.
/// Plugins are discovered dynamically via `types_registry`. To include
/// plugins in a build, add them to `registered_modules.rs` in the server
/// binary or use Cargo feature flags.
#[modkit::module(
    name = "tenant_resolver_gateway",
    deps = ["types_registry"],
    capabilities = [rest]
)]
pub struct TenantResolverGateway {
    /// Service instance, initialized once during `init()`.
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
        let cfg: TenantResolverConfig = ctx.config()?;
        tracing::Span::current().record("vendor", cfg.vendor.as_str());
        info!(vendor = %cfg.vendor, "Initializing tenant_resolver_gateway");

        // === SCHEMA REGISTRATION ===
        // Gateway is responsible for registering the plugin SCHEMA in types-registry.
        // Plugins only register their INSTANCES.
        // Use GTS-provided method for proper $id and $ref handling.
        let registry = ctx.client_hub().get::<dyn TypesRegistryApi>()?;
        let schema_str = TenantResolverPluginSpecV1::gts_schema_with_refs_as_string();
        let schema_json: serde_json::Value = serde_json::from_str(&schema_str)?;

        let _ = registry
            .register(&SecurityContext::root(), vec![schema_json])
            .await?;
        info!(
            "Registered {} schema in types-registry",
            TenantResolverPluginSpecV1::gts_schema_id().clone()
        );

        let hub = ctx.client_hub();
        let svc = Arc::new(Service::new(hub, cfg.vendor));

        // Register gateway client into ClientHub for other modules
        let api: Arc<dyn TenantResolverClient> = Arc::new(TenantResolverGwClient::new(svc.clone()));
        ctx.client_hub().register::<dyn TenantResolverClient>(api);

        self.service
            .set(svc)
            .map_err(|_| anyhow::anyhow!("Service already initialized"))?;

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
            .get()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();

        Ok(crate::api::rest::routes::register_routes(
            router, openapi, svc,
        ))
    }
}
