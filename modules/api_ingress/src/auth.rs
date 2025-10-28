use axum::{
    extract::Request,
    http::{header, Method},
    middleware::Next,
    response::Response,
};
use modkit_auth::{
    authorizer::RoleAuthorizer,
    errors::AuthError,
    jwks::JwksValidator,
    scope_builder::SimpleScopeBuilder,
    traits::{PrimaryAuthorizer, ScopeBuilder, TokenValidator},
    types::SecRequirement,
    Claims,
};
use modkit_security::{SecurityCtx, Subject};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthMode {
    Enabled,
    Disabled,
}

#[derive(Clone)]
pub struct AuthConfig {
    pub mode: AuthMode,
    /// Whether routes without explicit role still require a valid token.
    pub require_auth_by_default: bool,
}

#[derive(Clone)]
pub struct Requirement {
    pub resource: String,
    pub action: String,
}

/// Route matcher for a specific HTTP method
struct RouteMatcher {
    matcher: matchit::Router<Requirement>,
}

impl RouteMatcher {
    fn new() -> Self {
        Self {
            matcher: matchit::Router::new(),
        }
    }

    fn insert(&mut self, path: &str, requirement: Requirement) -> Result<(), matchit::InsertError> {
        self.matcher.insert(path, requirement)
    }

    fn find(&self, path: &str) -> Option<&Requirement> {
        self.matcher.at(path).ok().map(|m| m.value)
    }
}

/// Global state for the auth middleware.
#[derive(Clone)]
pub struct AuthState {
    pub cfg: AuthConfig,
    pub validator: Arc<dyn TokenValidator>,
    pub scope_builder: Arc<dyn ScopeBuilder>,
    pub authorizer: Arc<dyn PrimaryAuthorizer>,
    /// Route matchers per HTTP method for efficient pattern matching
    route_matchers: Arc<HashMap<Method, RouteMatcher>>,
    /// Set of route patterns explicitly marked as public (no auth required).
    pub public_routes: Arc<std::collections::HashSet<(Method, String)>>,
}

/// Auth middleware implementation
pub async fn auth_middleware(
    state: AuthState,
    mut req: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Disabled mode: inject root SecurityCtx and continue
    if state.cfg.mode == AuthMode::Disabled {
        let sec = SecurityCtx::root_ctx(); // subject=root, scope=all tenants
        req.extensions_mut().insert(sec);
        return Ok(next.run(req).await);
    }

    // Enabled mode - use route pattern matching
    let method = req.method();
    let path = req.uri().path();

    // Find requirement using pattern matching
    let requirement = state
        .route_matchers
        .get(method)
        .and_then(|matcher| matcher.find(path))
        .cloned();

    // Check if route pattern is explicitly public
    let key = (method.clone(), path.to_string());
    let is_public = state.public_routes.contains(&key);
    let needs_authn = requirement.is_some() || (state.cfg.require_auth_by_default && !is_public);

    // Extract Bearer token if required
    let bearer = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let claims = if needs_authn {
        let token = bearer.ok_or(AuthError::Unauthenticated)?;
        Some(state.validator.validate_and_parse(&token).await?)
    } else {
        None
    };

    // Role-based authorization
    if let (Some(claims_ref), Some(reqm)) = (claims.as_ref(), requirement.as_ref()) {
        let sec_req = SecRequirement {
            resource: Box::leak(reqm.resource.clone().into_boxed_str()),
            action: Box::leak(reqm.action.clone().into_boxed_str()),
        };
        state.authorizer.check(claims_ref, &sec_req).await?;
    }

    // Build and attach SecurityCtx
    if let Some(claims) = claims {
        let scope = state.scope_builder.tenants_to_scope(&claims);
        let sec = SecurityCtx::new(scope, Subject::new(claims.sub));
        req.extensions_mut().insert(sec);
        req.extensions_mut().insert(claims);
    } else {
        // No token required: attach root context (for public routes)
        let sec = SecurityCtx::root_ctx();
        req.extensions_mut().insert(sec);
    }

    Ok(next.run(req).await)
}

/// Create a noop validator for disabled auth mode
pub struct NoopValidator;

#[async_trait::async_trait]
impl TokenValidator for NoopValidator {
    async fn validate_and_parse(&self, _token: &str) -> Result<Claims, AuthError> {
        unreachable!("NoopValidator should never be called")
    }
}

/// Helper to build AuthState from config
pub fn build_auth_state(
    cfg: &crate::config::ApiIngressConfig,
    requirements: HashMap<(Method, String), Requirement>,
    public_routes: std::collections::HashSet<(Method, String)>,
) -> Result<AuthState, anyhow::Error> {
    let mode = if cfg.auth_disabled {
        AuthMode::Disabled
    } else {
        AuthMode::Enabled
    };

    let validator: Arc<dyn TokenValidator> = if mode == AuthMode::Enabled {
        let jwks = cfg
            .jwks_uri
            .clone()
            .ok_or_else(|| anyhow::anyhow!("jwks_uri required when auth is enabled"))?;
        Arc::new(JwksValidator::new(
            jwks,
            cfg.issuer.clone(),
            cfg.audience.clone(),
        ))
    } else {
        Arc::new(NoopValidator)
    };

    let scope_builder: Arc<dyn ScopeBuilder> = Arc::new(SimpleScopeBuilder);
    let authorizer: Arc<dyn PrimaryAuthorizer> = Arc::new(RoleAuthorizer);

    // Build route matchers per HTTP method
    let mut route_matchers_map: HashMap<Method, RouteMatcher> = HashMap::new();

    for ((method, path), requirement) in requirements {
        let matcher = route_matchers_map
            .entry(method)
            .or_insert_with(RouteMatcher::new);
        matcher
            .insert(&path, requirement)
            .map_err(|e| anyhow::anyhow!("Failed to insert route pattern '{}': {}", path, e))?;
    }

    Ok(AuthState {
        cfg: AuthConfig {
            mode,
            require_auth_by_default: cfg.require_auth_by_default,
        },
        validator,
        scope_builder,
        authorizer,
        route_matchers: Arc::new(route_matchers_map),
        public_routes: Arc::new(public_routes),
    })
}
