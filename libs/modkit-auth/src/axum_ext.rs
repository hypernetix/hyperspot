//! Axum extractors and middleware for auth

use crate::{claims::Claims, dispatcher::AuthDispatcher, errors::AuthError};
use axum::{
    extract::{FromRequestParts, Request, State},
    http::{request::Parts, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use modkit_security::SecurityCtx;
use std::sync::Arc;

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

/// Extractor for Claims - validates that auth middleware has run
#[derive(Debug, Clone)]
pub struct AuthClaims(pub Claims);

impl<S> FromRequestParts<S> for AuthClaims
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
                .get::<Claims>()
                .cloned()
                .map(AuthClaims)
                .ok_or(AuthError::Internal(
                    "Claims not found - auth middleware not configured".to_string(),
                ))
        }
    }
}

/// Axum middleware that validates JWT tokens using AuthDispatcher
///
/// This middleware:
/// 1. Skips authentication for CORS preflight requests (OPTIONS with Origin header)
/// 2. Extracts the Bearer token from the Authorization header
/// 3. Validates it using the AuthDispatcher
/// 4. Inserts Claims and SecurityCtx into request extensions
/// 5. Returns 401 if validation fails
///
/// # Usage
///
/// ```rust,ignore
/// use axum::{Router, routing::get, middleware};
/// use modkit_auth::{build_auth_dispatcher, AuthConfig, axum_ext::auth_required};
/// use std::sync::Arc;
///
/// let config = AuthConfig::default();
/// let dispatcher = Arc::new(build_auth_dispatcher(&config)?);
///
/// let app = Router::new()
///     .route("/protected", get(|| async { "OK" }))
///     .layer(middleware::from_fn_with_state(dispatcher, auth_required));
/// ```
pub async fn auth_required(
    State(dispatcher): State<Arc<AuthDispatcher>>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Skip auth for CORS preflight requests
    if is_preflight_request(&request) {
        return Ok(next.run(request).await);
    }

    // Extract token from Authorization header
    let token = extract_bearer_token(request.headers()).ok_or_else(|| {
        tracing::debug!("Missing or invalid Authorization header");
        (
            StatusCode::UNAUTHORIZED,
            "Missing or invalid Authorization header",
        )
            .into_response()
    })?;

    // Validate token using dispatcher
    let claims = dispatcher.validate_jwt(token).await.map_err(|e| {
        tracing::warn!(error_type = e.to_string(), "Token validation failed");
        (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e)).into_response()
    })?;

    // Convert to SecurityCtx
    let security_ctx = SecurityCtx::for_tenants(claims.tenants.clone(), claims.sub);

    // Insert into request extensions
    request.extensions_mut().insert(claims);
    request.extensions_mut().insert(security_ctx);

    Ok(next.run(request).await)
}

/// Axum middleware that validates JWT tokens but doesn't fail on missing/invalid tokens
///
/// This is useful for endpoints that have optional authentication.
/// If a valid token is present, Claims and SecurityCtx are added to extensions.
/// If not, an anonymous SecurityCtx is inserted.
///
/// CORS preflight requests are always allowed through.
pub async fn auth_optional(
    State(dispatcher): State<Arc<AuthDispatcher>>,
    mut request: Request,
    next: Next,
) -> Response {
    // Skip auth for CORS preflight requests
    if is_preflight_request(&request) {
        return next.run(request).await;
    }

    // Try to extract and validate token
    if let Some(token) = extract_bearer_token(request.headers()) {
        if let Ok(claims) = dispatcher.validate_jwt(token).await {
            let security_ctx = SecurityCtx::for_tenants(claims.tenants.clone(), claims.sub);

            request.extensions_mut().insert(claims);
            request.extensions_mut().insert(security_ctx);

            return next.run(request).await;
        }
    }

    // No valid token - insert anonymous context
    request
        .extensions_mut()
        .insert(SecurityCtx::deny_all(uuid::Uuid::nil()));

    next.run(request).await
}

/// Extract Bearer token from Authorization header
fn extract_bearer_token(headers: &axum::http::HeaderMap) -> Option<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ").map(|t| t.trim()))
}

/// Check if this is a CORS preflight request
///
/// Preflight requests are OPTIONS requests with:
/// - Origin header present
/// - Access-Control-Request-Method header present
fn is_preflight_request(request: &Request) -> bool {
    request.method() == Method::OPTIONS
        && request.headers().contains_key(axum::http::header::ORIGIN)
        && request
            .headers()
            .contains_key(axum::http::header::ACCESS_CONTROL_REQUEST_METHOD)
}
