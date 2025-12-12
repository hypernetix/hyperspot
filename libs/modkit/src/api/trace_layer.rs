//! Trace propagation utilities for Problem responses
//!
//! This module provides helper traits and functions to automatically enrich
//! `Problem` with trace context:
//! - `trace_id`: extracted from the current tracing span
//! - `instance`: extracted from the request URI
//!
//! This eliminates per-callsite boilerplate and ensures consistent error reporting.

use crate::api::problem::Problem;

/// Extract trace_id from the current tracing span
fn extract_trace_id() -> Option<String> {
    // Try to extract from the current span's trace_id field
    // This requires coordination with the tracing subscriber
    tracing::Span::current().id().map(|id| format!("{:?}", id))
}

/// Helper trait for enriching Problem with trace context
pub trait WithTraceContext {
    /// Enrich this Problem with trace_id and instance from the current request context
    fn with_trace_context(self, instance: impl Into<String>) -> Self;
}

impl WithTraceContext for Problem {
    fn with_trace_context(mut self, instance: impl Into<String>) -> Self {
        self = self.with_instance(instance);
        if let Some(tid) = extract_trace_id() {
            self = self.with_trace_id(tid);
        }
        self
    }
}

/// Middleware-friendly: enrich errors from Axum extractors
///
/// Use this in handlers to automatically add trace context:
///
/// ```ignore
/// async fn handler(uri: Uri) -> Result<Json<Data>, Problem> {
///     let data = fetch_data()
///         .await
///         .map_err(Problem::from)
///         .map_err(|p| p.with_request_context(&uri))?;
///     Ok(Json(data))
/// }
/// ```
pub trait WithRequestContext {
    /// Add trace_id and instance from the current request
    fn with_request_context(self, uri: &axum::http::Uri) -> Self;
}

impl WithRequestContext for Problem {
    fn with_request_context(self, uri: &axum::http::Uri) -> Self {
        self.with_trace_context(uri.path())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_with_trace_context() {
        use http::StatusCode;

        let problem = Problem::new(StatusCode::NOT_FOUND, "Not Found", "Resource not found")
            .with_trace_context("/api/users/123");

        assert_eq!(problem.instance, "/api/users/123");
        // trace_id may or may not be set depending on tracing context
    }

    #[test]
    fn test_with_request_context() {
        use axum::http::Uri;
        use http::StatusCode;

        let uri: Uri = "/api/users/123".parse().unwrap();
        let problem = Problem::new(StatusCode::NOT_FOUND, "Not Found", "Resource not found")
            .with_request_context(&uri);

        assert_eq!(problem.instance, "/api/users/123");
    }
}
