//! Public error types for the `types` module.
//!
//! These errors are safe to expose to other modules and consumers.

use thiserror::Error;

/// Errors that can be returned by the `TypesClient`.
#[derive(Error, Debug, Clone)]
pub enum TypesError {
    /// Core types are not yet registered.
    #[error("Core types not ready")]
    NotReady,

    /// Failed to register core types.
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),

    /// An internal error occurred.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl TypesError {
    /// Creates a `NotReady` error.
    #[must_use]
    pub const fn not_ready() -> Self {
        Self::NotReady
    }

    /// Creates a `RegistrationFailed` error.
    #[must_use]
    pub fn registration_failed(message: impl Into<String>) -> Self {
        Self::RegistrationFailed(message.into())
    }

    /// Creates an `Internal` error.
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    /// Returns `true` if this is a not ready error.
    #[must_use]
    pub const fn is_not_ready(&self) -> bool {
        matches!(self, Self::NotReady)
    }

    /// Returns `true` if this is a registration failed error.
    #[must_use]
    pub const fn is_registration_failed(&self) -> bool {
        matches!(self, Self::RegistrationFailed(_))
    }

    /// Returns `true` if this is an internal error.
    #[must_use]
    pub const fn is_internal(&self) -> bool {
        matches!(self, Self::Internal(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_constructors() {
        let err = TypesError::not_ready();
        assert!(err.is_not_ready());

        let err = TypesError::registration_failed("schema invalid");
        assert!(err.is_registration_failed());
        assert!(err.to_string().contains("schema invalid"));

        let err = TypesError::internal("unexpected");
        assert!(matches!(err, TypesError::Internal(_)));
    }

    #[test]
    fn test_error_display() {
        let err = TypesError::NotReady;
        assert_eq!(err.to_string(), "Core types not ready");

        let err = TypesError::RegistrationFailed("failed to parse".to_owned());
        assert_eq!(err.to_string(), "Registration failed: failed to parse");

        let err = TypesError::Internal("database error".to_owned());
        assert_eq!(err.to_string(), "Internal error: database error");
    }
}
