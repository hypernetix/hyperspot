//! Centralized `OData` error mapping
//!
//! This module adds HTTP-specific context (instance path, trace ID) to OData errors.
//! The core Error → Problem mapping is owned by modkit-odata.

use crate::api::problem::Problem;
use modkit_odata::Error as ODataError;

/// Extract trace ID from current tracing span
#[inline]
fn current_trace_id() -> Option<String> {
    tracing::Span::current()
        .id()
        .map(|id| id.into_u64().to_string())
}

/// Returns a fully contextualized Problem for `OData` errors.
///
/// This function maps all `modkit_odata::Error` variants to appropriate system
/// error codes from the framework catalog. The `instance` parameter should
/// be the request path.
///
/// # Arguments
/// * `err` - The `OData` error to convert
/// * `instance` - The request path (e.g., "/api/user-management/v1/users")
/// * `trace_id` - Optional trace ID (uses current span if None)
pub fn odata_error_to_problem(
    err: &ODataError,
    instance: &str,
    trace_id: Option<String>,
) -> Problem {
    use modkit_odata::Error as OE;

    // Add logging for errors that need it before conversion
    match err {
        OE::Db(msg) => {
            tracing::error!(error = %msg, "Unexpected database error in OData layer");
        }
        OE::ParsingUnavailable(msg) => {
            tracing::error!(error = %msg, "OData parsing unavailable");
        }
        _ => {}
    }

    // Delegate to modkit-odata's base mapping (single source of truth)
    let mut problem: Problem = err.clone().into();

    // Add HTTP-specific context
    problem = problem.with_instance(instance);
    
    let trace_id = trace_id.or_else(current_trace_id);
    if let Some(tid) = trace_id {
        problem = problem.with_trace_id(tid);
    }

    problem
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_filter_error_mapping() {
        use http::StatusCode;

        let error = ODataError::InvalidFilter("malformed expression".to_owned());
        let problem = odata_error_to_problem(&error, "/user-management/v1/users", None);

        assert_eq!(problem.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(problem.code.contains("invalid_filter"));
        assert_eq!(problem.instance, "/user-management/v1/users");
    }

    #[test]
    fn test_orderby_error_mapping() {
        use http::StatusCode;

        let error = ODataError::InvalidOrderByField("unknown_field".to_owned());
        let problem = odata_error_to_problem(&error, "/user-management/v1/users", None);

        assert_eq!(problem.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(problem.code.contains("invalid_orderby"));
    }

    #[test]
    fn test_cursor_error_mapping() {
        use http::StatusCode;

        let error = ODataError::CursorInvalidBase64;
        let problem = odata_error_to_problem(
            &error,
            "/user-management/v1/users",
            Some("trace123".to_owned()),
        );

        assert_eq!(problem.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(problem.code.contains("invalid_cursor"));
        assert_eq!(problem.trace_id, Some("trace123".to_owned()));
    }

    #[test]
    fn test_gts_code_format() {
        let error = ODataError::InvalidFilter("test".to_owned());
        let problem = odata_error_to_problem(&error, "/user-management/v1/test", None);

        // Verify the code follows GTS format
        assert!(problem.code.starts_with("gts.hx.core.errors.err.v1~"));
        assert!(problem.code.contains("odata"));
    }
}
