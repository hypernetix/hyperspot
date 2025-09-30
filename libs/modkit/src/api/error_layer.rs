//! Centralized error mapping for Axum
//!
//! This module provides utilities for automatically converting all framework
//! and module errors into consistent RFC 9457 Problem+JSON responses, eliminating
//! per-route boilerplate.

use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::any::Any;

use crate::api::problem::{Problem, ProblemResponse};
use crate::context::ConfigError;
use odata_core::Error as ODataError;

/// Middleware function that provides centralized error mapping
///
/// This middleware can be applied to routes to automatically extract request context
/// and provide it to error handlers. The actual error conversion happens in the
/// `IntoProblemResponse` trait implementations and `map_error_to_problem` function.
pub async fn error_mapping_middleware(request: Request, next: Next) -> Response {
    let _uri = request.uri().clone();
    let _headers = request.headers().clone();

    let response = next.run(request).await;

    // If the response is already successful or is already a Problem response, pass it through
    if response.status().is_success() || is_problem_response(&response) {
        return response;
    }

    // For error responses, the actual error conversion should happen in the handlers
    // using the IntoProblemResponse trait or map_error_to_problem function
    // This middleware provides the infrastructure for extracting request context
    response
}

/// Check if a response is already a Problem+JSON response
fn is_problem_response(response: &Response) -> bool {
    response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.contains("application/problem+json"))
        .unwrap_or(false)
}

/// Extract trace ID from headers or generate one
pub fn extract_trace_id(headers: &HeaderMap) -> Option<String> {
    // Try to get trace ID from various common headers
    headers
        .get("x-trace-id")
        .or_else(|| headers.get("x-request-id"))
        .or_else(|| headers.get("traceparent"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| {
            // Try to get from current tracing span
            tracing::Span::current()
                .id()
                .map(|id| id.into_u64().to_string())
        })
}

/// Centralized error mapping function
///
/// This function provides a single place to convert all framework and module errors
/// into consistent Problem responses with proper trace IDs and instance paths.
pub fn map_error_to_problem(
    error: &dyn Any,
    instance: &str,
    trace_id: Option<String>,
) -> ProblemResponse {
    // Try to downcast to known error types
    if let Some(odata_err) = error.downcast_ref::<ODataError>() {
        let mut problem = crate::api::odata::error::odata_error_to_problem(odata_err, instance);
        if let Some(tid) = trace_id {
            problem.0 = problem.0.with_trace_id(tid);
        }
        return problem;
    }

    if let Some(config_err) = error.downcast_ref::<ConfigError>() {
        let mut problem = match config_err {
            ConfigError::ModuleNotFound { module } => Problem::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Configuration Error",
                format!("Module '{}' configuration not found", module),
            )
            .with_code("CONFIG_MODULE_NOT_FOUND")
            .with_type("https://errors.example.com/CONFIG_MODULE_NOT_FOUND"),

            ConfigError::InvalidModuleStructure { module } => Problem::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Configuration Error",
                format!("Module '{}' has invalid configuration structure", module),
            )
            .with_code("CONFIG_INVALID_STRUCTURE")
            .with_type("https://errors.example.com/CONFIG_INVALID_STRUCTURE"),

            ConfigError::MissingConfigSection { module } => Problem::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Configuration Error",
                format!("Module '{}' is missing required config section", module),
            )
            .with_code("CONFIG_MISSING_SECTION")
            .with_type("https://errors.example.com/CONFIG_MISSING_SECTION"),

            ConfigError::InvalidConfig { module, .. } => Problem::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Configuration Error",
                format!("Module '{}' has invalid configuration", module),
            )
            .with_code("CONFIG_INVALID")
            .with_type("https://errors.example.com/CONFIG_INVALID"),
        };

        problem = problem.with_instance(instance);
        if let Some(tid) = trace_id {
            problem = problem.with_trace_id(tid);
        }
        return problem.into();
    }

    // Handle anyhow::Error
    if let Some(anyhow_err) = error.downcast_ref::<anyhow::Error>() {
        let mut problem = Problem::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error",
            "An internal error occurred",
        )
        .with_code("INTERNAL_ERROR")
        .with_type("https://errors.example.com/INTERNAL_ERROR");

        problem = problem.with_instance(instance);
        if let Some(tid) = trace_id {
            problem = problem.with_trace_id(tid);
        }

        // Log the full error for debugging
        tracing::error!(error = %anyhow_err, "Internal server error");
        return problem.into();
    }

    // Fallback for unknown error types
    let mut problem = Problem::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Unknown Error",
        "An unknown error occurred",
    )
    .with_code("UNKNOWN_ERROR")
    .with_type("https://errors.example.com/UNKNOWN_ERROR");

    problem = problem.with_instance(instance);
    if let Some(tid) = trace_id {
        problem = problem.with_trace_id(tid);
    }

    tracing::error!("Unknown error type in error mapping layer");
    problem.into()
}

/// Helper trait for converting errors to Problem responses with context
pub trait IntoProblemResponse {
    fn into_problem_response(self, instance: &str, trace_id: Option<String>) -> ProblemResponse;
}

impl IntoProblemResponse for ODataError {
    fn into_problem_response(self, instance: &str, trace_id: Option<String>) -> ProblemResponse {
        let mut problem = crate::api::odata::error::odata_error_to_problem(&self, instance);
        if let Some(tid) = trace_id {
            problem.0 = problem.0.with_trace_id(tid);
        }
        problem
    }
}

impl IntoProblemResponse for ConfigError {
    fn into_problem_response(self, instance: &str, trace_id: Option<String>) -> ProblemResponse {
        map_error_to_problem(&self as &dyn Any, instance, trace_id)
    }
}

impl IntoProblemResponse for anyhow::Error {
    fn into_problem_response(self, instance: &str, trace_id: Option<String>) -> ProblemResponse {
        map_error_to_problem(&self as &dyn Any, instance, trace_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_odata_error_mapping() {
        let error = ODataError::InvalidFilter("malformed".to_string());
        let problem = error.into_problem_response("/test", Some("trace123".to_string()));

        assert_eq!(problem.0.status, 400);
        assert_eq!(problem.0.code, "ODATA_FILTER_INVALID");
        assert_eq!(problem.0.instance, "/test");
        assert_eq!(problem.0.trace_id, Some("trace123".to_string()));
    }

    #[test]
    fn test_config_error_mapping() {
        let error = ConfigError::ModuleNotFound {
            module: "test_module".to_string(),
        };
        let problem = error.into_problem_response("/api/test", None);

        assert_eq!(problem.0.status, 500);
        assert_eq!(problem.0.code, "CONFIG_MODULE_NOT_FOUND");
        assert_eq!(problem.0.instance, "/api/test");
        assert!(problem.0.detail.contains("test_module"));
    }

    #[test]
    fn test_anyhow_error_mapping() {
        let error = anyhow::anyhow!("Something went wrong");
        let problem = error.into_problem_response("/api/test", Some("trace456".to_string()));

        assert_eq!(problem.0.status, 500);
        assert_eq!(problem.0.code, "INTERNAL_ERROR");
        assert_eq!(problem.0.instance, "/api/test");
        assert_eq!(problem.0.trace_id, Some("trace456".to_string()));
    }

    #[test]
    fn test_extract_trace_id_from_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-trace-id", "test-trace-123".parse().unwrap());

        let trace_id = extract_trace_id(&headers);
        assert_eq!(trace_id, Some("test-trace-123".to_string()));
    }
}
