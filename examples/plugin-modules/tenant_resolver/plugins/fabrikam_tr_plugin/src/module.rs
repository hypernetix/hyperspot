//! Fabrikam tenant resolver plugin module.

use std::sync::Arc;

use async_trait::async_trait;
use modkit::client_hub::ClientScope;
use modkit::context::ModuleCtx;
use modkit::Module;
use modkit_security::SecurityCtx;
use tenant_resolver_sdk::{ThrPluginApi, ThrPluginSpec};
use tracing::info;
use types_registry_sdk::TypesRegistryApi;

use crate::config::FabrikamPluginConfig;
use crate::domain::Service;

/// Fabrikam tenant resolver plugin module.
#[modkit::module(
    name = "fabrikam_tr_plugin",
    deps = ["types_registry"],
)]
pub struct FabrikamTrPlugin {
    service: arc_swap::ArcSwapOption<Service>,
}

impl Default for FabrikamTrPlugin {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::empty(),
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
        let instance_id =
            ThrPluginSpec::make_gts_instance_id("fabrikam.plugins._.thr_plugin.v1").to_string();

        // Register plugin schema and instance in types-registry
        let registry = ctx.client_hub().get::<dyn TypesRegistryApi>()?;

        // First, register the schema (idempotent — OK if already registered by another plugin)
        // Generate JSON Schema from Rust type using schemars, then add GTS $id
        let mut schema = schemars::schema_for!(ThrPluginSpec);
        schema.schema.extensions.insert(
            "$id".to_owned(),
            serde_json::json!(ThrPluginSpec::GTS_SCHEMA_ID),
        );
        let schema_json = serde_json::to_value(&schema)?;
        let _ = registry
            .register(&SecurityCtx::root_ctx(), vec![schema_json])
            .await?;

        // Then, register the instance
        let instance = ThrPluginSpec {
            id: instance_id.clone(),
            vendor: cfg.vendor.clone(),
            priority: cfg.priority,
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
        self.service.store(Some(domain_service.clone()));

        // Register scoped client in ClientHub
        let api: Arc<dyn ThrPluginApi> = domain_service;
        ctx.client_hub()
            .register_scoped::<dyn ThrPluginApi>(ClientScope::gts_id(&instance_id), api);

        info!(
            instance_id = %instance_id,
            vendor = %cfg.vendor,
            tenant_count = cfg.tenants.len(),
            "Fabrikam plugin initialized with tenant tree"
        );
        Ok(())
    }
}
