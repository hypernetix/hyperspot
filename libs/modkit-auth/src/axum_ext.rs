//! Axum extractors and middleware for auth

use crate::{
    claims::Claims,
    errors::AuthError,
    traits::{PrimaryAuthorizer, ScopeBuilder, TokenValidator},
    types::{AuthRequirement, RoutePolicy},
};
use axum::{
    extract::{FromRequestParts, Request, State},
    http::{request::Parts, HeaderMap, Method},
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

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<SecurityCtx>()
            .cloned() // TODO: drop this clone
            .map(Authz)
            .ok_or(AuthError::Internal(
                "SecurityCtx not found - auth middleware not configured".to_string(),
            ))
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

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned() // TODO: drop this clone
            .map(AuthClaims)
            .ok_or(AuthError::Internal(
                "Claims not found - auth middleware not configured".to_string(),
            ))
    }
}

#[derive(Clone)]
pub struct AuthPolicyState {
    validator: Arc<dyn TokenValidator>,
    scope_builder: Arc<dyn ScopeBuilder>,
    authorizer: Arc<dyn PrimaryAuthorizer>,
    policy: Arc<dyn RoutePolicy>,
}

impl AuthPolicyState {
    pub fn new(
        validator: Arc<dyn TokenValidator>,
        scope_builder: Arc<dyn ScopeBuilder>,
        authorizer: Arc<dyn PrimaryAuthorizer>,
        policy: Arc<dyn RoutePolicy>,
    ) -> Self {
        Self {
            validator,
            scope_builder,
            authorizer,
            policy,
        }
    }
}

/// Unified auth middleware with route policy support
///
/// This middleware:
/// 1. Skips authentication for CORS preflight requests
/// 2. Resolves the route's authentication requirement using RoutePolicy
/// 3. For public routes (AuthRequirement::None): inserts anonymous SecurityCtx
/// 4. For required routes: validates JWT, enforces RBAC if needed, inserts SecurityCtx
/// 5. For optional routes: validates JWT if present, otherwise inserts anonymous SecurityCtx
///
/// Returns Response directly (Axum 0.8 style) with errors converted via IntoResponse.
pub async fn auth_with_policy(
    State(AuthPolicyState {
        validator,
        scope_builder,
        authorizer,
        policy,
    }): State<AuthPolicyState>,
    mut request: Request,
    next: Next,
) -> Response {
    // 1. Preflight: skip auth
    if is_preflight_request(request.method(), request.headers()) {
        return next.run(request).await;
    }

    // 2. Resolve route policy
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let auth_requirement = policy.resolve(&method, &path).await;

    match auth_requirement {
        AuthRequirement::None => {
            // Public: anonymous SecurityCtx
            let sec = SecurityCtx::anonymous();
            request.extensions_mut().insert(sec);
            next.run(request).await
        }
        AuthRequirement::Required(sec_requirement) => {
            // Auth required: token must be present & valid.
            let token = match extract_bearer_token(request.headers()) {
                Some(token) => token,
                None => {
                    return AuthError::Unauthenticated.into_response();
                }
            };

            let claims = match validator.validate_and_parse(token).await {
                Ok(claims) => claims,
                Err(err) => {
                    return err.into_response();
                }
            };

            // Optional RBAC requirement
            if let Some(sec_req) = sec_requirement {
                if let Err(err) = authorizer.check(&claims, &sec_req).await {
                    return err.into_response();
                }
            }

            // Build SecurityCtx from validated claims
            let scope = scope_builder.tenants_to_scope(&claims);
            let sec = SecurityCtx::new(scope, modkit_security::Subject::new(claims.sub));

            request.extensions_mut().insert(claims);
            request.extensions_mut().insert(sec);
            next.run(request).await
        }
        AuthRequirement::Optional => {
            // If token present: validate, else anonymous.
            if let Some(token) = extract_bearer_token(request.headers()) {
                match validator.validate_and_parse(token).await {
                    Ok(claims) => {
                        let scope = scope_builder.tenants_to_scope(&claims);
                        let sec =
                            SecurityCtx::new(scope, modkit_security::Subject::new(claims.sub));
                        request.extensions_mut().insert(claims);
                        request.extensions_mut().insert(sec);
                    }
                    Err(err) => {
                        tracing::debug!("Optional auth: invalid token: {err}");
                        request.extensions_mut().insert(SecurityCtx::anonymous());
                    }
                }
            } else {
                request.extensions_mut().insert(SecurityCtx::anonymous());
            }
            next.run(request).await
        }
    }
}

/// Extract Bearer token from Authorization header
fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
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
fn is_preflight_request(method: &Method, headers: &HeaderMap) -> bool {
    method == Method::OPTIONS
        && headers.contains_key(axum::http::header::ORIGIN)
        && headers.contains_key(axum::http::header::ACCESS_CONTROL_REQUEST_METHOD)
}

// Note: Unit tests for auth_with_policy are in tests/auth_integration.rs
// Direct unit testing requires the full Axum middleware stack, so integration tests are more appropriate.
