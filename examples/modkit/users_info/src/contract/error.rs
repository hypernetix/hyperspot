use thiserror::Error;
use uuid::Uuid;

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
            InvalidFilter { .. } => Self::validation("Invalid filter".to_string()),
            InvalidOrderBy(msg) => Self::validation(format!("Invalid orderby: {}", msg)),
            Database { .. } => Self::internal(),
            OrderMismatch {
                cursor_order,
                query_order,
            } => Self::validation(format!(
                "Order mismatch: cursor order '{}' doesn't match query order '{}'",
                cursor_order, query_order
            )),
            FilterMismatch {
                cursor_hash,
                query_hash,
            } => Self::validation(format!(
                "Filter mismatch: cursor filter hash '{}' doesn't match query filter hash '{}'",
                cursor_hash, query_hash
            )),
        }
    }
}

impl From<odata_core::ODataPageError> for UsersInfoError {
    fn from(page_error: odata_core::ODataPageError) -> Self {
        use odata_core::ODataPageError::*;
        match page_error {
            InvalidFilter(msg) => Self::validation(format!("Invalid filter: {}", msg)),
            InvalidOrderByField(field) => {
                Self::validation(format!("Invalid orderby field: {}", field))
            }
            OrderMismatch => Self::validation("Order mismatch".to_string()),
            FilterMismatch => Self::validation("Filter mismatch".to_string()),
            InvalidCursor => Self::validation("Invalid cursor".to_string()),
            InvalidLimit => Self::validation("Invalid limit".to_string()),
            OrderWithCursor => {
                Self::validation("Cannot specify both orderby and cursor".to_string())
            }
            Db(_) => Self::internal(),
        }
    }
}
