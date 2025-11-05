use axum::http::Method;
use modkit_auth::{
    authorizer::RoleAuthorizer,
    errors::AuthError,
    jwks::JwksValidator,
    scope_builder::SimpleScopeBuilder,
    traits::{PrimaryAuthorizer, ScopeBuilder, TokenValidator},
    types::SecRequirement,
    AuthRequirement, Claims, RoutePolicy,
};
use std::{collections::HashMap, sync::Arc};

/// Route matcher for a specific HTTP method
#[derive(Clone)]
pub(crate) struct RouteMatcher {
    matcher: matchit::Router<SecRequirement>,
}

impl RouteMatcher {
    fn new() -> Self {
        Self {
            matcher: matchit::Router::new(),
        }
    }

    fn insert(
        &mut self,
        path: &str,
        requirement: SecRequirement,
    ) -> Result<(), matchit::InsertError> {
        self.matcher.insert(path, requirement)
    }

    fn find(&self, path: &str) -> Option<&SecRequirement> {
        self.matcher.at(path).ok().map(|m| m.value)
    }
}

/// Public route matcher for a specific HTTP method (pattern-based)
#[derive(Clone)]
pub(crate) struct PublicRouteMatcher {
    matcher: matchit::Router<()>,
}

impl PublicRouteMatcher {
    fn new() -> Self {
        Self {
            matcher: matchit::Router::new(),
        }
    }

    fn insert(&mut self, path: &str) -> Result<(), matchit::InsertError> {
        self.matcher.insert(path, ())
    }

    fn find(&self, path: &str) -> bool {
        self.matcher.at(path).is_ok()
    }
}

/// Simplified auth state containing only what's needed for the unified middleware
#[derive(Clone)]
pub struct AuthState {
    pub validator: Arc<dyn TokenValidator>,
    pub scope_builder: Arc<dyn ScopeBuilder>,
    pub authorizer: Arc<dyn PrimaryAuthorizer>,
}

/// Implementation of RoutePolicy for api_ingress that uses operation specs
#[derive(Clone)]
pub struct IngressRoutePolicy {
    route_matchers: Arc<HashMap<Method, RouteMatcher>>,
    public_matchers: Arc<HashMap<Method, PublicRouteMatcher>>,
    require_auth_by_default: bool,
}

impl IngressRoutePolicy {
    pub fn new(
        route_matchers: Arc<HashMap<Method, RouteMatcher>>,
        public_matchers: Arc<HashMap<Method, PublicRouteMatcher>>,
        require_auth_by_default: bool,
    ) -> Self {
        Self {
            route_matchers,
            public_matchers,
            require_auth_by_default,
        }
    }
}

#[async_trait::async_trait]
impl RoutePolicy for IngressRoutePolicy {
    async fn resolve(&self, method: &Method, path: &str) -> AuthRequirement {
        // Find requirement using pattern matching (returns SecRequirement directly now)
        let requirement = self
            .route_matchers
            .get(method)
            .and_then(|matcher| matcher.find(path))
            .cloned();

        // Check if route is explicitly public using pattern matching
        let is_public = self
            .public_matchers
            .get(method)
            .map(|matcher| matcher.find(path))
            .unwrap_or(false);

        // Determine if authentication is needed
        // Public routes should NEVER require auth, even if require_auth_by_default is true
        let needs_authn = requirement.is_some() || (self.require_auth_by_default && !is_public);

        if !needs_authn {
            AuthRequirement::None
        } else {
            // SecRequirement is already properly typed, just wrap it
            AuthRequirement::Required(requirement)
        }
    }
}

/// Create a noop validator for disabled auth mode
pub struct NoopValidator;

#[async_trait::async_trait]
impl TokenValidator for NoopValidator {
    async fn validate_and_parse(&self, _token: &str) -> Result<Claims, AuthError> {
        unreachable!("NoopValidator should never be called")
    }
}

/// Helper to build AuthState and IngressRoutePolicy from config
pub fn build_auth_state(
    cfg: &crate::config::ApiIngressConfig,
    requirements: HashMap<(Method, String), SecRequirement>,
    public_routes: std::collections::HashSet<(Method, String)>,
) -> Result<(AuthState, IngressRoutePolicy), anyhow::Error> {
    // Build validator based on whether auth is enabled
    let validator: Arc<dyn TokenValidator> = if !cfg.auth_disabled {
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

    // Build public route matchers per HTTP method (pattern-based)
    let mut public_matchers_map: HashMap<Method, PublicRouteMatcher> = HashMap::new();

    for (method, path) in &public_routes {
        let matcher = public_matchers_map
            .entry(method.clone())
            .or_insert_with(PublicRouteMatcher::new);
        matcher.insert(path).map_err(|e| {
            anyhow::anyhow!("Failed to insert public route pattern '{}': {}", path, e)
        })?;
    }

    let route_matchers = Arc::new(route_matchers_map);
    let public_matchers = Arc::new(public_matchers_map);

    let auth_state = AuthState {
        validator,
        scope_builder,
        authorizer,
    };

    let route_policy =
        IngressRoutePolicy::new(route_matchers, public_matchers, cfg.require_auth_by_default);

    Ok((auth_state, route_policy))
}
