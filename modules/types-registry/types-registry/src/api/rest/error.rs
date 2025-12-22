//! REST error mapping for the Types Registry module.

use modkit::api::prelude::StatusCode;
use modkit::api::problem::Problem;

use crate::domain::error::DomainError;

impl From<DomainError> for Problem {
    fn from(e: DomainError) -> Self {
        let trace_id = tracing::Span::current()
            .id()
            .map(|id| id.into_u64().to_string());

        let (status, code, title, detail) = match &e {
            DomainError::InvalidGtsId(msg) => (
                StatusCode::BAD_REQUEST,
                "TYPES_REGISTRY_INVALID_GTS_ID",
                "Invalid GTS ID",
                msg.clone(),
            ),
            DomainError::NotFound(id) => (
                StatusCode::NOT_FOUND,
                "TYPES_REGISTRY_NOT_FOUND",
                "Entity not found",
                format!("No entity with GTS ID: {id}"),
            ),
            DomainError::AlreadyExists(id) => (
                StatusCode::CONFLICT,
                "TYPES_REGISTRY_ALREADY_EXISTS",
                "Entity already exists",
                format!("Entity with GTS ID already exists: {id}"),
            ),
            DomainError::ValidationFailed(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "TYPES_REGISTRY_VALIDATION_FAILED",
                "Validation failed",
                msg.clone(),
            ),
            DomainError::NotInReadyMode => (
                StatusCode::SERVICE_UNAVAILABLE,
                "TYPES_REGISTRY_NOT_READY",
                "Service not ready",
                "The types registry is not yet ready".to_owned(),
            ),
            DomainError::ReadyCommitFailed(errors) => {
                let error_strings: Vec<String> = errors
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect();
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "TYPES_REGISTRY_ACTIVATION_FAILED",
                    "Registry activation failed",
                    format!(
                        "Failed to activate registry: {} validation errors: {}",
                        errors.len(),
                        error_strings.join("; ")
                    ),
                )
            }
            DomainError::Internal(e) => {
                tracing::error!(error = ?e, "Internal error in types_registry");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "TYPES_REGISTRY_INTERNAL",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_error_to_problem_not_found() {
        let err = DomainError::not_found("gts.test~");
        let problem: Problem = err.into();
        assert_eq!(problem.status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_domain_error_to_problem_already_exists() {
        let err = DomainError::already_exists("gts.test~");
        let problem: Problem = err.into();
        assert_eq!(problem.status, StatusCode::CONFLICT);
    }

    #[test]
    fn test_domain_error_to_problem_invalid_gts_id() {
        let err = DomainError::invalid_gts_id("bad format");
        let problem: Problem = err.into();
        assert_eq!(problem.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_domain_error_to_problem_validation_failed() {
        let err = DomainError::validation_failed("schema invalid");
        let problem: Problem = err.into();
        assert_eq!(problem.status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn test_domain_error_to_problem_not_in_ready_mode() {
        let err = DomainError::NotInReadyMode;
        let problem: Problem = err.into();
        assert_eq!(problem.status, StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_domain_error_to_problem_ready_commit_failed() {
        use crate::domain::error::ValidationError;
        let err = DomainError::ReadyCommitFailed(vec![
            ValidationError::new("gts.test1~", "error1"),
            ValidationError::new("gts.test2~", "error2"),
            ValidationError::new("gts.test3~", "error3"),
        ]);
        let problem: Problem = err.into();
        assert_eq!(problem.status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_domain_error_to_problem_internal() {
        let err = DomainError::Internal(anyhow::anyhow!("test error"));
        let problem: Problem = err.into();
        assert_eq!(problem.status, StatusCode::INTERNAL_SERVER_ERROR);
    }
}
