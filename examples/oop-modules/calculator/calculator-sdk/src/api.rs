//! Calculator API trait and types
//!
//! Contract trait and types for the calculator service.

use async_trait::async_trait;
use modkit_security::SecurityContext;

/// Calculator API trait
///
/// A simple service that performs addition operations.
/// All methods require a SecurityContext for authorization.
#[async_trait]
pub trait CalculatorClient: Send + Sync {
    /// Add two numbers and return the sum.
    async fn add(&self, ctx: &SecurityContext, a: i64, b: i64) -> Result<i64, CalculatorError>;
}

/// Error type for Calculator operations
#[derive(thiserror::Error, Debug)]
pub enum CalculatorError {
    #[error("gRPC transport error: {0}")]
    Transport(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),
}
