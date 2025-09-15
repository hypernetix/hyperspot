use thiserror::Error;
use uuid::Uuid;

/// Domain-specific errors using thiserror
#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Setting for : {id}")]
    UserNotFound { id: Uuid },

    #[error("User with email '{email}' already exists")]
    EmailAlreadyExists { email: String },

    #[error("Invalid email format: '{email}'")]
    InvalidEmail { email: String },

    #[error("Display name cannot be empty")]
    EmptyDisplayName,

    #[error("Display name too long: {len} characters (max: {max})")]
    DisplayNameTooLong { len: usize, max: usize },

    #[error("Database error: {message}")]
    Database { message: String },

    #[error("Validation failed: {field}: {message}")]
    Validation { field: String, message: String },
}

impl DomainError {
    pub fn user_not_found(id: Uuid) -> Self {
        Self::UserNotFound { id }
    }

    pub fn email_already_exists(email: String) -> Self {
        Self::EmailAlreadyExists { email }
    }

    pub fn invalid_email(email: String) -> Self {
        Self::InvalidEmail { email }
    }

    pub fn empty_display_name() -> Self {
        Self::EmptyDisplayName
    }

    pub fn display_name_too_long(len: usize, max: usize) -> Self {
        Self::DisplayNameTooLong { len, max }
    }

    pub fn database(message: impl Into<String>) -> Self {
        Self::Database {
            message: message.into(),
        }
    }

    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }
}
