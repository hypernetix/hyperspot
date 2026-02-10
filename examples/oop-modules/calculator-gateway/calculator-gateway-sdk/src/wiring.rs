//! Wiring utilities for CalculatorGateway SDK
//!
//! Provides `wire_client()` to register CalculatorGatewayClientV1 in ClientHub.

use std::sync::Arc;

use modkit::client_hub::ClientHub;

use calculator_gateway::Service;

use crate::api::CalculatorGatewayClientV1;
use crate::client::CalculatorGatewayLocalClient;

/// Wire the CalculatorGateway client into ClientHub.
///
/// This function retrieves the Service from ClientHub (registered by the module)
/// and creates a CalculatorGatewayLocalClient that implements CalculatorGatewayClientV1.
///
/// # Prerequisites
/// The calculator_gateway module must be initialized before calling this function.
///
/// # Example
/// ```ignore
/// // After module initialization
/// wire_client(&ctx.client_hub())?;
///
/// // Now you can get the client
/// let client = ctx.client_hub().get::<dyn CalculatorGatewayClientV1>()?;
/// ```
pub fn wire_client(hub: &ClientHub) -> anyhow::Result<()> {
    // Get Service from ClientHub (registered by the module in init)
    let service = hub.get::<Service>().map_err(|e| {
        anyhow::anyhow!(
            "Service not available (is calculator_gateway module initialized?): {}",
            e
        )
    })?;

    // Create client that wraps the Service
    let client = CalculatorGatewayLocalClient::new(service);

    // Register as CalculatorGatewayClientV1 trait object
    hub.register::<dyn CalculatorGatewayClientV1>(Arc::new(client));

    tracing::debug!("CalculatorGatewayClientV1 client wired");
    Ok(())
}
