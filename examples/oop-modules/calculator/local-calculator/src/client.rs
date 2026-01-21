//! Local CalculatorClient implementation
//!
//! Lazily initializes CalculatorGrpcClient using DirectoryClient from ClientHub.

use std::sync::Arc;

use async_trait::async_trait;
use modkit::client_hub::ClientHub;
use modkit_security::SecurityContext;
use module_orchestrator_sdk::DirectoryClient;
use tokio::sync::RwLock;

use calculator_sdk::{CalculatorClient, CalculatorError, CalculatorGrpcClient, SERVICE_NAME};

/// Local CalculatorClient that lazily initializes the gRPC client.
///
/// On first use, it:
/// 1. Gets `DirectoryClient` from the ClientHub
/// 2. Resolves the CalculatorService endpoint
/// 3. Connects to the gRPC service
/// 4. Caches the client for subsequent requests
///
/// This client is thread-safe and can be shared across multiple threads.
pub struct LocalCalculatorClient {
    client_hub: Arc<ClientHub>,
    grpc_client: RwLock<Option<Arc<CalculatorGrpcClient>>>,
}

impl LocalCalculatorClient {
    /// Create a new LocalCalculatorClient with a reference to ClientHub.
    pub fn new(client_hub: Arc<ClientHub>) -> Self {
        Self {
            client_hub,
            grpc_client: RwLock::new(None),
        }
    }

    /// Get or initialize the gRPC client.
    ///
    /// This method is called on first use and caches the result.
    /// Subsequent calls return the cached client.
    async fn get_client(&self) -> Result<Arc<CalculatorGrpcClient>, CalculatorError> {
        // Fast path: check if already initialized (read lock)
        {
            let guard = self.grpc_client.read().await;
            if let Some(client) = guard.as_ref() {
                return Ok(Arc::clone(client));
            }
        }

        // Slow path: initialize the client (write lock)
        let mut guard = self.grpc_client.write().await;

        // Double-check: another thread might have initialized it while we waited for the write lock
        if let Some(client) = guard.as_ref() {
            return Ok(Arc::clone(client));
        }

        // Initialize the client
        let directory = self
            .client_hub
            .get::<dyn DirectoryClient>()
            .map_err(|e| {
                CalculatorError::Internal(format!(
                    "DirectoryClient not available in ClientHub: {}",
                    e
                ))
            })?;

        // Resolve the service endpoint
        let endpoint = directory
            .resolve_grpc_service(SERVICE_NAME)
            .await
            .map_err(|e| {
                CalculatorError::Internal(format!(
                    "Failed to resolve calculator service endpoint: {}",
                    e
                ))
            })?;

        // Connect to the gRPC service
        let grpc_client = CalculatorGrpcClient::connect(&endpoint.uri)
            .await
            .map_err(|e| {
                CalculatorError::Internal(format!(
                    "Failed to connect to calculator service at {}: {}",
                    endpoint.uri, e
                ))
            })?;

        let grpc_client = Arc::new(grpc_client);

        // Cache the client
        *guard = Some(Arc::clone(&grpc_client));

        Ok(grpc_client)
    }
}

#[async_trait]
impl CalculatorClient for LocalCalculatorClient {
    async fn add(
        &self,
        ctx: &SecurityContext,
        a: i64,
        b: i64,
    ) -> Result<i64, CalculatorError> {
        let client = self.get_client().await?;
        client.add(ctx, a, b).await
    }
}
