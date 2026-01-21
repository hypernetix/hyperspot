//! Local Calculator Module definition
//!
//! A module that registers LocalCalculatorClient in the ClientHub.
//! The client lazily initializes the gRPC connection to the calculator service
//! using DirectoryClient for service discovery.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use modkit::context::ModuleCtx;

use calculator_sdk::CalculatorClient;

use crate::client::LocalCalculatorClient;

/// Local Calculator module.
///
/// Registers LocalCalculatorClient in ClientHub for other modules to use.
/// The client lazily initializes the gRPC connection on first use.
#[modkit::module(
    name = "local_calculator",
    capabilities = []
)]
pub struct LocalCalculatorModule;

impl Default for LocalCalculatorModule {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl modkit::Module for LocalCalculatorModule {
    async fn init(&self, ctx: &ModuleCtx) -> Result<()> {
        tracing::info!("Initializing local_calculator module");

        // Register LocalCalculatorClient in ClientHub
        // This lazily initializes the gRPC client on first use
        let calculator_client = Arc::new(LocalCalculatorClient::new(ctx.client_hub()));
        ctx.client_hub()
            .register::<dyn CalculatorClient>(calculator_client);

        tracing::info!("local_calculator module initialized");
        Ok(())
    }
}
