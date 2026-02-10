//! Domain service for calculator_gateway
//!
//! Contains business logic for accumulator operations.
//! Resolves calculator client from ClientHub at call time.

use std::sync::Arc;

use calculator_sdk::CalculatorClientV1;
use modkit::client_hub::ClientHub;
use modkit_macros::domain_model;
use modkit_security::SecurityContext;
use tracing::{debug, instrument};

/// Error type for Service operations.
///
/// This is the internal error type. SDK's CalculatorGatewayLocalClient
/// converts these to CalculatorGatewayError for external consumers.
#[domain_model]
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
#[domain_model]
pub struct Service {
    client_hub: Arc<ClientHub>,
}

impl Service {
    /// Create a new service with ClientHub for dependency resolution.
    pub fn new(client_hub: Arc<ClientHub>) -> Self {
        Self { client_hub }
    }

    /// Add two numbers by delegating to calculator service.
    #[instrument(skip(self, ctx), fields(a, b))]
    pub async fn add(&self, ctx: &SecurityContext, a: i64, b: i64) -> Result<i64, ServiceError> {
        debug!("Resolving calculator client from ClientHub");

        let calculator = self
            .client_hub
            .get::<dyn CalculatorClientV1>()
            .map_err(|e| {
                ServiceError::Internal(format!("CalculatorClientV1 not available: {}", e))
            })?;

        debug!("Delegating addition to calculator service");

        let result = calculator
            .add(ctx, a, b)
            .await
            .map_err(|e| ServiceError::RemoteError(e.to_string()))?;

        debug!(result, "Addition completed successfully");
        Ok(result)
    }
}
