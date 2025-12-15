//! Public error types for the `user_info` module.
//!
//! These errors are safe to expose to other modules and consumers.

use thiserror::Error;
use uuid::Uuid;

/// Errors that can be returned by the `UsersInfoApi`.
#[derive(Error, Debug, Clone)]
pub enum UsersInfoError {
    /// User with the specified ID was not found.
    #[error("User not found: {id}")]
    NotFound { id: Uuid },

    /// A user with the specified email already exists.
    #[error("User with email '{email}' already exists")]
    Conflict { email: String },

    /// Validation error with the provided data.
    #[error("Validation error: {message}")]
    Validation { message: String },

    /// An internal error occurred.
    #[error("Internal error")]
    Internal,
}

impl UsersInfoError {
    /// Create a `NotFound` error.
    #[must_use]
    pub fn not_found(id: Uuid) -> Self {
        Self::NotFound { id }
    }

    /// Create a Conflict error.
    pub fn conflict(email: impl Into<String>) -> Self {
        Self::Conflict {
            email: email.into(),
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
}
