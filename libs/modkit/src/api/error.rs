use crate::api::problem::Problem;
use axum::response::IntoResponse;
use modkit_odata::Error as ODataError;

/// Unified API error type that handles all errors at the API boundary
///
/// This centralizes error handling so that handlers can use `?` operator
/// and automatically get proper RFC 9457 Problem+json responses.
///
/// The `D` type parameter allows different modules to use their own domain error types
/// while still getting unified error handling at the API boundary.
#[derive(thiserror::Error, Debug)]
pub enum ApiError<D> {
    /// `OData` pagination errors (filter, orderby, cursor issues)
    #[error(transparent)]
    OData(ODataError),

    /// Domain business logic errors
    #[error(transparent)]
    Domain(D),
}

// Manual implementations to avoid conflicts with generic From trait
impl<D> ApiError<D> {
    /// Create an `ApiError` from an `OData` error
    #[must_use]
    pub fn from_odata(e: ODataError) -> Self {
        ApiError::OData(e)
    }

    /// Create an `ApiError` from a domain error
    pub fn from_domain(e: D) -> Self {
        ApiError::Domain(e)
    }
}

impl<D> From<ODataError> for ApiError<D> {
    fn from(e: ODataError) -> Self {
        ApiError::OData(e)
    }
}

impl<D> IntoResponse for ApiError<D>
where
    D: Into<Problem>,
{
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::OData(e) => {
                // Use fallback instance "/" if no request context available
                // In real apps, this could be improved to get actual request path
                crate::api::odata::error::odata_error_to_problem(&e, "/", None).into_response()
            }
            ApiError::Domain(e) => {
                // Convert the domain error to a Problem, which then becomes a response
                e.into().into_response()
            }
        }
    }
}

/// Generic Result type for API handlers.
/// Each module typically defines its own alias: `type UsersResult<T> = ApiResult<T, DomainError>;`
pub type ApiResult<T, D> = Result<T, ApiError<D>>;
