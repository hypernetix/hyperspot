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
            Database { .. } => Self::internal(),
        }
    }
}
