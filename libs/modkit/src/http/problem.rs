use axum::{
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Content type for Problem Details as per RFC 9457.
pub const APPLICATION_PROBLEM_JSON: &str = "application/problem+json";

/// RFC 9457 Problem Details for HTTP APIs.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(
    title = "Problem",
    description = "RFC 9457 Problem Details for HTTP APIs"
)]
pub struct Problem {
    /// A URI reference that identifies the problem type.
    /// When dereferenced, it might provide human-readable documentation.
    #[serde(rename = "type")]
    pub type_url: String,
    /// A short, human-readable summary of the problem type.
    pub title: String,
    /// The HTTP status code for this occurrence of the problem.
    pub status: u16,
    /// A human-readable explanation specific to this occurrence of the problem.
    pub detail: String,
    /// A URI reference that identifies the specific occurrence of the problem.
    pub instance: String,
    /// Optional machine-readable error code defined by the application.
    pub code: String,
    /// Optional request id useful for tracing.
    pub request_id: Option<String>,
    /// Optional validation errors for 4xx problems.
    pub errors: Option<Vec<ValidationError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "ValidationError")]
pub struct ValidationError {
    pub detail: String,
    /// JSON Pointer to the invalid location (e.g., "/user/email").
    pub pointer: String,
}

impl Problem {
    pub fn new(status: StatusCode, title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            type_url: "about:blank".to_string(),
            title: title.into(),
            status: status.as_u16(),
            detail: detail.into(),
            instance: String::new(),
            code: String::new(),
            request_id: None,
            errors: None,
        }
    }

    pub fn with_type(mut self, type_url: impl Into<String>) -> Self {
        self.type_url = type_url.into();
        self
    }

    pub fn with_instance(mut self, uri: impl Into<String>) -> Self {
        self.instance = uri.into();
        self
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = code.into();
        self
    }

    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    pub fn with_errors(mut self, errors: Vec<ValidationError>) -> Self {
        self.errors = Some(errors);
        self
    }
}

/// Axum response wrapper that renders `Problem` with correct status & content type.
#[derive(Debug, Clone)]
pub struct ProblemResponse(pub Problem);

impl From<Problem> for ProblemResponse {
    fn from(p: Problem) -> Self {
        Self(p)
    }
}

impl IntoResponse for ProblemResponse {
    fn into_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.0.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut resp = axum::Json(self.0).into_response();
        *resp.status_mut() = status;
        resp.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            HeaderValue::from_static(APPLICATION_PROBLEM_JSON),
        );
        resp
    }
}

// Convenience constructors (optional).
pub fn bad_request(detail: impl Into<String>) -> ProblemResponse {
    Problem::new(StatusCode::BAD_REQUEST, "Bad Request", detail).into()
}

pub fn not_found(detail: impl Into<String>) -> ProblemResponse {
    Problem::new(StatusCode::NOT_FOUND, "Not Found", detail).into()
}

pub fn conflict(detail: impl Into<String>) -> ProblemResponse {
    Problem::new(StatusCode::CONFLICT, "Conflict", detail).into()
}

pub fn internal_error(detail: impl Into<String>) -> ProblemResponse {
    Problem::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal Server Error",
        detail,
    )
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    #[test]
    fn problem_into_response_sets_status_and_content_type() {
        let p = Problem::new(StatusCode::BAD_REQUEST, "Bad Request", "invalid payload");
        let resp = ProblemResponse(p).into_response();
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
        let p = Problem::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Validation Failed",
            "Input validation errors",
        )
        .with_code("VALIDATION_ERROR")
        .with_instance("/users/123")
        .with_request_id("req-456")
        .with_errors(vec![ValidationError {
            detail: "Email is required".to_string(),
            pointer: "/email".to_string(),
        }]);

        assert_eq!(p.status, 422);
        assert_eq!(p.code, "VALIDATION_ERROR");
        assert_eq!(p.instance, "/users/123");
        assert_eq!(p.request_id, Some("req-456".to_string()));
        assert!(p.errors.is_some());
        assert_eq!(p.errors.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn convenience_constructors() {
        let bad_req = bad_request("Invalid input");
        assert_eq!(bad_req.0.status, 400);
        assert_eq!(bad_req.0.title, "Bad Request");

        let not_found_resp = not_found("User not found");
        assert_eq!(not_found_resp.0.status, 404);
        assert_eq!(not_found_resp.0.title, "Not Found");

        let conflict_resp = conflict("Email already exists");
        assert_eq!(conflict_resp.0.status, 409);
        assert_eq!(conflict_resp.0.title, "Conflict");

        let internal_resp = internal_error("Database connection failed");
        assert_eq!(internal_resp.0.status, 500);
        assert_eq!(internal_resp.0.title, "Internal Server Error");
    }
}
