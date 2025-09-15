use axum::http::StatusCode;
use modkit::{Problem, ProblemResponse};

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
        problem.with_request_id(id.into_u64().to_string())
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
