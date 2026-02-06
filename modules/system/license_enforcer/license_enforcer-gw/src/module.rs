//! License enforcer gateway module.

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use license_enforcer_sdk::{
    LicenseCachePluginSpecV1, LicenseEnforcerGatewayClient, LicensePlatformPluginSpecV1,
};
use modkit::Module;
use modkit::context::ModuleCtx;
use tracing::info;
use types_registry_sdk::TypesRegistryClient;

use crate::config::LicenseEnforcerGatewayConfig;
use crate::domain::{LocalClient, Service};

/// License enforcer gateway module.
///
/// Discovers and routes to platform and cache plugins for license enforcement.
///
/// **Registration pattern:**
/// - Gateway registers plugin schemas (GTS type definitions) with types-registry
/// - Gateway registers gateway client in `ClientHub` (unscoped)
/// - Plugins register instances (metadata) with types-registry
/// - Plugins register scoped clients in `ClientHub`
#[modkit::module(
    name = "license_enforcer_gateway",
    deps = ["types_registry"]
)]
pub struct LicenseEnforcerGateway {
    service: OnceLock<Arc<Service>>,
}

impl Default for LicenseEnforcerGateway {
    fn default() -> Self {
        Self {
            service: OnceLock::new(),
        }
    }
}

#[async_trait]
impl Module for LicenseEnforcerGateway {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing license_enforcer_gateway");

        // Load configuration
        let cfg: LicenseEnforcerGatewayConfig = ctx.config()?;
        info!(vendor = %cfg.vendor, "Loaded gateway configuration");

        // Register plugin schemas with types-registry
        let registry = ctx.client_hub().get::<dyn TypesRegistryClient>()?;

        // Register platform plugin schema
        let platform_schema_id = LicensePlatformPluginSpecV1::gts_schema_id();
        info!(schema_id = %platform_schema_id, "Registering platform plugin schema");
        let platform_schema_str = LicensePlatformPluginSpecV1::gts_schema_with_refs_as_string();
        let platform_schema_json: serde_json::Value = serde_json::from_str(&platform_schema_str)?;
        let results = registry.register(vec![platform_schema_json]).await?;
        for result in results {
            if let types_registry_sdk::RegisterResult::Err { gts_id, error } = result {
                let schema_id_str = platform_schema_id.to_string();
                let gts_id_str = gts_id.as_deref().unwrap_or(&schema_id_str);
                return Err(anyhow::anyhow!(
                    "Failed to register platform plugin schema '{gts_id_str}': {error}"
                ));
            }
        }

        // Register cache plugin schema
        let cache_schema_id = LicenseCachePluginSpecV1::gts_schema_id();
        info!(schema_id = %cache_schema_id, "Registering cache plugin schema");
        let cache_schema_str = LicenseCachePluginSpecV1::gts_schema_with_refs_as_string();
        let cache_schema_json: serde_json::Value = serde_json::from_str(&cache_schema_str)?;
        let results = registry.register(vec![cache_schema_json]).await?;
        for result in results {
            if let types_registry_sdk::RegisterResult::Err { gts_id, error } = result {
                let schema_id_str = cache_schema_id.to_string();
                let gts_id_str = gts_id.as_deref().unwrap_or(&schema_id_str);
                return Err(anyhow::anyhow!(
                    "Failed to register cache plugin schema '{gts_id_str}': {error}"
                ));
            }
        }

        // Register feature GTS schemas (base schema + global feature instances)
        info!("Registering feature GTS schemas");
        let feature_schemas = license_enforcer_sdk::get_feature_gts_schemas()
            .map_err(|e| anyhow::anyhow!("Failed to get feature GTS schemas: {e}"))?;
        let results = registry.register(feature_schemas).await?;

        let mut success_count = 0;
        let mut failure_count = 0;

        for result in results {
            match result {
                types_registry_sdk::RegisterResult::Ok(entity) => {
                    tracing::trace!(gts_id = %entity.gts_id, "Registered feature GTS schema");
                    success_count += 1;
                }
                types_registry_sdk::RegisterResult::Err { gts_id, error } => {
                    tracing::warn!(
                        gts_id = ?gts_id,
                        error = %error,
                        "Failed to register feature GTS schema"
                    );
                    failure_count += 1;
                }
            }
        }

        info!(
            success = success_count,
            failures = failure_count,
            "Feature GTS schema registration completed"
        );

        // Fail initialization if any feature schemas failed to register
        if failure_count > 0 {
            return Err(anyhow::anyhow!(
                "Failed to register {failure_count} feature GTS schema(s). See warnings above for details."
            ));
        }

        // Create service
        let service = Arc::new(Service::new(ctx.client_hub(), cfg.vendor.clone()));
        self.service
            .set(service.clone())
            .map_err(|_| anyhow::anyhow!("Service already initialized"))?;

        // Create and register gateway client
        let local_client = Arc::new(LocalClient::new(service));
        let gateway_client: Arc<dyn LicenseEnforcerGatewayClient> = local_client;
        ctx.client_hub()
            .register::<dyn LicenseEnforcerGatewayClient>(gateway_client);

        info!("License enforcer gateway initialized");
        Ok(())
    }
}
