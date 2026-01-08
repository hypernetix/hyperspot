//! OAGW Default Plugin module definition.

use std::sync::Arc;

use async_trait::async_trait;
use modkit::client_hub::ClientScope;
use modkit::{Module, ModuleCtx};
use modkit_security::SecurityCtx;
use oagw_sdk::{OagwPluginApi, OagwPluginSpecV1};
use tracing::info;
use types_registry_sdk::TypesRegistryApi;

use crate::config::PluginConfig;
use crate::service::HttpPluginService;

/// OAGW Default HTTP Plugin module.
///
/// This plugin provides HTTP/1.1, HTTP/2, and SSE support for the OAGW gateway.
#[modkit::module(
    name = "oagw_default_plugin",
    deps = ["types_registry"]
)]
pub struct OagwDefaultPlugin {
    service: arc_swap::ArcSwapOption<HttpPluginService>,
}

impl Default for OagwDefaultPlugin {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::from(None),
        }
    }
}

#[async_trait]
impl Module for OagwDefaultPlugin {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing OAGW default HTTP plugin");

        // Load plugin configuration
        let cfg: PluginConfig = ctx.config()?;

        // Create plugin service
        let service = Arc::new(HttpPluginService::new(cfg.clone()));
        self.service.store(Some(service.clone()));

        // Generate stable GTS instance ID
        let instance_id = OagwPluginSpecV1::gts_make_instance_id("x.core.oagw.http_plugin.v1");

        // Register plugin INSTANCE in types-registry
        // Note: The plugin SCHEMA is registered by the gateway module
        let registry = ctx.client_hub().get::<dyn TypesRegistryApi>()?;

        // Ensure schema exists before registering instance (plugin may start before gateway).
        let schema_str = OagwPluginSpecV1::gts_schema_with_refs_as_string();
        let schema_json: serde_json::Value = serde_json::from_str(&schema_str)?;
        let _ = registry
            .register(&SecurityCtx::root_ctx(), vec![schema_json])
            .await?;
        info!(
            schema_id = %OagwPluginSpecV1::gts_schema_id(),
            "Ensured OAGW plugin schema registered in types-registry"
        );

        let supported_protocols = service.supported_protocols().to_vec();
        let supported_stream_protocols = service.supported_stream_protocols().to_vec();
        let supported_auth_types = service.supported_auth_types().to_vec();
        let supported_strategies = service.supported_strategies().to_vec();

        info!(
            protocols = ?supported_protocols,
            stream_protocols = ?supported_stream_protocols,
            auth_types = ?supported_auth_types,
            strategies = ?supported_strategies,
            "Registering plugin instance with capabilities"
        );

        // Construct instance JSON with properties nested under "properties" field.
        // The base schema requires "properties" as a field, and the child schema
        // should validate the contents of that field.
        let instance_json = serde_json::json!({
            "id": instance_id.to_string(),
            "vendor": cfg.vendor.clone(),
            "priority": cfg.priority,
            "properties": {
                "supported_protocols": supported_protocols,
                "supported_stream_protocols": supported_stream_protocols,
                "supported_auth_types": supported_auth_types,
                "supported_strategies": supported_strategies,
            }
        });
        info!(
            instance_json = ?instance_json,
            "Registering plugin instance JSON"
        );

        // Register the instance - the registry expects the full BaseModkitPluginV1 wrapper
        let _ = registry
            .register(&SecurityCtx::root_ctx(), vec![instance_json])
            .await?;

        info!(
            instance_id = %instance_id,
            vendor = %cfg.vendor,
            priority = cfg.priority,
            "Registered OAGW plugin instance in types-registry"
        );

        // Register scoped client in ClientHub
        let api: Arc<dyn OagwPluginApi> = service;
        ctx.client_hub()
            .register_scoped::<dyn OagwPluginApi>(ClientScope::gts_id(&instance_id), api);

        info!(
            instance_id = %instance_id,
            "OAGW plugin API registered in ClientHub (scoped)"
        );

        Ok(())
    }
}
