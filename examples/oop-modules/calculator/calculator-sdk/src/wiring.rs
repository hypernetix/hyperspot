//! Wiring for Calculator SDK
//!
//! Provides `wire_client` to register the gRPC client into ClientHub.

use std::sync::Arc;

use anyhow::Result;
use cf_system_sdks::directory::DirectoryClient;
use modkit::client_hub::ClientHub;

use crate::SERVICE_NAME;
use crate::api::CalculatorClientV1;
use crate::client::CalculatorGrpcClient;

/// Wire the Calculator gRPC client into the ClientHub.
///
/// This function:
/// 1. Resolves the CalculatorService endpoint from the DirectoryClient
/// 2. Creates a gRPC client
/// 3. Registers it in the ClientHub as `dyn CalculatorClientV1`
///
/// # Example
/// ```ignore
/// use calculator_sdk::{wire_client, CalculatorClientV1};
///
/// wire_client(&hub, &directory_api).await?;
/// let client = hub.get::<dyn CalculatorClientV1>()?;
/// ```
pub async fn wire_client(hub: &ClientHub, resolver: &dyn DirectoryClient) -> Result<()> {
    let endpoint = resolver.resolve_grpc_service(SERVICE_NAME).await?;
    let client = CalculatorGrpcClient::connect(&endpoint.uri).await?;
    hub.register::<dyn CalculatorClientV1>(Arc::new(client));
    tracing::info!(service = SERVICE_NAME, "CalculatorClientV1 client wired");
    Ok(())
}
