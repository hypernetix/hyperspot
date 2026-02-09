//! Error types for the `AuthN` resolver module.

use thiserror::Error;

/// Errors that can occur when using the `AuthN` resolver API.
#[derive(Debug, Error)]
pub enum AuthNResolverError {
    /// The token is invalid, expired, or malformed.
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    /// No `AuthN` plugin is available to handle the request.
    #[error("no plugin available")]
    NoPluginAvailable,

    /// The plugin is not available yet.
    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),

    /// An internal error occurred.
    #[error("internal error: {0}")]
    Internal(String),
}
