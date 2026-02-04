use modkit_db::DbError;
use simple_user_settings_sdk::errors::SettingsError;

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Settings not found")]
    NotFound,

    #[error("Validation error on field '{field}': {message}")]
    Validation { field: String, message: String },

    #[error("Access forbidden: {0}")]
    Forbidden(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Database(#[from] DbError),
}

impl DomainError {
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden(message.into())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

impl From<DomainError> for SettingsError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::NotFound => Self::not_found(),
            DomainError::Validation { field, message } => Self::validation(field, message),
            DomainError::Forbidden(_) => Self::forbidden(),
            DomainError::Internal(_) | DomainError::Database(_) => Self::internal(),
        }
    }
}
