use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Authentication required: missing or invalid token")]
    Unauthenticated,

    #[error("Forbidden: insufficient permissions")]
    Forbidden,

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Token validation failed: {0}")]
    ValidationFailed(String),

    #[error("JWKS fetch failed: {0}")]
    JwksFetchFailed(String),

    #[error("Issuer mismatch: expected {expected}, got {actual}")]
    IssuerMismatch { expected: String, actual: String },

    #[error("Audience mismatch: expected {expected:?}, got {actual:?}")]
    AudienceMismatch {
        expected: Vec<String>,
        actual: Vec<String>,
    },

    #[error("Token expired")]
    TokenExpired,

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(feature = "axum-ext")]
impl axum::response::IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        use axum::response::Json;
        use serde_json::json;

        let (status, message) = match self {
            AuthError::Unauthenticated | AuthError::InvalidToken(_) | AuthError::TokenExpired => {
                (StatusCode::UNAUTHORIZED, self.to_string())
            }
            AuthError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}
