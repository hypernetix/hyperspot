use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use modkit::context::ModuleCtx;
use modkit::contracts::{OpenApiRegistry, RestfulModule};
use modkit::Module;

use crate::contract::client::NodesRegistryApi;
use crate::domain::service::Service;
use crate::gateways::local::NodesRegistryLocalClient;

/// Nodes Registry Module
///
/// Manages node information in the Hyperspot deployment.
/// Provides REST API endpoints for:
/// - Listing nodes
/// - Getting node details
/// - Accessing node system information (sysinfo)
/// - Accessing node system capabilities (syscap)
#[modkit::module(
    name = "nodes_registry",
    capabilities = [rest],
    client = crate::contract::client::NodesRegistryApi
)]
pub struct NodesRegistry {
    service: arc_swap::ArcSwapOption<Service>,
}

impl Default for NodesRegistry {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::empty(),
        }
    }
}

#[async_trait]
impl Module for NodesRegistry {
    async fn init(&self, ctx: &ModuleCtx) -> Result<()> {
        // let cfg: NodesRegistryConfig = ctx.config()?; not needed for now

        // Create the service
        let service = Service::new();
        self.service.store(Some(Arc::new(service.clone())));

        // Expose the client to the ClientHub
        let api: Arc<dyn NodesRegistryApi> =
            Arc::new(NodesRegistryLocalClient::new(Arc::new(service)));
        expose_nodes_registry_client(ctx, &api)?;

        tracing::info!("Nodes registry module initialized");
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl RestfulModule for NodesRegistry {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> Result<axum::Router> {
        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();

        let router = crate::api::rest::routes::register_routes(router, openapi, service)?;

        tracing::info!("Nodes registry REST routes registered");
        Ok(router)
    }
}
