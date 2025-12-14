use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// Type alias for error response tuple (without details).
/// We keep `details` separately as `Option<String>` to avoid lifetime issues.
type ErrorResponseTuple<'a> = (StatusCode, &'static str, &'a str);

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("rate limited")]
    TooManyRequests,
    #[error("internal error")]
    Internal(#[source] anyhow::Error),
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    code: &'a str,
    message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<&'a str>,
    #[cfg(feature = "debug-errors")]
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<&'a str>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        use AppError::{
            BadRequest, Conflict, Forbidden, Internal, NotFound, TooManyRequests, Unauthorized,
        };

        // Extract request_id from current span context if available
        // Note: In real handlers we will get request_id from extensions
        let request_id = "unknown";

        // Keep details as owned String to avoid dangling references
        #[cfg(feature = "debug-errors")]
        let mut dbg_details: Option<String> = None;

        // Map AppError to tuple (status, code, safe_msg)
        let (status, code, safe_msg): ErrorResponseTuple = match &self {
            BadRequest(m) => (StatusCode::BAD_REQUEST, "bad_request", m.as_str()),
            Unauthorized(m) => (StatusCode::UNAUTHORIZED, "unauthorized", m.as_str()),
            Forbidden(m) => (StatusCode::FORBIDDEN, "forbidden", m.as_str()),
            NotFound(m) => (StatusCode::NOT_FOUND, "not_found", m.as_str()),
            Conflict(m) => (StatusCode::CONFLICT, "conflict", m.as_str()),
            TooManyRequests => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "rate limited",
            ),
            #[cfg_attr(not(feature = "debug-errors"), allow(unused_variables))]
            Internal(err) => {
                #[cfg(feature = "debug-errors")]
                {
                    // Save error details as String, later exposed as Option<&str>
                    dbg_details = Some(err.to_string());
                }
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "internal error",
                )
            }
        };

        // Log error with appropriate severity
        match &self {
            Internal(err) => tracing::error!(
                request_id = %request_id,
                error = %err,
                status = status.as_u16(),
                "request failed"
            ),
            other => tracing::warn!(
                request_id = %request_id,
                error = %other,
                status = status.as_u16(),
                "request failed"
            ),
        }

        // Build JSON response body
        let body = ErrorBody {
            code,
            message: safe_msg,
            request_id: Some(request_id),
            #[cfg(feature = "debug-errors")]
            details: dbg_details.as_deref(),
        };

        (status, Json(body)).into_response()
    }
}
