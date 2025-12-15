//! Ergonomic result types for API handlers
//!
//! This module provides type aliases and conversions to make error handling
//! in HTTP handlers more concise and uniform.

use crate::api::problem::Problem;

/// Standard result type for API operations
///
/// Use this throughout your handlers for consistent error handling:
///
/// ```ignore
/// async fn handler() -> ApiResult<Json<User>> {
///     let user = fetch_user().await?;  // auto-converts errors to Problem
///     Ok(Json(user))
/// }
/// ```
///
/// The `?` operator automatically converts any error implementing
/// `Into<Problem>` (including `modkit_odata::Error`) into a Problem.
/// Problem implements `IntoResponse`, so Axum will automatically convert it
/// to an HTTP response when returned from a handler.
pub type ApiResult<T = ()> = Result<T, Problem>;

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_api_result_ok() {
        let result: ApiResult<i32> = Ok(42);
        assert!(result.is_ok());
    }

    #[test]
    fn test_api_result_err() {
        use http::StatusCode;

        let result: ApiResult<i32> = Err(Problem::new(
            StatusCode::BAD_REQUEST,
            "Bad Request",
            "Invalid input",
        ));
        assert!(result.is_err());
    }
}
