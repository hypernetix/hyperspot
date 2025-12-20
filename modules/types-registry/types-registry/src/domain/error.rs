//! Domain error types for the Types Registry module.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use types_registry_sdk::TypesRegistryError;

/// A structured validation error with typed fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationError {
    /// The GTS ID of the entity that failed validation.
    pub gts_id: String,
    /// The validation error message.
    pub message: String,
}

impl ValidationError {
    /// Creates a new validation error.
    #[must_use]
    pub fn new(gts_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            gts_id: gts_id.into(),
            message: message.into(),
        }
    }

    /// Parses a validation error from a string in the format "`gts_id`: message".
    #[must_use]
    pub fn from_string(s: &str) -> Self {
        if let Some((gts_id, message)) = s.split_once(": ") {
            Self::new(gts_id, message)
        } else {
            Self::new("unknown", s)
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.gts_id, self.message)
    }
}

/// Domain-level errors for the Types Registry module.
#[derive(Error, Debug)]
pub enum DomainError {
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

    /// The operation requires production mode but registry is in configuration mode.
    #[error("Not in production mode")]
    NotInProductionMode,

    /// Multiple validation errors occurred during `switch_to_production`.
    #[error("Production commit failed with {} errors", .0.len())]
    ProductionCommitFailed(Vec<ValidationError>),

    /// An internal error occurred.
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl DomainError {
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

    /// Returns the list of validation errors if this is a `ProductionCommitFailed` error.
    #[must_use]
    pub fn validation_errors(&self) -> Option<&[ValidationError]> {
        match self {
            Self::ProductionCommitFailed(errors) => Some(errors),
            _ => None,
        }
    }
}

impl From<DomainError> for TypesRegistryError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::InvalidGtsId(msg) => TypesRegistryError::invalid_gts_id(msg),
            DomainError::NotFound(id) => TypesRegistryError::not_found(id),
            DomainError::AlreadyExists(id) => TypesRegistryError::already_exists(id),
            DomainError::ValidationFailed(msg) => TypesRegistryError::validation_failed(msg),
            DomainError::NotInProductionMode => TypesRegistryError::not_in_production_mode(),
            DomainError::ProductionCommitFailed(errors) => {
                let error_strings: Vec<String> = errors
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect();
                TypesRegistryError::validation_failed(format!(
                    "Production commit failed with {} errors: {}",
                    errors.len(),
                    error_strings.join("; ")
                ))
            }
            DomainError::Internal(e) => TypesRegistryError::internal(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_constructors() {
        let err = DomainError::invalid_gts_id("missing vendor");
        assert!(matches!(err, DomainError::InvalidGtsId(_)));

        let err = DomainError::not_found("gts.acme.core.events.test.v1~");
        assert!(matches!(err, DomainError::NotFound(_)));

        let err = DomainError::already_exists("gts.acme.core.events.test.v1~");
        assert!(matches!(err, DomainError::AlreadyExists(_)));

        let err = DomainError::validation_failed("schema invalid");
        assert!(matches!(err, DomainError::ValidationFailed(_)));
    }

    #[test]
    fn test_domain_to_sdk_error_conversion() {
        let domain_err = DomainError::not_found("gts.test~");
        let sdk_err: TypesRegistryError = domain_err.into();
        assert!(sdk_err.is_not_found());

        let domain_err = DomainError::already_exists("gts.test~");
        let sdk_err: TypesRegistryError = domain_err.into();
        assert!(sdk_err.is_already_exists());

        let domain_err = DomainError::validation_failed("bad schema");
        let sdk_err: TypesRegistryError = domain_err.into();
        assert!(sdk_err.is_validation_failed());

        let domain_err = DomainError::invalid_gts_id("bad format");
        let sdk_err: TypesRegistryError = domain_err.into();
        assert!(sdk_err.is_invalid_gts_id());
    }

    #[test]
    fn test_domain_to_sdk_error_not_in_production() {
        let domain_err = DomainError::NotInProductionMode;
        let sdk_err: TypesRegistryError = domain_err.into();
        assert!(matches!(sdk_err, TypesRegistryError::NotInProductionMode));
    }

    #[test]
    fn test_domain_to_sdk_error_production_commit_failed() {
        let errors = vec![
            ValidationError::new("gts.test1~", "error1"),
            ValidationError::new("gts.test2~", "error2"),
        ];
        let domain_err = DomainError::ProductionCommitFailed(errors);
        let sdk_err: TypesRegistryError = domain_err.into();
        assert!(sdk_err.is_validation_failed());
    }

    #[test]
    fn test_domain_to_sdk_error_internal() {
        let domain_err = DomainError::Internal(anyhow::anyhow!("test error"));
        let sdk_err: TypesRegistryError = domain_err.into();
        assert!(matches!(sdk_err, TypesRegistryError::Internal(_)));
    }

    #[test]
    fn test_error_display() {
        let err = DomainError::InvalidGtsId("bad format".to_owned());
        assert_eq!(err.to_string(), "Invalid GTS ID: bad format");

        let err = DomainError::NotFound("gts.test~".to_owned());
        assert_eq!(err.to_string(), "Entity not found: gts.test~");

        let err = DomainError::AlreadyExists("gts.test~".to_owned());
        assert_eq!(err.to_string(), "Entity already exists: gts.test~");

        let err = DomainError::ValidationFailed("schema invalid".to_owned());
        assert_eq!(err.to_string(), "Validation failed: schema invalid");

        let err = DomainError::NotInProductionMode;
        assert_eq!(err.to_string(), "Not in production mode");

        let err = DomainError::ProductionCommitFailed(vec![
            ValidationError::new("gts.test1~", "error1"),
            ValidationError::new("gts.test2~", "error2"),
            ValidationError::new("gts.test3~", "error3"),
        ]);
        assert_eq!(err.to_string(), "Production commit failed with 3 errors");
    }

    #[test]
    fn test_internal_error_from_anyhow() {
        let anyhow_err = anyhow::anyhow!("test error");
        let domain_err: DomainError = anyhow_err.into();
        assert!(matches!(domain_err, DomainError::Internal(_)));
    }
}
