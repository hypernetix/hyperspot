use axum::http::StatusCode;
use modkit::api::problem::{Problem, ProblemResponse};

/// Helper to create a ProblemResponse with less boilerplate
pub fn from_parts(
    status: StatusCode,
    code: &str,
    title: &str,
    detail: impl Into<String>,
    instance: &str,
) -> ProblemResponse {
    let problem = Problem::new(status, title, detail)
        .with_type(format!("https://errors.example.com/{}", code))
        .with_code(code)
        .with_instance(instance);

    // Add request ID from current tracing span if available
    let problem = if let Some(id) = tracing::Span::current().id() {
        problem.with_trace_id(id.into_u64().to_string())
    } else {
        problem
    };

    ProblemResponse(problem)
}

use crate::domain::error::DomainError;

/// Map domain error to RFC9457 ProblemResponse
pub fn map_domain_error(e: &DomainError, instance: &str) -> ProblemResponse {
    match e {
        DomainError::UserNotFound { id } => from_parts(
            StatusCode::NOT_FOUND,
            "USERS_NOT_FOUND",
            "User not found",
            format!("User with id {} was not found", id),
            instance,
        ),
        DomainError::EmailAlreadyExists { email } => from_parts(
            StatusCode::CONFLICT,
            "USERS_EMAIL_CONFLICT",
            "Email already exists",
            format!("Email '{}' is already in use", email),
            instance,
        ),
        DomainError::InvalidEmail { email } => from_parts(
            StatusCode::BAD_REQUEST,
            "USERS_INVALID_EMAIL",
            "Invalid email",
            format!("Email '{}' is invalid", email),
            instance,
        ),
        DomainError::EmptyDisplayName => from_parts(
            StatusCode::BAD_REQUEST,
            "USERS_VALIDATION",
            "Validation error",
            "Display name cannot be empty",
            instance,
        ),
        DomainError::DisplayNameTooLong { .. } | DomainError::Validation { .. } => from_parts(
            StatusCode::BAD_REQUEST,
            "USERS_VALIDATION",
            "Validation error",
            format!("{}", e),
            instance,
        ),
        DomainError::InvalidFilter { .. } => {
            // Log the internal error details but don't expose them to the client
            tracing::error!(error = ?e, "Filter error");
            from_parts(
                StatusCode::BAD_REQUEST,
                "ODATA_FILTER_INVALID",
                "Filter error",
                format!("invalid $filter: {e}"),
                instance,
            )
        }
        DomainError::InvalidOrderBy(msg) => from_parts(
            StatusCode::BAD_REQUEST,
            "ODATA_ORDERBY_INVALID",
            "Orderby error",
            format!("invalid $orderby: {}", msg),
            instance,
        ),
        DomainError::OrderMismatch {
            cursor_order,
            query_order,
        } => from_parts(
            StatusCode::BAD_REQUEST,
            "ORDER_MISMATCH",
            "Order mismatch",
            format!(
                "Cursor order '{}' doesn't match query order '{}'",
                cursor_order, query_order
            ),
            instance,
        ),
        DomainError::FilterMismatch {
            cursor_hash,
            query_hash,
        } => from_parts(
            StatusCode::BAD_REQUEST,
            "FILTER_MISMATCH",
            "Filter mismatch",
            format!(
                "Cursor filter hash '{}' doesn't match query filter hash '{}'",
                cursor_hash, query_hash
            ),
            instance,
        ),
        DomainError::Database { .. } => {
            // Log the internal error details but don't expose them to the client
            tracing::error!(error = ?e, "Database error occurred");
            from_parts(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_DB",
                "Internal error",
                "An internal database error occurred",
                instance,
            )
        }
    }
}
