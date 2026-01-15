use modkit::api::problem::Problem;

use crate::domain::error::DomainError;
use crate::errors::ErrorCode;

/// Map domain error to RFC9457 Problem using the GTS error catalog
pub fn domain_error_to_problem(e: &DomainError, instance: &str) -> Problem {
    let trace_id = tracing::Span::current()
        .id()
        .map(|id| id.into_u64().to_string());

    match e {
        DomainError::NotFound => ErrorCode::settings_simple_user_settings_not_found_v1()
            .with_context("Settings not found", instance, trace_id),
        DomainError::Validation { field, message } => {
            ErrorCode::settings_simple_user_settings_validation_v1().with_context(
                format!("Validation error on '{field}': {message}"),
                instance,
                trace_id,
            )
        }
        DomainError::Database(_) => {
            tracing::error!(error = ?e, "Database error occurred");
            ErrorCode::settings_simple_user_settings_internal_database_v1().with_context(
                "An internal database error occurred",
                instance,
                trace_id,
            )
        }
    }
}

/// Implement From<DomainError> for Problem so `?` works in handlers
impl From<DomainError> for Problem {
    fn from(e: DomainError) -> Self {
        domain_error_to_problem(&e, "/")
    }
}
