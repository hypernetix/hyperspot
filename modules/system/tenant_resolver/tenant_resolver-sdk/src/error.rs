//! Error types for the tenant resolver module.

use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur when using the tenant resolver API.
#[derive(Debug, Error)]
pub enum TenantResolverError {
    /// The requested target tenant was not found.
    #[error("tenant not found: {tenant_id}")]
    TenantNotFound {
        /// The tenant ID that was not found.
        tenant_id: Uuid,
    },

    /// The request is not authorized.
    ///
    /// Reserved for future plugins that implement access control.
    /// Built-in plugins currently use `TenantNotFound` for unauthorized access.
    #[error("unauthorized")]
    Unauthorized,

    /// No plugin is available to handle the request.
    #[error("no plugin available")]
    NoPluginAvailable,

    /// The plugin is not available yet.
    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),

    /// An internal error occurred.
    #[error("internal error: {0}")]
    Internal(String),
}
