use modkit::api::problem::Problem;

use crate::domain::error::DomainError;
use crate::errors::ErrorCode;

/// Map domain error to RFC9457 Problem using the GTS error catalog
pub fn domain_error_to_problem(e: &DomainError, instance: &str) -> Problem {
    let trace_id = tracing::Span::current()
        .id()
        .map(|id| id.into_u64().to_string());

    match e {
        DomainError::NotFound => build_not_found_problem(instance, trace_id),
        DomainError::Validation { field, message } => {
            build_validation_problem(field, message, instance, trace_id)
        }
        DomainError::Forbidden(msg) => build_forbidden_problem(e, msg, instance, trace_id),
        DomainError::Internal(msg) => build_internal_problem(e, msg, instance, trace_id),
        DomainError::Database(_) => build_database_problem(e, instance, trace_id),
    }
}

fn build_not_found_problem(instance: &str, trace_id: Option<String>) -> Problem {
    ErrorCode::settings_simple_user_settings_not_found_v1().with_context(
        "Settings not found",
        instance,
        trace_id,
    )
}

fn build_validation_problem(
    field: &str,
    message: &str,
    instance: &str,
    trace_id: Option<String>,
) -> Problem {
    ErrorCode::settings_simple_user_settings_validation_v1().with_context(
        format!("Validation error on '{field}': {message}"),
        instance,
        trace_id,
    )
}

fn build_forbidden_problem(
    e: &DomainError,
    msg: &str,
    instance: &str,
    trace_id: Option<String>,
) -> Problem {
    tracing::warn!(error = ?e, "Access forbidden: {}", msg);
    // TODO: Add settings_simple_user_settings_forbidden_v1 to errors catalog
    // For now, use not_found to avoid exposing sensitive scope information
    ErrorCode::settings_simple_user_settings_not_found_v1().with_context(
        "Settings not found or not accessible",
        instance,
        trace_id,
    )
}

fn build_internal_problem(
    e: &DomainError,
    msg: &str,
    instance: &str,
    trace_id: Option<String>,
) -> Problem {
    tracing::error!(error = ?e, "Internal error: {}", msg);
    ErrorCode::settings_simple_user_settings_internal_database_v1().with_context(
        "An internal error occurred",
        instance,
        trace_id,
    )
}

fn build_database_problem(e: &DomainError, instance: &str, trace_id: Option<String>) -> Problem {
    tracing::error!(error = ?e, "Database error occurred");
    ErrorCode::settings_simple_user_settings_internal_database_v1().with_context(
        "An internal database error occurred",
        instance,
        trace_id,
    )
}

/// Implement From<DomainError> for Problem so `?` works in handlers
impl From<DomainError> for Problem {
    fn from(e: DomainError) -> Self {
        domain_error_to_problem(&e, "/")
    }
}
