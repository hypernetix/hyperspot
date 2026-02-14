//! Public API trait for the `AuthZ` resolver.

use async_trait::async_trait;

use crate::error::AuthZResolverError;
use crate::models::{EvaluationRequest, EvaluationResponse};

/// Public API trait for the `AuthZ` resolver gateway.
///
/// This trait is registered in `ClientHub` by the module and
/// can be consumed by other modules acting as PEPs:
///
/// ```ignore
/// let authz = hub.get::<dyn AuthZResolverClient>()?;
///
/// let response = authz.evaluate(request).await?;
/// ```
#[async_trait]
pub trait AuthZResolverClient: Send + Sync {
    /// Evaluate an authorization request.
    ///
    /// Returns a decision (allow/deny) with optional row-level constraints.
    ///
    /// # Errors
    ///
    /// - `Denied` if the PDP explicitly denies access
    /// - `NoPluginAvailable` if no `AuthZ` plugin is registered
    /// - `ServiceUnavailable` if the plugin is not ready
    /// - `Internal` for unexpected errors
    async fn evaluate(
        &self,
        request: EvaluationRequest,
    ) -> Result<EvaluationResponse, AuthZResolverError>;
}
