use simple_user_settings_sdk::errors::SettingsError;

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Settings not found")]
    NotFound,

    #[error("Validation error on field '{field}': {message}")]
    Validation { field: String, message: String },

    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),
}

impl DomainError {
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }
}

impl From<DomainError> for SettingsError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::NotFound => Self::not_found(),
            DomainError::Validation { field, message } => {
                Self::validation(format!("{field}: {message}"))
            }
            DomainError::Database(_) => Self::internal(),
        }
    }
}
