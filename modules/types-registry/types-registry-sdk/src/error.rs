//! Public error types for the `types-registry` module.
//!
//! These errors are safe to expose to other modules and consumers.

use thiserror::Error;

/// Errors that can be returned by the `TypesRegistryApi`.
#[derive(Error, Debug, Clone)]
pub enum TypesRegistryError {
    /// The GTS ID format is invalid.
    #[error("Invalid GTS ID: {0}")]
    InvalidGtsId(String),

    /// The requested entity was not found.
    #[error("Entity not found: {0}")]
    NotFound(String),

    /// An entity with the same GTS ID already exists.
    #[error("Entity already exists: {0}")]
    AlreadyExists(String),

    /// Validation of the entity content failed.
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// The operation requires ready mode.
    #[error("Not in ready mode")]
    NotInReadyMode,

    /// An internal error occurred.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl TypesRegistryError {
    /// Creates an `InvalidGtsId` error.
    #[must_use]
    pub fn invalid_gts_id(message: impl Into<String>) -> Self {
        Self::InvalidGtsId(message.into())
    }

    /// Creates a `NotFound` error.
    #[must_use]
    pub fn not_found(gts_id: impl Into<String>) -> Self {
        Self::NotFound(gts_id.into())
    }

    /// Creates an `AlreadyExists` error.
    #[must_use]
    pub fn already_exists(gts_id: impl Into<String>) -> Self {
        Self::AlreadyExists(gts_id.into())
    }

    /// Creates a `ValidationFailed` error.
    #[must_use]
    pub fn validation_failed(message: impl Into<String>) -> Self {
        Self::ValidationFailed(message.into())
    }

    /// Creates a `NotInReadyMode` error.
    #[must_use]
    pub const fn not_in_ready_mode() -> Self {
        Self::NotInReadyMode
    }

    /// Creates an `Internal` error.
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    /// Returns `true` if this is a not found error.
    #[must_use]
    pub const fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }

    /// Returns `true` if this is an already exists error.
    #[must_use]
    pub const fn is_already_exists(&self) -> bool {
        matches!(self, Self::AlreadyExists(_))
    }

    /// Returns `true` if this is a validation error.
    #[must_use]
    pub const fn is_validation_failed(&self) -> bool {
        matches!(self, Self::ValidationFailed(_))
    }

    /// Returns `true` if this is an invalid GTS ID error.
    #[must_use]
    pub const fn is_invalid_gts_id(&self) -> bool {
        matches!(self, Self::InvalidGtsId(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_constructors() {
        let err = TypesRegistryError::invalid_gts_id("missing vendor");
        assert!(err.is_invalid_gts_id());
        assert!(err.to_string().contains("missing vendor"));

        let err = TypesRegistryError::not_found("gts.acme.core.events.test.v1~");
        assert!(err.is_not_found());

        let err = TypesRegistryError::already_exists("gts.acme.core.events.test.v1~");
        assert!(err.is_already_exists());

        let err = TypesRegistryError::validation_failed("schema invalid");
        assert!(err.is_validation_failed());

        let err = TypesRegistryError::not_in_ready_mode();
        assert!(matches!(err, TypesRegistryError::NotInReadyMode));

        let err = TypesRegistryError::internal("database error");
        assert!(matches!(err, TypesRegistryError::Internal(_)));
    }

    #[test]
    fn test_error_display() {
        let err = TypesRegistryError::InvalidGtsId("bad format".to_owned());
        assert_eq!(err.to_string(), "Invalid GTS ID: bad format");

        let err = TypesRegistryError::NotFound("gts.x.core.events.test.v1~".to_owned());
        assert_eq!(
            err.to_string(),
            "Entity not found: gts.x.core.events.test.v1~"
        );

        let err = TypesRegistryError::AlreadyExists("gts.x.core.events.test.v1~".to_owned());
        assert_eq!(
            err.to_string(),
            "Entity already exists: gts.x.core.events.test.v1~"
        );

        let err = TypesRegistryError::ValidationFailed("missing required field".to_owned());
        assert_eq!(err.to_string(), "Validation failed: missing required field");

        let err = TypesRegistryError::NotInReadyMode;
        assert_eq!(err.to_string(), "Not in ready mode");

        let err = TypesRegistryError::Internal("unexpected".to_owned());
        assert_eq!(err.to_string(), "Internal error: unexpected");
    }
}
