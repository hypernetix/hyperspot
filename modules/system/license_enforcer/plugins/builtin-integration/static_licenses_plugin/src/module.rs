//! Static licenses plugin module.

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use license_enforcer_sdk::models::parse_license_feature_id;
use license_enforcer_sdk::{LicensePlatformPluginSpecV1, PlatformPluginClient};
use modkit::Module;
use modkit::client_hub::ClientScope;
use modkit::context::ModuleCtx;
use modkit::gts::BaseModkitPluginV1;
use tracing::info;
use types_registry_sdk::TypesRegistryClient;

use crate::config::StaticLicensesPluginConfig;
use crate::domain::{Client, Service};

/// Static licenses plugin module.
///
/// Provides static license data from configuration.
///
/// **Plugin registration pattern:**
/// - Gateway registers the plugin schema (GTS type definition)
/// - This plugin registers its instance (implementation metadata)
/// - This plugin registers its scoped client (implementation in `ClientHub`)
#[modkit::module(
    name = "static_licenses_plugin",
    deps = ["types_registry", "license_enforcer_gateway"]
)]
pub struct StaticLicensesPlugin {
    service: OnceLock<Arc<Service>>,
}

impl Default for StaticLicensesPlugin {
    fn default() -> Self {
        Self {
            service: OnceLock::new(),
        }
    }
}

#[async_trait]
impl Module for StaticLicensesPlugin {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing static_licenses_plugin");

        // Load configuration
        let cfg: StaticLicensesPluginConfig = ctx.config()?;
        info!(
            vendor = %cfg.vendor,
            priority = cfg.priority,
            features_count = cfg.static_licenses_features.len(),
            "Loaded plugin configuration"
        );

        // Validate and convert configured features from strings to GtsInstanceId
        // Uses SDK's parse_license_feature_id for proper validation
        let mut configured_features = Vec::new();
        for feature_id in &cfg.static_licenses_features {
            let parsed = parse_license_feature_id(feature_id).map_err(|e| {
                anyhow::anyhow!("Invalid static_licenses_features: '{feature_id}' - {e}")
            })?;
            configured_features.push(parsed.to_gts());
        }

        // Generate plugin instance ID
        let instance_id = LicensePlatformPluginSpecV1::gts_make_instance_id(
            "hyperspot.builtin.static_licenses.plugin.v1",
        );

        // Register plugin instance in types-registry
        let registry = ctx.client_hub().get::<dyn TypesRegistryClient>()?;
        let instance = BaseModkitPluginV1::<LicensePlatformPluginSpecV1> {
            id: instance_id.clone(),
            vendor: cfg.vendor.clone(),
            priority: cfg.priority,
            properties: LicensePlatformPluginSpecV1,
        };
        let instance_json = serde_json::to_value(&instance)?;

        // Register plugin instance and check for per-entity failures
        let results = registry.register(vec![instance_json]).await?;
        for result in results {
            if let types_registry_sdk::RegisterResult::Err { gts_id, error } = result {
                let instance_id_str = instance_id.to_string();
                let gts_id_str = gts_id.as_deref().unwrap_or(&instance_id_str);
                return Err(anyhow::anyhow!(
                    "Failed to register plugin instance '{gts_id_str}': {error}"
                ));
            }
        }

        // Create service with configured features
        let service = Arc::new(Service::new(configured_features));
        self.service
            .set(service.clone())
            .map_err(|_| anyhow::anyhow!("Service already initialized"))?;

        // Register scoped client in ClientHub
        let client = Arc::new(Client::new(service));
        let api: Arc<dyn PlatformPluginClient> = client;
        ctx.client_hub()
            .register_scoped::<dyn PlatformPluginClient>(ClientScope::gts_id(&instance_id), api);

        info!(instance_id = %instance_id, "Static licenses plugin initialized");
        Ok(())
    }
}
