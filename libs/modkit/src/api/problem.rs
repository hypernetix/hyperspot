//! Re-exports and convenience constructors for Problem types

use http::StatusCode;

pub use modkit_errors::problem::{
    APPLICATION_PROBLEM_JSON, Problem, ValidationError, ValidationErrorResponse,
    ValidationViolation,
};

// Optional convenience constructors that return `Problem` directly
pub fn bad_request(detail: impl Into<String>) -> Problem {
    Problem::new(StatusCode::BAD_REQUEST, "Bad Request", detail)
}

pub fn not_found(detail: impl Into<String>) -> Problem {
    Problem::new(StatusCode::NOT_FOUND, "Not Found", detail)
}

pub fn conflict(detail: impl Into<String>) -> Problem {
    Problem::new(StatusCode::CONFLICT, "Conflict", detail)
}

pub fn internal_error(detail: impl Into<String>) -> Problem {
    Problem::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal Server Error",
        detail,
    )
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    #[test]
    fn problem_into_response_sets_status_and_content_type() {
        use axum::http::StatusCode;

        let p = Problem::new(StatusCode::BAD_REQUEST, "Bad Request", "invalid payload");
        let resp = p.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let ct = resp
            .headers()
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert_eq!(ct, APPLICATION_PROBLEM_JSON);
    }

    #[test]
    fn problem_builder_pattern() {
        use http::StatusCode;

        let p = Problem::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Validation Failed",
            "Input validation errors",
        )
        .with_code("VALIDATION_ERROR")
        .with_instance("/users/123")
        .with_trace_id("req-456")
        .with_errors(vec![ValidationViolation {
            message: "Email is required".to_owned(),
            field: "email".to_owned(),
            code: None,
        }]);

        assert_eq!(p.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(p.code, "VALIDATION_ERROR");
        assert_eq!(p.instance, "/users/123");
        assert_eq!(p.trace_id, Some("req-456".to_owned()));
        assert!(p.errors.is_some());
        assert_eq!(p.errors.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn convenience_constructors() {
        use http::StatusCode;

        let bad_req = bad_request("Invalid input");
        assert_eq!(bad_req.status, StatusCode::BAD_REQUEST);
        assert_eq!(bad_req.title, "Bad Request");

        let not_found_resp = not_found("User not found");
        assert_eq!(not_found_resp.status, StatusCode::NOT_FOUND);
        assert_eq!(not_found_resp.title, "Not Found");

        let conflict_resp = conflict("Email already exists");
        assert_eq!(conflict_resp.status, StatusCode::CONFLICT);
        assert_eq!(conflict_resp.title, "Conflict");

        let internal_resp = internal_error("Database connection failed");
        assert_eq!(internal_resp.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(internal_resp.title, "Internal Server Error");
    }
}
