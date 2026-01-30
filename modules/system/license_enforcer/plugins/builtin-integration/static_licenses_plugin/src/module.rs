//! Static licenses plugin module.

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use gts::GtsID;
use license_enforcer_sdk::{LicenseFeatureID, LicensePlatformPluginSpecV1, PlatformPluginClient};
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

        // Validate and convert configured features from strings to LicenseFeatureID
        // Use gts crate for proper GTS ID validation (structure only, no registry validation)
        for feature_id in &cfg.static_licenses_features {
            if !GtsID::is_valid(feature_id) {
                anyhow::bail!(
                    "Invalid static_licenses_features: '{feature_id}' is not a valid GTS ID"
                );
            }
        }

        let configured_features: Vec<LicenseFeatureID> = cfg
            .static_licenses_features
            .iter()
            .map(|s| LicenseFeatureID::from(s.as_str()))
            .collect();

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

        let _ = registry.register(vec![instance_json]).await?;

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
