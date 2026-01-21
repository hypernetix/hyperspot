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

    /// Access to target tenant is denied.
    ///
    /// The source tenant is available from the security context.
    #[error("access denied to tenant: {target_tenant}")]
    AccessDenied {
        /// The target tenant being accessed.
        target_tenant: Uuid,
    },

    /// The request is not authorized.
    #[error("unauthorized")]
    Unauthorized,

    /// No plugin is available to handle the request.
    #[error("no plugin available")]
    NoPluginAvailable,

    /// An internal error occurred.
    #[error("internal error: {0}")]
    Internal(String),
}
