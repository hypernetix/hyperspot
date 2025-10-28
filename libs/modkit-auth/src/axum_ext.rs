//! Axum extractors for auth-related types

use crate::errors::AuthError;
use axum::{extract::FromRequestParts, http::request::Parts};
use modkit_security::SecurityCtx;

/// Extractor for SecurityCtx - validates that auth middleware has run
#[derive(Debug, Clone)]
pub struct Authz(pub SecurityCtx);

impl<S> FromRequestParts<S> for Authz
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    #[allow(clippy::manual_async_fn)]
    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl core::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            parts
                .extensions
                .get::<SecurityCtx>()
                .cloned()
                .map(Authz)
                .ok_or(AuthError::Internal(
                    "SecurityCtx not found - auth middleware not configured".to_string(),
                ))
        }
    }
}

/// Re-export Claims for convenience
pub use crate::claims::Claims;
