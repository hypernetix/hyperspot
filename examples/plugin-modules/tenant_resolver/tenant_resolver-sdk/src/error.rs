use serde::{Deserialize, Serialize};

/// Public error type for tenant resolver operations.
#[derive(thiserror::Error, Debug, Clone, Serialize, Deserialize)]
pub enum TenantResolverError {
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("internal error: {0}")]
    Internal(String),
}
