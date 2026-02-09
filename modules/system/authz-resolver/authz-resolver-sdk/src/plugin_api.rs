//! Plugin API trait for `AuthZ` resolver implementations.

use async_trait::async_trait;

use crate::error::AuthZResolverError;
use crate::models::{EvaluationRequest, EvaluationResponse};

/// Plugin API trait for `AuthZ` resolver implementations.
///
/// Each plugin registers this trait with a scoped `ClientHub` entry
/// using its GTS instance ID as the scope.
#[async_trait]
pub trait AuthZResolverPluginClient: Send + Sync {
    /// Evaluate an authorization request.
    ///
    /// # Errors
    ///
    /// - `Internal` for unexpected errors
    async fn evaluate(
        &self,
        request: EvaluationRequest,
    ) -> Result<EvaluationResponse, AuthZResolverError>;
}
