use modkit::api::problem::ProblemResponse;

use crate::domain::error::DomainError;
use crate::errors::ErrorCode;

/// Map domain error to RFC9457 ProblemResponse using the catalog
pub fn domain_error_to_problem(e: DomainError, instance: &str) -> ProblemResponse {
    // Extract trace ID from current tracing span if available
    let trace_id = tracing::Span::current()
        .id()
        .map(|id| id.into_u64().to_string());

    match &e {
        DomainError::UserNotFound { id } => ErrorCode::users_info_user_not_found_v1.to_response(
            format!("User with id {} was not found", id),
            instance,
            trace_id,
        ),
        DomainError::EmailAlreadyExists { email } => ErrorCode::users_info_user_email_conflict_v1
            .to_response(
                format!("Email '{}' is already in use", email),
                instance,
                trace_id,
            ),
        DomainError::InvalidEmail { email } => ErrorCode::users_info_user_invalid_email_v1
            .to_response(format!("Email '{}' is invalid", email), instance, trace_id),
        DomainError::EmptyDisplayName => ErrorCode::users_info_user_validation_v1.to_response(
            "Display name cannot be empty",
            instance,
            trace_id,
        ),
        DomainError::DisplayNameTooLong { .. } | DomainError::Validation { .. } => {
            ErrorCode::users_info_user_validation_v1.to_response(
                format!("{}", e),
                instance,
                trace_id,
            )
        }
        DomainError::Database { .. } => {
            // Log the internal error details but don't expose them to the client
            tracing::error!(error = ?e, "Database error occurred");
            ErrorCode::users_info_internal_database_v1.to_response(
                "An internal database error occurred",
                instance,
                trace_id,
            )
        }
    }
}

/// Implement Into<ProblemResponse> for DomainError so it works with ApiError
impl From<DomainError> for modkit::api::problem::ProblemResponse {
    fn from(e: DomainError) -> Self {
        domain_error_to_problem(e, "/")
    }
}
