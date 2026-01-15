#[cfg(test)]
mod tests {
    use super::super::error::domain_error_to_problem;
    use crate::domain::error::DomainError;
    use axum::http::StatusCode;
    use modkit::api::problem::Problem;

    #[test]
    fn test_not_found_error_to_problem() {
        let error = DomainError::NotFound;
        let problem = domain_error_to_problem(&error, "/test/instance");

        assert_eq!(problem.status, StatusCode::NOT_FOUND);
        assert_eq!(problem.instance, "/test/instance");
        assert!(problem.detail.contains("Settings not found"));
    }

    #[test]
    fn test_validation_error_to_problem() {
        let error = DomainError::Validation {
            field: "theme".to_owned(),
            message: "exceeds max length".to_owned(),
        };
        let problem = domain_error_to_problem(&error, "/api/settings");

        assert_eq!(problem.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(problem.instance, "/api/settings");
        assert!(problem.detail.contains("theme"));
        assert!(problem.detail.contains("exceeds max length"));
    }

    #[test]
    fn test_database_error_to_problem() {
        let error = DomainError::Database(anyhow::anyhow!("connection failed"));
        let problem = domain_error_to_problem(&error, "/db/error");

        assert_eq!(problem.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(problem.instance, "/db/error");
        assert!(problem.detail.contains("internal database error"));
    }

    #[test]
    fn test_from_domain_error_for_problem_not_found() {
        let error = DomainError::NotFound;
        let problem: Problem = error.into();

        assert_eq!(problem.status, StatusCode::NOT_FOUND);
        assert_eq!(problem.instance, "/");
    }

    #[test]
    fn test_from_domain_error_for_problem_validation() {
        let error = DomainError::Validation {
            field: "language".to_owned(),
            message: "invalid format".to_owned(),
        };
        let problem: Problem = error.into();

        assert_eq!(problem.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(problem.detail.contains("language"));
    }

    #[test]
    fn test_from_domain_error_for_problem_database() {
        let error = DomainError::Database(anyhow::anyhow!("db error"));
        let problem: Problem = error.into();

        assert_eq!(problem.status, StatusCode::INTERNAL_SERVER_ERROR);
    }
}
