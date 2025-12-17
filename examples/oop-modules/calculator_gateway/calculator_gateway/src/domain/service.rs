//! Domain service for calculator_gateway
//!
//! Contains business logic for accumulator operations.
//! Resolves calculator client from ClientHub at call time.

use std::sync::Arc;

use calculator_sdk::CalculatorClient;
use modkit::client_hub::ClientHub;
use tracing::{debug, instrument};
use modkit_security::PolicyEngine;

/// Error type for Service operations.
///
/// This is the internal error type. SDK's CalculatorGatewayLocalClient
/// converts these to CalculatorGatewayError for external consumers.
#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    /// Remote service call failed
    #[error("remote service error: {0}")]
    RemoteError(String),

    /// Internal processing error
    #[error("internal error: {0}")]
    Internal(String),
}

/// Domain service that orchestrates accumulator operations.
///
/// Holds a reference to ClientHub for resolving dependencies at call time.
pub struct Service {
    client_hub: Arc<ClientHub>,
}

impl Service {
    /// Create a new service with ClientHub for dependency resolution.
    pub fn new(client_hub: Arc<ClientHub>) -> Self {
        Self { client_hub }
    }

    /// Add two numbers by delegating to calculator service.
    #[instrument(skip(self, pe), fields(a, b))]
    pub async fn add(&self, pe: Arc<dyn PolicyEngine>, a: i64, b: i64) -> Result<i64, ServiceError> {
        debug!("Resolving calculator client from ClientHub");

        let calculator = self.client_hub.get::<dyn CalculatorClient>().map_err(|e| {
            ServiceError::Internal(format!("CalculatorClient not available: {}", e))
        })?;

        debug!("Delegating addition to calculator service");

        let result = calculator
            .add(pe, a, b)
            .await
            .map_err(|e| ServiceError::RemoteError(e.to_string()))?;

        debug!(result, "Addition completed successfully");
        Ok(result)
    }
}
