//! Public error types for the `user_info` module.
//!
//! These errors are safe to expose to other modules and consumers.

use thiserror::Error;
use uuid::Uuid;

/// Errors that can be returned by the `UsersInfoClient`.
#[derive(Error, Debug, Clone)]
pub enum UsersInfoError {
    /// Resource with the specified ID was not found.
    #[error("Resource not found: {id}")]
    NotFound { id: Uuid },

    /// A resource with the specified identifier already exists.
    #[error("Resource with identifier '{identifier}' already exists")]
    Conflict { identifier: String },

    /// Validation error with the provided data.
    #[error("Validation error: {message}")]
    Validation { message: String },

    /// An internal error occurred.
    #[error("Internal error")]
    Internal,

    /// Feature not yet implemented.
    #[error("Feature not implemented")]
    NotImplemented,

    /// Streaming or pagination failure in cursor-based APIs.
    #[error("Streaming error: {message}")]
    Streaming { message: String },
}

impl UsersInfoError {
    /// Create a `NotFound` error.
    #[must_use]
    pub fn not_found(id: Uuid) -> Self {
        Self::NotFound { id }
    }

    /// Create a Conflict error.
    pub fn conflict(identifier: impl Into<String>) -> Self {
        Self::Conflict {
            identifier: identifier.into(),
        }
    }

    /// Create a Validation error.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create an Internal error.
    #[must_use]
    pub fn internal() -> Self {
        Self::Internal
    }

    /// Create a `NotImplemented` error.
    #[must_use]
    pub fn not_implemented() -> Self {
        Self::NotImplemented
    }

    /// Create a streaming error.
    pub fn streaming(message: impl Into<String>) -> Self {
        Self::Streaming {
            message: message.into(),
        }
    }
}
