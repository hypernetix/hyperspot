//! Axum extractors and middleware for auth-related types

use crate::{
    errors::AuthError,
    traits::{PrimaryAuthorizer, ScopeBuilder, TokenValidator},
    types::AuthRequirement,
    RoutePolicy,
};
use axum::{
    extract::{FromRequestParts, Request, State},
    http::{header, request::Parts, Method},
    middleware::Next,
    response::{IntoResponse, Response},
};
use modkit_security::{SecurityCtx, Subject};
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

/// Re-export Claims for convenience
pub use crate::claims::Claims;

/// Helper to check if request is a CORS preflight
fn is_preflight_request(method: &Method, headers: &axum::http::HeaderMap) -> bool {
    method == Method::OPTIONS
        && (headers.contains_key(header::ACCESS_CONTROL_REQUEST_METHOD)
            || headers.contains_key(header::ACCESS_CONTROL_REQUEST_HEADERS))
}

/// Extract bearer token from Authorization header
fn extract_bearer_token(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

/// Unified authentication middleware using RoutePolicy
///
/// This middleware centralizes all JWT validation, SecurityCtx creation, and authorization logic.
/// The route-level policy (public vs. required, with optional RBAC requirements) is determined
/// by the `RoutePolicy` trait implementation passed in via state.
///
/// Returns a `Response` directly, handling all errors internally via early returns.
pub async fn auth_with_policy(
    State(validator): State<Arc<dyn TokenValidator>>,
    State(scope_builder): State<Arc<dyn ScopeBuilder>>,
    State(authorizer): State<Arc<dyn PrimaryAuthorizer>>,
    State(policy): State<Arc<dyn RoutePolicy>>,
    mut request: Request,
    next: Next,
) -> Response {
    // 1. Handle CORS preflight - skip all auth logic
    if is_preflight_request(request.method(), request.headers()) {
        return next.run(request).await;
    }

    // 2. Resolve route policy
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let auth_requirement = policy.resolve(&method, &path).await;

    // 3. Handle based on requirement
    match auth_requirement {
        AuthRequirement::None => {
            // Public route - no authentication required
            // Insert an anonymous SecurityCtx with no access to any resources
            let sec = SecurityCtx::anonymous();
            request.extensions_mut().insert(sec);
            next.run(request).await
        }
        AuthRequirement::Required(sec_requirement) => {
            // Authentication required
            let token = match extract_bearer_token(request.headers()) {
                Some(token) => token,
                None => {
                    // No token and auth is required
                    return AuthError::Unauthenticated.into_response();
                }
            };

            // Validate JWT and extract claims
            let claims = match validator.validate_and_parse(&token).await {
                Ok(claims) => claims,
                Err(err) => {
                    return err.into_response();
                }
            };

            // If there's a specific security requirement, check authorization
            if let Some(sec_req) = sec_requirement {
                if let Err(err) = authorizer.check(&claims, &sec_req).await {
                    return err.into_response();
                }
            }

            // Build SecurityCtx from validated claims
            let scope = scope_builder.tenants_to_scope(&claims);
            let sec = SecurityCtx::new(scope, Subject::new(claims.sub));

            // Insert Claims and SecurityCtx into request extensions
            request.extensions_mut().insert(claims);
            request.extensions_mut().insert(sec);

            next.run(request).await
        }
        AuthRequirement::Optional => {
            // Optional authentication: use token if present and valid, otherwise anonymous
            if let Some(token) = extract_bearer_token(request.headers()) {
                match validator.validate_and_parse(&token).await {
                    Ok(claims) => {
                        // Valid token: build SecurityCtx from claims
                        let scope = scope_builder.tenants_to_scope(&claims);
                        let sec = SecurityCtx::new(scope, Subject::new(claims.sub));
                        request.extensions_mut().insert(claims);
                        request.extensions_mut().insert(sec);
                    }
                    Err(err) => {
                        // Invalid token: log and proceed with anonymous context
                        tracing::debug!("Optional auth: invalid token: {err}");
                        request.extensions_mut().insert(SecurityCtx::anonymous());
                    }
                }
            } else {
                // No token: proceed with anonymous context
                request.extensions_mut().insert(SecurityCtx::anonymous());
            }

            next.run(request).await
        }
    }
}

/// Static route policy that always returns the same requirement
///
/// This is used by `auth_required` and `auth_optional` to provide a simple
/// authentication policy without needing route-specific logic.
#[derive(Clone)]
struct StaticRoutePolicy {
    requirement: AuthRequirement,
}

impl StaticRoutePolicy {
    fn new(requirement: AuthRequirement) -> Self {
        Self { requirement }
    }
}

#[async_trait::async_trait]
impl RoutePolicy for StaticRoutePolicy {
    async fn resolve(&self, _method: &Method, _path: &str) -> AuthRequirement {
        self.requirement.clone()
    }
}

/// Authentication required middleware
///
/// This middleware requires a valid JWT token for all requests (except CORS preflight).
/// Use this for routes that always need authentication but don't have specific RBAC requirements.
///
/// # Example
/// ```ignore
/// use axum::Router;
/// use modkit_auth::axum_ext::auth_required;
///
/// let router = Router::new()
///     .route("/protected", get(handler))
///     .layer(auth_required(validator, scope_builder, authorizer));
/// ```
pub fn auth_required(
    validator: Arc<dyn TokenValidator>,
    scope_builder: Arc<dyn ScopeBuilder>,
    authorizer: Arc<dyn PrimaryAuthorizer>,
) -> impl Clone {
    let policy = Arc::new(StaticRoutePolicy::new(AuthRequirement::Required(None)))
        as Arc<dyn RoutePolicy>;

    // Type annotation needed for state parameter to help compiler inference
    axum::middleware::from_fn::<_, ()>(move |req: Request, next: Next| {
        let validator = validator.clone();
        let scope_builder = scope_builder.clone();
        let authorizer = authorizer.clone();
        let policy = policy.clone();
        async move {
            auth_with_policy(
                State(validator),
                State(scope_builder),
                State(authorizer),
                State(policy),
                req,
                next,
            )
            .await
        }
    })
}

/// Authentication optional middleware
///
/// This middleware allows requests without authentication, inserting an anonymous SecurityCtx
/// for unauthenticated requests. Use this for routes that can work with or without authentication.
///
/// # Example
/// ```ignore
/// use axum::Router;
/// use modkit_auth::axum_ext::auth_optional;
///
/// let router = Router::new()
///     .route("/public-or-authenticated", get(handler))
///     .layer(auth_optional(validator, scope_builder, authorizer));
/// ```
pub fn auth_optional(
    validator: Arc<dyn TokenValidator>,
    scope_builder: Arc<dyn ScopeBuilder>,
    authorizer: Arc<dyn PrimaryAuthorizer>,
) -> impl Clone {
    let policy = Arc::new(StaticRoutePolicy::new(AuthRequirement::Optional)) as Arc<dyn RoutePolicy>;

    // Type annotation needed for state parameter to help compiler inference
    axum::middleware::from_fn::<_, ()>(move |req: Request, next: Next| {
        let validator = validator.clone();
        let scope_builder = scope_builder.clone();
        let authorizer = authorizer.clone();
        let policy = policy.clone();
        async move {
            auth_with_policy(
                State(validator),
                State(scope_builder),
                State(authorizer),
                State(policy),
                req,
                next,
            )
            .await
        }
    })
}
