use modkit::api::problem::Problem;

use crate::domain::error::DomainError;
use crate::errors::ErrorCode;

/// Map domain error to RFC9457 Problem using the catalog
pub fn domain_error_to_problem(e: &DomainError, instance: &str) -> Problem {
    // Extract trace ID from current tracing span if available
    let trace_id = tracing::Span::current()
        .id()
        .map(|id| id.into_u64().to_string());

    match &e {
        DomainError::UserNotFound { id } => ErrorCode::example1_user_not_found_v1().with_context(
            format!("User with id {} was not found", id),
            instance,
            trace_id,
        ),
        DomainError::EmailAlreadyExists { email } => ErrorCode::example1_user_invalid_email_v1()
            .with_context(
                format!("Email '{}' is already in use", email),
                instance,
                trace_id,
            ),
        DomainError::InvalidEmail { email } => ErrorCode::example1_user_invalid_email_v1()
            .with_context(format!("Email '{}' is invalid", email), instance, trace_id),
        DomainError::EmptyDisplayName => ErrorCode::example1_user_validation_v1().with_context(
            "Display name cannot be empty",
            instance,
            trace_id,
        ),
        DomainError::DisplayNameTooLong { .. } | DomainError::Validation { .. } => {
            ErrorCode::example1_user_validation_v1().with_context(
                format!("{}", e),
                instance,
                trace_id,
            )
        }
        DomainError::Database { .. } => {
            // Log the internal error details but don't expose them to the client
            tracing::error!(error = ?e, "Database error occurred");
            ErrorCode::example1_user_internal_database_v1().with_context(
                "An internal database error occurred",
                instance,
                trace_id,
            )
        }
    }
}

/// Implement Into<Problem> for DomainError so it works with ApiError
impl From<DomainError> for Problem {
    fn from(e: DomainError) -> Self {
        domain_error_to_problem(&e, "/")
    }
}
