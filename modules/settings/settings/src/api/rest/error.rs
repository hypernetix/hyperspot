use axum::http::StatusCode;
use modkit::api::problem::Problem;

use crate::domain::error::DomainError;

impl From<DomainError> for Problem {
    fn from(e: DomainError) -> Self {
        let trace_id = tracing::Span::current()
            .id()
            .map(|id| id.into_u64().to_string());

        let (status, code, title, detail) = match &e {
            DomainError::NotFound => (
                StatusCode::NOT_FOUND,
                "SETTINGS_NOT_FOUND",
                "Settings not found",
                "Settings not found".to_owned(),
            ),
            DomainError::Validation { field, message } => (
                StatusCode::BAD_REQUEST,
                "SETTINGS_VALIDATION",
                "Bad Request",
                format!("Validation error on '{field}': {message}"),
            ),
            DomainError::Database(_) => {
                tracing::error!(error = ?e, "Database error occurred");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "SETTINGS_INTERNAL",
                    "Internal Server Error",
                    "An internal error occurred".to_owned(),
                )
            }
        };

        let mut problem = Problem::new(status, title, detail)
            .with_type(format!("https://errors.hyperspot.com/{code}"))
            .with_code(code);

        if let Some(id) = trace_id {
            problem = problem.with_trace_id(id);
        }

        problem
    }
}
