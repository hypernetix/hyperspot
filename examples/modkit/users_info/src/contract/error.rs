use modkit::api::problem::ProblemResponse;
use thiserror::Error;
use uuid::Uuid;

use crate::errors::ErrorCode;

/// Errors that are safe to expose to other modules
#[derive(Error, Debug, Clone)]
pub enum UsersInfoError {
    #[error("User not found: {id}")]
    NotFound { id: Uuid },

    #[error("User with email '{email}' already exists")]
    Conflict { email: String },

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Internal error")]
    Internal,
}

impl UsersInfoError {
    pub fn not_found(id: Uuid) -> Self {
        Self::NotFound { id }
    }

    pub fn conflict(email: String) -> Self {
        Self::Conflict { email }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    pub fn internal() -> Self {
        Self::Internal
    }

    /// Convert to ProblemResponse using the catalog
    pub fn to_problem(&self, instance: &str, trace_id: Option<String>) -> ProblemResponse {
        use UsersInfoError as E;
        match self {
            E::NotFound { id } => ErrorCode::users_info_user_not_found_v1.to_response(
                format!("User with id {} was not found", id),
                instance,
                trace_id,
            ),
            E::Conflict { email } => ErrorCode::users_info_user_email_conflict_v1.to_response(
                format!("Email '{}' is already in use", email),
                instance,
                trace_id,
            ),
            E::Validation { message } => ErrorCode::users_info_user_validation_v1.to_response(
                message.clone(),
                instance,
                trace_id,
            ),
            E::Internal => ErrorCode::users_info_internal_database_v1.to_response(
                "An internal error occurred",
                instance,
                trace_id,
            ),
        }
    }
}

impl From<crate::domain::error::DomainError> for UsersInfoError {
    fn from(domain_error: crate::domain::error::DomainError) -> Self {
        use crate::domain::error::DomainError::*;
        match domain_error {
            UserNotFound { id } => Self::not_found(id),
            EmailAlreadyExists { email } => Self::conflict(email),
            InvalidEmail { email } => Self::validation(format!("Invalid email: {}", email)),
            EmptyDisplayName => Self::validation("Display name cannot be empty".to_string()),
            DisplayNameTooLong { len, max } => Self::validation(format!(
                "Display name too long: {} characters (max: {})",
                len, max
            )),
            Validation { field, message } => Self::validation(format!("{}: {}", field, message)),
            Database { .. } => Self::internal(),
        }
    }
}

// Convert OData errors to contract-safe errors using catalog
impl From<odata_core::Error> for UsersInfoError {
    fn from(odata_error: odata_core::Error) -> Self {
        use odata_core::Error::*;
        match odata_error {
            // Filter and OrderBy parsing errors
            InvalidFilter(msg) => Self::validation(format!("Invalid filter: {}", msg)),
            InvalidOrderByField(field) => {
                Self::validation(format!("Invalid orderby field: {}", field))
            }

            // Pagination and cursor validation errors
            OrderMismatch => Self::validation("Order mismatch".to_string()),
            FilterMismatch => Self::validation("Filter mismatch".to_string()),
            InvalidCursor => Self::validation("Invalid cursor".to_string()),
            InvalidLimit => Self::validation("Invalid limit".to_string()),
            OrderWithCursor => {
                Self::validation("Cannot specify both orderby and cursor".to_string())
            }

            // Cursor parsing errors (all validation issues from client perspective)
            CursorInvalidBase64 => Self::validation("Invalid cursor encoding".to_string()),
            CursorInvalidJson => Self::validation("Malformed cursor data".to_string()),
            CursorInvalidVersion => Self::validation("Unsupported cursor version".to_string()),
            CursorInvalidKeys => Self::validation("Invalid cursor keys".to_string()),
            CursorInvalidFields => Self::validation("Invalid cursor fields".to_string()),
            CursorInvalidDirection => Self::validation("Invalid cursor direction".to_string()),

            // Database and low-level errors (don't expose internal details)
            Db(_) => Self::internal(),
        }
    }
}
