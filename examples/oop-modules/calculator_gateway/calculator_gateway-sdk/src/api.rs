//! API trait and error types for CalculatorGateway

use async_trait::async_trait;
use modkit_security::SecurityContext;

/// Calculator Gateway API trait
///
/// A simple service that performs addition operations.
/// All methods require a SecurityContext for authorization.
///
/// This trait is implemented by `CalculatorGatewayLocalClient` which
/// delegates to the module's internal Service.
#[async_trait]
pub trait CalculatorGatewayClient: Send + Sync {
    /// Add two numbers and return the sum.
    async fn add(
        &self,
        ctx: &SecurityContext,
        a: i64,
        b: i64,
    ) -> Result<i64, CalculatorGatewayError>;
}

/// Error type for CalculatorGateway operations
#[derive(thiserror::Error, Debug, Clone)]
pub enum CalculatorGatewayError {
    /// Remote service call failed
    #[error("remote service error: {0}")]
    RemoteError(String),

    /// Internal processing error
    #[error("internal error: {0}")]
    Internal(String),

    /// Authorization failed
    #[error("unauthorized: {0}")]
    Unauthorized(String),
}
