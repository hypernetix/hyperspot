use thiserror::Error;

/// Errors that can occur during JWT claims validation and processing
#[derive(Debug, Error)]
pub enum ClaimsError {
    #[error("Invalid signature or key")]
    InvalidSignature,

    #[error("Invalid issuer: expected one of {expected:?}, got {actual}")]
    InvalidIssuer {
        expected: Vec<String>,
        actual: String,
    },

    #[error("Invalid audience: expected one of {expected:?}, got {actual:?}")]
    InvalidAudience {
        expected: Vec<String>,
        actual: Vec<String>,
    },

    #[error("Token expired")]
    Expired,

    #[error("Token not yet valid (nbf check failed)")]
    NotYetValid,

    #[error("Malformed claims: {0}")]
    Malformed(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Missing required claim: {0}")]
    MissingClaim(String),

    #[error("Invalid claim format: {field} - {reason}")]
    InvalidClaimFormat { field: String, reason: String },

    #[error("No matching plugin found for token")]
    NoMatchingPlugin,

    #[error("No key provider could validate this token")]
    NoValidatingKey,

    #[error("No matching key provider")]
    NoMatchingProvider,

    #[error("Unknown key ID after refresh")]
    UnknownKidAfterRefresh,

    #[error("Introspection denied")]
    IntrospectionDenied,

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("JWT decode failed: {0}")]
    DecodeFailed(String),

    #[error("JWKS fetch failed: {0}")]
    JwksFetchFailed(String),

    #[error("Unknown key ID: {0}")]
    UnknownKeyId(String),
}

// Conversion from ClaimsError to AuthError for backward compatibility
impl From<ClaimsError> for crate::errors::AuthError {
    fn from(err: ClaimsError) -> Self {
        match err {
            ClaimsError::Expired => crate::errors::AuthError::TokenExpired,
            ClaimsError::InvalidSignature => {
                crate::errors::AuthError::InvalidToken("Invalid signature".into())
            }
            ClaimsError::InvalidIssuer { expected, actual } => {
                crate::errors::AuthError::IssuerMismatch {
                    expected: expected.join(", "),
                    actual,
                }
            }
            ClaimsError::InvalidAudience { expected, actual } => {
                crate::errors::AuthError::AudienceMismatch { expected, actual }
            }
            ClaimsError::JwksFetchFailed(msg) => crate::errors::AuthError::JwksFetchFailed(msg),
            other => crate::errors::AuthError::ValidationFailed(other.to_string()),
        }
    }
}

#[cfg(feature = "axum-ext")]
impl axum::response::IntoResponse for ClaimsError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        use axum::response::Json;
        use serde_json::json;

        let (status, message) = match self {
            // 401 Unauthorized - authentication failures
            ClaimsError::Expired
            | ClaimsError::NotYetValid
            | ClaimsError::InvalidSignature
            | ClaimsError::InvalidIssuer { .. }
            | ClaimsError::InvalidAudience { .. }
            | ClaimsError::Malformed(_)
            | ClaimsError::MissingClaim(_)
            | ClaimsError::InvalidClaimFormat { .. }
            | ClaimsError::NoMatchingPlugin
            | ClaimsError::NoValidatingKey
            | ClaimsError::NoMatchingProvider
            | ClaimsError::UnknownKidAfterRefresh
            | ClaimsError::DecodeFailed(_)
            | ClaimsError::UnknownKeyId(_) => (StatusCode::UNAUTHORIZED, self.to_string()),

            // 403 Forbidden - introspection denied
            ClaimsError::IntrospectionDenied => (StatusCode::FORBIDDEN, self.to_string()),

            // 500 Internal Server Error - system/config errors
            ClaimsError::ConfigError(_)
            | ClaimsError::Provider(_)
            | ClaimsError::JwksFetchFailed(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}
