//! RFC 9457 Problem Details for HTTP APIs (pure data model, no HTTP framework dependencies)

use serde::{Deserialize, Serialize};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

/// Content type for Problem Details as per RFC 9457.
pub const APPLICATION_PROBLEM_JSON: &str = "application/problem+json";

/// RFC 9457 Problem Details for HTTP APIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[cfg_attr(
    feature = "utoipa",
    schema(
        title = "Problem",
        description = "RFC 9457 Problem Details for HTTP APIs"
    )
)]
pub struct Problem {
    /// A URI reference that identifies the problem type.
    /// When dereferenced, it might provide human-readable documentation.
    #[serde(rename = "type")]
    pub type_url: String,
    /// A short, human-readable summary of the problem type.
    pub title: String,
    /// The HTTP status code for this occurrence of the problem. (u16 for a reason - do not create axum dependency)
    pub status: u16,
    /// A human-readable explanation specific to this occurrence of the problem.
    pub detail: String,
    /// A URI reference that identifies the specific occurrence of the problem.
    pub instance: String,
    /// Optional machine-readable error code defined by the application.
    pub code: String,
    /// Optional trace id useful for tracing.
    #[serde(rename = "traceId")]
    pub trace_id: Option<String>,
    /// Optional validation errors for 4xx problems.
    pub errors: Option<Vec<ValidationViolation>>,
}

/// Individual validation violation for a specific field or property.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[cfg_attr(feature = "utoipa", schema(title = "ValidationViolation"))]
pub struct ValidationViolation {
    /// JSON Pointer or field path, e.g. "/email" or "user.email"
    pub pointer: String,
    /// Human-readable message describing the validation error
    pub message: String,
    /// Optional machine-readable error code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Collection of validation errors for 422 responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[cfg_attr(feature = "utoipa", schema(title = "ValidationError"))]
pub struct ValidationError {
    /// List of individual validation violations
    pub errors: Vec<ValidationViolation>,
}

/// Wrapper for ValidationError that can be used as a standalone response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[cfg_attr(feature = "utoipa", schema(title = "ValidationErrorResponse"))]
pub struct ValidationErrorResponse {
    /// The validation errors
    #[serde(flatten)]
    pub validation: ValidationError,
}

impl Problem {
    /// Create a new Problem with the given status, title, and detail.
    ///
    /// Note: This function takes `status` as `u16` to avoid depending on HTTP crates.
    /// Use `http::StatusCode::as_u16()` when calling from HTTP contexts.
    pub fn new(status: u16, title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            type_url: "about:blank".to_string(),
            title: title.into(),
            status,
            detail: detail.into(),
            instance: String::new(),
            code: String::new(),
            trace_id: None,
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

    pub fn with_trace_id(mut self, id: impl Into<String>) -> Self {
        self.trace_id = Some(id.into());
        self
    }

    pub fn with_errors(mut self, errors: Vec<ValidationViolation>) -> Self {
        self.errors = Some(errors);
        self
    }
}

/// Axum integration: make Problem directly usable as a response
#[cfg(feature = "axum")]
impl axum::response::IntoResponse for Problem {
    fn into_response(self) -> axum::response::Response {
        use axum::http::{HeaderValue, StatusCode};

        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut resp = axum::Json(self).into_response();
        *resp.status_mut() = status;
        resp.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            HeaderValue::from_static(APPLICATION_PROBLEM_JSON),
        );
        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
