//! Error types for the settings SDK.

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum SettingsError {
    #[error("Settings not found")]
    NotFound,

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Internal error")]
    Internal,
}

impl SettingsError {
    #[must_use]
    pub fn not_found() -> Self {
        Self::NotFound
    }

    #[must_use]
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn internal() -> Self {
        Self::Internal
    }
}
