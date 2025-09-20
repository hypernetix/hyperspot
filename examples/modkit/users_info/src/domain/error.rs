use db::odata;
use thiserror::Error;
use uuid::Uuid;

/// Domain-specific errors using thiserror
#[derive(Error, Debug)]
pub enum DomainError {
    #[error("User not found: {id}")]
    UserNotFound { id: Uuid },

    #[error("User with email '{email}' already exists")]
    EmailAlreadyExists { email: String },

    #[error("Invalid email format: '{email}'")]
    InvalidEmail { email: String },

    #[error("Display name cannot be empty")]
    EmptyDisplayName,

    #[error("Display name too long: {len} characters (max: {max})")]
    DisplayNameTooLong { len: usize, max: usize },

    /// Semantic error in $filter (unknown field, wrong type, unsupported fn, etc.)
    #[error("invalid $filter: {0}")]
    InvalidFilter(#[from] odata::ODataBuildError),

    /// Semantic error in $orderby (unknown field, etc.)
    #[error("invalid $orderby: {0}")]
    InvalidOrderBy(String),

    #[error(
        "Order mismatch: cursor order '{cursor_order}' doesn't match query order '{query_order}'"
    )]
    OrderMismatch {
        cursor_order: String,
        query_order: String,
    },

    #[error("Filter mismatch: cursor filter hash '{cursor_hash}' doesn't match query filter hash '{query_hash}'")]
    FilterMismatch {
        cursor_hash: String,
        query_hash: String,
    },

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

    pub fn order_mismatch(
        cursor_order: impl std::fmt::Display,
        query_order: impl std::fmt::Display,
    ) -> Self {
        Self::OrderMismatch {
            cursor_order: cursor_order.to_string(),
            query_order: query_order.to_string(),
        }
    }

    pub fn filter_mismatch(cursor_hash: String, query_hash: String) -> Self {
        Self::FilterMismatch {
            cursor_hash,
            query_hash,
        }
    }

    pub fn invalid_orderby(message: String) -> Self {
        Self::InvalidOrderBy(message)
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
