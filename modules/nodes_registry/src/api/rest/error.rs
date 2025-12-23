use crate::domain::error::DomainError;
use axum::http::StatusCode;
use modkit::api::problem::Problem;

/// Map domain errors to HTTP problem responses
pub fn domain_error_to_problem(err: DomainError, instance: &str) -> Problem {
    let trace_id = tracing::Span::current()
        .id()
        .map(|id| id.into_u64().to_string());

    let mut problem = match err {
        DomainError::NodeNotFound(id) => Problem::new(
            StatusCode::NOT_FOUND,
            "Node not found",
            format!("No node with id {id}"),
        )
        .with_type("https://errors.hyperspot.com/NODES_NOT_FOUND")
        .with_code("NODES_NOT_FOUND")
        .with_instance(instance),
        DomainError::SysInfoCollectionFailed(msg) => Problem::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "System information collection failed",
            msg,
        )
        .with_type("https://errors.hyperspot.com/SYSINFO_COLLECTION_FAILED")
        .with_code("SYSINFO_COLLECTION_FAILED")
        .with_instance(instance),
        DomainError::SysCapCollectionFailed(msg) => Problem::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "System capabilities collection failed",
            msg,
        )
        .with_type("https://errors.hyperspot.com/SYSCAP_COLLECTION_FAILED")
        .with_code("SYSCAP_COLLECTION_FAILED")
        .with_instance(instance),
        DomainError::InvalidInput(msg) => {
            Problem::new(StatusCode::BAD_REQUEST, "Validation error", msg)
                .with_type("https://errors.hyperspot.com/VALIDATION_ERROR")
                .with_code("VALIDATION_ERROR")
                .with_instance(instance)
        }
        DomainError::Internal(msg) => Problem::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error",
            msg,
        )
        .with_type("https://errors.hyperspot.com/INTERNAL_ERROR")
        .with_code("INTERNAL_ERROR")
        .with_instance(instance),
    };

    if let Some(tid) = trace_id {
        problem = problem.with_trace_id(tid);
    }

    problem
}

/// Implement Into<Problem> for `DomainError` so `?` works in handlers
impl From<DomainError> for Problem {
    fn from(e: DomainError) -> Self {
        domain_error_to_problem(e, "/")
    }
}
