//! Calculator Gateway Module definition
//!
//! An in-process module that exposes a REST API for addition.
//! It delegates the actual computation to the calculator OoP service via gRPC.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use axum::Router;

use modkit::api::OpenApiRegistry;
use modkit::context::ModuleCtx;
use modkit::contracts::RestApiCapability;

use crate::api::rest::routes;
use crate::domain::Service;

/// Calculator gateway module.
///
/// Exposes a REST API delegating to calculator service.
/// Registers Service in ClientHub for SDK consumers to access.
#[modkit::module(
    name = "calculator_gateway",
    capabilities = [rest],
    deps = ["calculator"]
)]
pub struct CalculatorGateway;

impl Default for CalculatorGateway {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl modkit::Module for CalculatorGateway {
    async fn init(&self, ctx: &ModuleCtx) -> Result<()> {
        tracing::info!("Initializing calculator_gateway module");

        // Create domain service with ClientHub for dependency resolution
        let service = Arc::new(Service::new(ctx.client_hub()));

        // Register Service in ClientHub for SDK's wire_client() to access
        ctx.client_hub().register::<Service>(service);

        tracing::info!("calculator_gateway module initialized");
        Ok(())
    }
}

impl RestApiCapability for CalculatorGateway {
    fn register_rest(
        &self,
        ctx: &ModuleCtx,
        router: Router,
        openapi: &dyn OpenApiRegistry,
    ) -> Result<Router> {
        tracing::info!("Registering calculator_gateway REST routes");

        // Get Service from ClientHub (registered in init)
        let service = ctx
            .client_hub()
            .get::<Service>()
            .map_err(|e| anyhow::anyhow!("Service not available: {}", e))?;

        let router = routes::register_routes(router, openapi, service)?;

        tracing::info!("calculator_gateway REST routes registered");
        Ok(router)
    }
}
