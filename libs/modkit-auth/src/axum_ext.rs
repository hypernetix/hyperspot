//! Axum extractors and middleware for auth

use crate::{
    claims::Claims,
    errors::AuthError,
    traits::{PrimaryAuthorizer, ScopeBuilder, TokenValidator},
    types::{AuthRequirement, RoutePolicy},
};
use axum::{
    body::Body,
    extract::{FromRequestParts, Request},
    http::{request::Parts, HeaderMap, Method},
    response::{IntoResponse, Response},
};
use modkit_security::{SecurityContext, SecurityCtx};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service};

/// Extractor for `SecurityCtx` - validates that auth middleware has run
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
                "SecurityCtx not found - auth middleware not configured".to_owned(),
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
                "Claims not found - auth middleware not configured".to_owned(),
            ))
    }
}

/// Shared state for authentication policy middleware.
struct AuthPolicyState {
    validator: Arc<dyn TokenValidator>,
    scope_builder: Arc<dyn ScopeBuilder>,
    authorizer: Arc<dyn PrimaryAuthorizer>,
    policy: Arc<dyn RoutePolicy>,
}

/// Layer that applies authentication policy middleware to services.
///
/// # Example
/// ```ignore
/// router = router.layer(AuthPolicyLayer::new(validator, scope_builder, authorizer, policy));
/// ```
#[derive(Clone)]
pub struct AuthPolicyLayer {
    state: Arc<AuthPolicyState>,
}

impl AuthPolicyLayer {
    pub fn new(
        validator: Arc<dyn TokenValidator>,
        scope_builder: Arc<dyn ScopeBuilder>,
        authorizer: Arc<dyn PrimaryAuthorizer>,
        policy: Arc<dyn RoutePolicy>,
    ) -> Self {
        Self {
            state: Arc::new(AuthPolicyState {
                validator,
                scope_builder,
                authorizer,
                policy,
            }),
        }
    }
}

impl<S> Layer<S> for AuthPolicyLayer {
    type Service = AuthPolicyService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthPolicyService {
            inner,
            state: self.state.clone(),
        }
    }
}

/// Service that applies authentication policy to requests.
#[derive(Clone)]
pub struct AuthPolicyService<S> {
    inner: S,
    state: Arc<AuthPolicyState>,
}

impl<S> Service<Request<Body>> for AuthPolicyService<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<Body>) -> Self::Future {
        let state = self.state.clone();
        let not_ready_inner = self.inner.clone();
        let mut ready_inner = std::mem::replace(&mut self.inner, not_ready_inner);

        Box::pin(async move {
            // 1. Skips authentication for CORS preflight requests
            if is_preflight_request(request.method(), request.headers()) {
                return ready_inner.call(request).await;
            }

            // 2. Resolves the route's authentication requirement using RoutePolicy
            let auth_requirement = state
                .policy
                .resolve(request.method(), request.uri().path())
                .await;

            match auth_requirement {
                AuthRequirement::None => {
                    // 3. For public routes (AuthRequirement::None): inserts anonymous SecurityCtx
                    request.extensions_mut().insert(SecurityCtx::anonymous());
                    request
                        .extensions_mut()
                        .insert(SecurityContext::anonymous());
                    ready_inner.call(request).await
                }
                AuthRequirement::Required(sec_requirement) => {
                    // 4. For required routes: validates JWT, enforces RBAC if needed, inserts SecurityCtx
                    let Some(token) = extract_bearer_token(request.headers()) else {
                        return Ok(AuthError::Unauthenticated.into_response());
                    };

                    let claims = match state.validator.validate_and_parse(token).await {
                        Ok(claims) => claims,
                        Err(err) => {
                            return Ok(err.into_response());
                        }
                    };

                    // Optional RBAC requirement
                    if let Some(sec_req) = sec_requirement {
                        if let Err(err) = state.authorizer.check(&claims, &sec_req).await {
                            return Ok(err.into_response());
                        }
                    }

                    // Build SecurityCtx from validated claims (legacy)
                    let scope = state.scope_builder.tenants_to_scope(&claims);
                    let sec =
                        SecurityCtx::new(scope, modkit_security::Subject::new(claims.subject));

                    // Build SecurityContext from validated claims (new)
                    let sec_context = SecurityContext::builder()
                        .tenant_id(claims.tenant_id)
                        .subject_id(claims.subject)
                        .build();

                    request.extensions_mut().insert(claims);
                    request.extensions_mut().insert(sec);
                    request.extensions_mut().insert(sec_context);
                    ready_inner.call(request).await
                }
                AuthRequirement::Optional => {
                    // 5. For optional routes: validates JWT if present, otherwise inserts anonymous SecurityCtx
                    if let Some(token) = extract_bearer_token(request.headers()) {
                        match state.validator.validate_and_parse(token).await {
                            Ok(claims) => {
                                // Build SecurityCtx from validated claims (legacy)
                                let scope = state.scope_builder.tenants_to_scope(&claims);
                                let sec = SecurityCtx::new(
                                    scope,
                                    modkit_security::Subject::new(claims.subject),
                                );

                                // Build SecurityContext from validated claims (new)
                                let sec_context = SecurityContext::builder()
                                    .tenant_id(claims.tenant_id)
                                    .subject_id(claims.subject)
                                    .build();

                                request.extensions_mut().insert(claims);
                                request.extensions_mut().insert(sec);
                                request.extensions_mut().insert(sec_context);
                            }
                            Err(err) => {
                                tracing::debug!("Optional auth: invalid token: {err}");
                                request.extensions_mut().insert(SecurityCtx::anonymous());
                                request
                                    .extensions_mut()
                                    .insert(SecurityContext::anonymous());
                            }
                        }
                    } else {
                        request.extensions_mut().insert(SecurityCtx::anonymous());
                        request
                            .extensions_mut()
                            .insert(SecurityContext::anonymous());
                    }
                    ready_inner.call(request).await
                }
            }
        })
    }
}

/// Extract Bearer token from Authorization header
fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ").map(str::trim))
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

// Note: Unit tests for AuthPolicyLayer are in tests/auth_integration.rs
// Direct unit testing requires the full Axum middleware stack, so integration tests are more appropriate.
