//! Centralized `OData` error mapping using `OData` catalog
//!
//! This module provides a single source of truth for mapping `modkit_odata::Error`
//! to RFC 9457 Problem+JSON responses using the `OData` error catalog.

use crate::api::problem::Problem;
use modkit_odata::errors::ErrorCode;
use modkit_odata::Error as ODataError;

/// Extract trace ID from current tracing span
#[inline]
fn current_trace_id() -> Option<String> {
    tracing::Span::current()
        .id()
        .map(|id| id.into_u64().to_string())
}

/// Helper to convert `ErrorCode` to Problem with context
#[inline]
fn to_problem(
    code: ErrorCode,
    detail: impl Into<String>,
    instance: &str,
    trace_id: Option<String>,
) -> Problem {
    let mut problem = code.as_problem(detail);
    problem = problem.with_instance(instance);
    if let Some(tid) = trace_id {
        problem = problem.with_trace_id(tid);
    }
    problem
}

/// Returns a fully contextualized Problem for `OData` errors.
///
/// This function maps all `modkit_odata::Error` variants to appropriate system
/// error codes from the framework catalog. The `instance` parameter should
/// be the request path.
///
/// # Arguments
/// * `err` - The `OData` error to convert
/// * `instance` - The request path (e.g., "/api/users")
/// * `trace_id` - Optional trace ID (uses current span if None)
pub fn odata_error_to_problem(
    err: &ODataError,
    instance: &str,
    trace_id: Option<String>,
) -> Problem {
    use modkit_odata::Error as OE;

    let trace_id = trace_id.or_else(current_trace_id);

    match err {
        // Filter parsing errors
        OE::InvalidFilter(msg) => to_problem(
            ErrorCode::odata_errors_invalid_filter_v1(),
            format!("Invalid $filter: {msg}"),
            instance,
            trace_id,
        ),

        // OrderBy parsing and validation errors
        OE::InvalidOrderByField(field) => to_problem(
            ErrorCode::odata_errors_invalid_orderby_v1(),
            format!("Unsupported $orderby field: {field}"),
            instance,
            trace_id,
        ),

        // All cursor-related errors map to invalid_cursor
        OE::InvalidCursor
        | OE::CursorInvalidBase64
        | OE::CursorInvalidJson
        | OE::CursorInvalidVersion
        | OE::CursorInvalidKeys
        | OE::CursorInvalidFields
        | OE::CursorInvalidDirection => to_problem(
            ErrorCode::odata_errors_invalid_cursor_v1(),
            err.to_string(), // Use the specific error message
            instance,
            trace_id,
        ),

        // Pagination validation errors
        OE::OrderMismatch => to_problem(
            ErrorCode::odata_errors_invalid_orderby_v1(),
            "Order mismatch between cursor and query",
            instance,
            trace_id,
        ),
        OE::FilterMismatch => to_problem(
            ErrorCode::odata_errors_invalid_filter_v1(),
            "Filter mismatch between cursor and query",
            instance,
            trace_id,
        ),
        OE::InvalidLimit => to_problem(
            ErrorCode::odata_errors_invalid_filter_v1(),
            "Invalid limit parameter",
            instance,
            trace_id,
        ),
        OE::OrderWithCursor => to_problem(
            ErrorCode::odata_errors_invalid_cursor_v1(),
            "Cannot specify both $orderby and cursor parameters",
            instance,
            trace_id,
        ),

        // Database errors should not happen at OData layer in production,
        // but if they do, map to filter error (422) as a safe default
        OE::Db(msg) => {
            tracing::error!(error = %msg, "Unexpected database error in OData layer");
            to_problem(
                ErrorCode::odata_errors_invalid_filter_v1(),
                "An internal error occurred while processing the query",
                instance,
                trace_id,
            )
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_filter_error_mapping() {
        use http::StatusCode;

        let error = ODataError::InvalidFilter("malformed expression".to_owned());
        let problem = odata_error_to_problem(&error, "/api/users", None);

        assert_eq!(problem.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(problem.code.contains("invalid_filter"));
        assert_eq!(problem.instance, "/api/users");
    }

    #[test]
    fn test_orderby_error_mapping() {
        use http::StatusCode;

        let error = ODataError::InvalidOrderByField("unknown_field".to_owned());
        let problem = odata_error_to_problem(&error, "/api/users", None);

        assert_eq!(problem.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(problem.code.contains("invalid_orderby"));
    }

    #[test]
    fn test_cursor_error_mapping() {
        use http::StatusCode;

        let error = ODataError::CursorInvalidBase64;
        let problem = odata_error_to_problem(&error, "/api/users", Some("trace123".to_owned()));

        assert_eq!(problem.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(problem.code.contains("invalid_cursor"));
        assert_eq!(problem.trace_id, Some("trace123".to_owned()));
    }

    #[test]
    fn test_gts_code_format() {
        let error = ODataError::InvalidFilter("test".to_owned());
        let problem = odata_error_to_problem(&error, "/api/test", None);

        // Verify the code follows GTS format
        assert!(problem.code.starts_with("gts.hx.core.errors.err.v1~"));
        assert!(problem.code.contains("odata"));
    }
}
