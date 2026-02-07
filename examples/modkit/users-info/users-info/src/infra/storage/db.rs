//! Database error conversion helpers.

use std::fmt::Display;

use crate::domain::error::DomainError;

/// Convert any displayable error into a `DomainError::Database`.
pub fn db_err(e: impl Display) -> DomainError {
    DomainError::database(e.to_string())
}
