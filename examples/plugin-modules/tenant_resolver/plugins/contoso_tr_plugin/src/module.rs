//! Contoso tenant resolver plugin module.

use std::sync::Arc;

use async_trait::async_trait;
use modkit::client_hub::ClientScope;
use modkit::context::ModuleCtx;
use modkit::Module;
use modkit_security::SecurityCtx;
use tenant_resolver_sdk::{ThrPluginApi, ThrPluginSpec};
use tracing::info;
use types_registry_sdk::TypesRegistryApi;

use crate::config::ContosoPluginConfig;
use crate::domain::Service;

/// Contoso tenant resolver plugin module.
#[modkit::module(
    name = "contoso_tr_plugin",
    deps = ["types_registry"],
)]
pub struct ContosoTrPlugin {
    service: arc_swap::ArcSwapOption<Service>,
}

impl Default for ContosoTrPlugin {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::empty(),
        }
    }
}

#[async_trait]
impl Module for ContosoTrPlugin {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing contoso_tr_plugin");

        // Load configuration
        let cfg: ContosoPluginConfig = ctx.config()?;

        // Generate plugin instance ID
        let instance_id =
            ThrPluginSpec::make_gts_instance_id("contoso.plugins._.thr_plugin.v1").to_string();

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
            vendor: cfg.vendor,
            priority: cfg.priority,
        };
        let instance_json = serde_json::to_value(&instance)?;
        let _ = registry
            .register(&SecurityCtx::root_ctx(), vec![instance_json])
            .await?;

        // Create and store the service
        let service = Arc::new(Service);
        self.service.store(Some(service.clone()));

        // Register scoped client in ClientHub
        let api: Arc<dyn ThrPluginApi> = service;
        ctx.client_hub()
            .register_scoped::<dyn ThrPluginApi>(ClientScope::gts_id(&instance_id), api);

        info!(instance_id = %instance_id, "Contoso plugin initialized");
        Ok(())
    }
}
