//! Error types for the `AuthZ` resolver module.

use thiserror::Error;

/// Errors that can occur when using the `AuthZ` resolver API.
#[derive(Debug, Error)]
pub enum AuthZResolverError {
    /// Access was explicitly denied by the PDP.
    #[error("access denied")]
    Denied,

    /// No `AuthZ` plugin is available to handle the request.
    #[error("no plugin available")]
    NoPluginAvailable,

    /// The plugin is not available yet.
    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),

    /// An internal error occurred.
    #[error("internal error: {0}")]
    Internal(String),
}
