//! Mapping from OData errors to Problem (pure data)
//!
//! This provides a baseline conversion from OData errors to RFC 9457 Problem
//! without HTTP framework dependencies. The HTTP layer in `modkit` adds
//! instance paths and trace IDs before the Problem is converted to an HTTP response.

use crate::errors::ErrorCode;
use crate::Error;
use modkit_errors::problem::Problem;

impl From<Error> for Problem {
    fn from(err: Error) -> Self {
        use Error::*;

        match err {
            // Filter parsing errors → 422
            InvalidFilter(msg) => ErrorCode::odata_errors_invalid_filter_v1()
                .to_problem(format!("Invalid $filter: {}", msg)),

            // OrderBy parsing and validation errors → 422
            InvalidOrderByField(field) => ErrorCode::odata_errors_invalid_orderby_v1()
                .to_problem(format!("Unsupported $orderby field: {}", field)),

            // All cursor-related errors → 422
            InvalidCursor
            | CursorInvalidBase64
            | CursorInvalidJson
            | CursorInvalidVersion
            | CursorInvalidKeys
            | CursorInvalidFields
            | CursorInvalidDirection => {
                ErrorCode::odata_errors_invalid_cursor_v1().to_problem(err.to_string())
            }

            // Pagination validation errors → 422
            OrderMismatch => ErrorCode::odata_errors_invalid_orderby_v1()
                .to_problem("Order mismatch between cursor and query"),

            FilterMismatch => ErrorCode::odata_errors_invalid_filter_v1()
                .to_problem("Filter mismatch between cursor and query"),

            InvalidLimit => {
                ErrorCode::odata_errors_invalid_filter_v1().to_problem("Invalid limit parameter")
            }

            OrderWithCursor => ErrorCode::odata_errors_invalid_cursor_v1()
                .to_problem("Cannot specify both $orderby and cursor parameters"),

            // Database errors → 500 (should be caught earlier)
            Db(_msg) => {
                // Use filter error as safe default for unexpected DB errors
                ErrorCode::odata_errors_internal_v1()
                    .to_problem("An internal error occurred while processing the OData query")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_error_converts_to_problem() {
        let err = Error::InvalidFilter("malformed".to_string());
        let problem: Problem = err.into();

        assert_eq!(problem.status, 422);
        assert_eq!(problem.title, "Invalid Filter");
        assert!(problem.detail.contains("malformed"));
        assert!(problem.code.contains("odata"));
        assert!(problem.code.contains("invalid_filter"));
    }

    #[test]
    fn test_orderby_error_converts_to_problem() {
        let err = Error::InvalidOrderByField("unknown".to_string());
        let problem: Problem = err.into();

        assert_eq!(problem.status, 422);
        assert_eq!(problem.title, "Invalid OrderBy");
        assert!(problem.code.contains("odata"));
        assert!(problem.code.contains("invalid_orderby"));
    }

    #[test]
    fn test_cursor_error_converts_to_problem() {
        let err = Error::CursorInvalidBase64;
        let problem: Problem = err.into();

        assert_eq!(problem.status, 422);
        assert_eq!(problem.title, "Invalid Cursor");
        assert!(problem.code.contains("odata"));
        assert!(problem.code.contains("invalid_cursor"));
    }
}
