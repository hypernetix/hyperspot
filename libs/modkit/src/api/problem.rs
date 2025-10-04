//! Re-exports and convenience constructors for Problem types

pub use modkit_errors::problem::{
    Problem, ValidationError, ValidationErrorResponse, ValidationViolation,
    APPLICATION_PROBLEM_JSON,
};

// Optional convenience constructors that return `Problem` directly
pub fn bad_request(detail: impl Into<String>) -> Problem {
    Problem::new(400, "Bad Request", detail)
}

pub fn not_found(detail: impl Into<String>) -> Problem {
    Problem::new(404, "Not Found", detail)
}

pub fn conflict(detail: impl Into<String>) -> Problem {
    Problem::new(409, "Conflict", detail)
}

pub fn internal_error(detail: impl Into<String>) -> Problem {
    Problem::new(500, "Internal Server Error", detail)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    #[test]
    fn problem_into_response_sets_status_and_content_type() {
        use axum::http::StatusCode;

        let p = Problem::new(400, "Bad Request", "invalid payload");
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
        let p = Problem::new(422, "Validation Failed", "Input validation errors")
            .with_code("VALIDATION_ERROR")
            .with_instance("/users/123")
            .with_trace_id("req-456")
            .with_errors(vec![ValidationViolation {
                message: "Email is required".to_string(),
                pointer: "/email".to_string(),
                code: None,
            }]);

        assert_eq!(p.status, 422);
        assert_eq!(p.code, "VALIDATION_ERROR");
        assert_eq!(p.instance, "/users/123");
        assert_eq!(p.trace_id, Some("req-456".to_string()));
        assert!(p.errors.is_some());
        assert_eq!(p.errors.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn convenience_constructors() {
        let bad_req = bad_request("Invalid input");
        assert_eq!(bad_req.status, 400);
        assert_eq!(bad_req.title, "Bad Request");

        let not_found_resp = not_found("User not found");
        assert_eq!(not_found_resp.status, 404);
        assert_eq!(not_found_resp.title, "Not Found");

        let conflict_resp = conflict("Email already exists");
        assert_eq!(conflict_resp.status, 409);
        assert_eq!(conflict_resp.title, "Conflict");

        let internal_resp = internal_error("Database connection failed");
        assert_eq!(internal_resp.status, 500);
        assert_eq!(internal_resp.title, "Internal Server Error");
    }
}
