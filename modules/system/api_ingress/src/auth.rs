use axum::http::Method;
use modkit_auth::{
    authorizer::RoleAuthorizer,
    build_auth_dispatcher,
    scope_builder::SimpleScopeBuilder,
    traits::{PrimaryAuthorizer, ScopeBuilder, TokenValidator},
    types::{AuthRequirement, SecRequirement},
    AuthConfig as ModkitAuthConfig, AuthModeConfig, JwksConfig, PluginConfig,
};
use modkit_security::{DummyPolicyEngine, PolicyEngineRef};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct Requirement {
    pub resource: String,
    pub action: String,
}

/// Route matcher for a specific HTTP method (secured routes with requirements)
#[derive(Clone)]
pub struct RouteMatcher {
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

/// Public route matcher for explicitly public routes
#[derive(Clone)]
pub struct PublicRouteMatcher {
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

/// Convert Axum path syntax `:param` to matchit syntax `{param}`
///
/// Axum uses `:id` for path parameters, but matchit 0.8 uses `{id}`.
/// This function converts between the two syntaxes.
fn convert_axum_path_to_matchit(path: &str) -> String {
    // Simple regex-free approach: find :word and replace with {word}
    let mut result = String::with_capacity(path.len());
    let mut chars = path.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == ':' {
            // Start of a parameter - collect the parameter name
            result.push('{');
            while matches!(chars.peek(), Some(c) if c.is_alphanumeric() || *c == '_') {
                if let Some(c) = chars.next() {
                    result.push(c);
                }
            }
            result.push('}');
        } else {
            result.push(ch);
        }
    }

    result
}

/// Simplified auth state containing only the core auth components
#[derive(Clone)]
pub struct AuthState {
    pub validator: Arc<dyn TokenValidator>,
    pub policy_engine: PolicyEngineRef,
    pub scope_builder: Arc<dyn ScopeBuilder>,
    pub authorizer: Arc<dyn PrimaryAuthorizer>,
}

/// Ingress-specific route policy implementation
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
impl modkit_auth::RoutePolicy for IngressRoutePolicy {
    async fn resolve(&self, method: &Method, path: &str) -> AuthRequirement {
        // Find requirement using pattern matching
        let requirement = self
            .route_matchers
            .get(method)
            .and_then(|matcher| matcher.find(path))
            .cloned();

        // Check if route is explicitly public using pattern matching
        let is_public = self
            .public_matchers
            .get(method)
            .is_some_and(|matcher| matcher.find(path));

        // Public routes should not be forced to auth by default
        let needs_authn = requirement.is_some() || (self.require_auth_by_default && !is_public);

        if needs_authn {
            AuthRequirement::Required(requirement)
        } else {
            AuthRequirement::None
        }
    }
}

// Old auth_middleware has been removed.
// Use modkit_auth::axum_ext::auth_with_policy via IngressRoutePolicy instead.

/// Helper to build `AuthState` and `IngressRoutePolicy` from config
///
/// # Note on `auth_disabled` mode
///
/// When `cfg.auth_disabled == true`:
/// - This function is still called to build the auth components for type consistency
/// - A `NoopValidator` is created as a **defensive fallback only**
/// - In normal flow, `lib.rs` bypasses `auth_with_policy` entirely when auth is disabled
/// - The router-level middleware injects `SecurityCtx::root_ctx()` directly
/// - The `NoopValidator` exists only as a safety net in case someone accidentally
///   wires `auth_with_policy` while `auth_disabled == true`
/// - If the `NoopValidator` is ever called, it will panic with a clear error message
///
/// When `cfg.auth_disabled == false`:
/// - This function builds the real OIDC/JWKS validator and associated auth components
/// - `auth_with_policy` is wired into the router to validate tokens and build security contexts
pub fn build_auth_state(
    cfg: &crate::config::ApiIngressConfig,
    requirements: HashMap<(Method, String), Requirement>,
    public_routes: std::collections::HashSet<(Method, String)>,
) -> Result<(AuthState, IngressRoutePolicy), anyhow::Error> {
    // Build validator (TokenValidator trait implementation)
    let validator: Arc<dyn TokenValidator> = if cfg.auth_disabled {
        // Defensive fallback: NoopValidator should never be called in normal flow.
        // When auth_disabled=true, lib.rs bypasses auth_with_policy and injects root_ctx directly.
        // This validator exists only for type consistency and as a safety net.
        Arc::new(NoopValidator)
    } else {
        // Build AuthConfig for new dispatcher system
        let jwks_uri = cfg
            .jwks_uri
            .clone()
            .ok_or_else(|| anyhow::anyhow!("jwks_uri required when auth is enabled"))?;

        let mut plugins = HashMap::new();
        plugins.insert(
            "default-oidc".to_owned(),
            PluginConfig::Oidc {
                tenant_claim: "tenants".to_owned(),
                roles_claim: "roles".to_owned(),
            },
        );

        let auth_config = ModkitAuthConfig {
            mode: AuthModeConfig {
                provider: "default-oidc".to_owned(),
            },
            leeway_seconds: 60,
            issuers: cfg
                .issuer
                .as_ref()
                .map(|i| vec![i.clone()])
                .unwrap_or_default(),
            audiences: cfg
                .audience
                .as_ref()
                .map(|a| vec![a.clone()])
                .unwrap_or_default(),
            jwks: Some(JwksConfig {
                uri: jwks_uri,
                refresh_interval_seconds: 300,
                max_backoff_seconds: 3600,
            }),
            plugins,
        };

        // Build dispatcher and use it as validator
        let dispatcher = build_auth_dispatcher(&auth_config)
            .map_err(|e| anyhow::anyhow!("Failed to build auth dispatcher: {e}"))?;

        Arc::new(dispatcher) as Arc<dyn TokenValidator>
    };

    let scope_builder: Arc<dyn ScopeBuilder> = Arc::new(SimpleScopeBuilder);
    let authorizer: Arc<dyn PrimaryAuthorizer> = Arc::new(RoleAuthorizer);

    // Build route matchers per HTTP method (secured routes with requirements)
    let mut route_matchers_map: HashMap<Method, RouteMatcher> = HashMap::new();

    for ((method, path), requirement) in requirements {
        let sec_req = SecRequirement::new(requirement.resource, requirement.action);
        let matcher = route_matchers_map
            .entry(method)
            .or_insert_with(RouteMatcher::new);
        // Convert Axum path syntax (:param) to matchit syntax ({param})
        let matchit_path = convert_axum_path_to_matchit(&path);
        matcher
            .insert(&matchit_path, sec_req)
            .map_err(|e| anyhow::anyhow!("Failed to insert route pattern '{path}': {e}"))?;
    }

    // Build public matchers per HTTP method
    let mut public_matchers_map: HashMap<Method, PublicRouteMatcher> = HashMap::new();

    for (method, path) in public_routes {
        let matcher = public_matchers_map
            .entry(method)
            .or_insert_with(PublicRouteMatcher::new);
        // Convert Axum path syntax (:param) to matchit syntax ({param})
        let matchit_path = convert_axum_path_to_matchit(&path);
        matcher
            .insert(&matchit_path)
            .map_err(|e| anyhow::anyhow!("Failed to insert public route pattern '{path}': {e}"))?;
    }

    let auth_state = AuthState {
        validator,
        policy_engine: Arc::new(DummyPolicyEngine),
        scope_builder,
        authorizer,
    };

    let route_policy = IngressRoutePolicy::new(
        Arc::new(route_matchers_map),
        Arc::new(public_matchers_map),
        cfg.require_auth_by_default,
    );

    Ok((auth_state, route_policy))
}

/// No-op validator for `auth_disabled` mode (should never be called)
struct NoopValidator;

#[async_trait::async_trait]
impl TokenValidator for NoopValidator {
    async fn validate_and_parse(
        &self,
        _token: &str,
    ) -> Result<modkit_auth::Claims, modkit_auth::AuthError> {
        panic!(
            "NoopValidator should never be called - auth_disabled mode should bypass validation"
        );
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use axum::http::Method;
    use modkit_auth::types::RoutePolicy;

    /// Helper to build `IngressRoutePolicy` with given matchers
    fn build_test_policy(
        route_matchers: HashMap<Method, RouteMatcher>,
        public_matchers: HashMap<Method, PublicRouteMatcher>,
        require_auth_by_default: bool,
    ) -> IngressRoutePolicy {
        IngressRoutePolicy::new(
            Arc::new(route_matchers),
            Arc::new(public_matchers),
            require_auth_by_default,
        )
    }

    #[test]
    fn test_convert_axum_path_to_matchit() {
        assert_eq!(convert_axum_path_to_matchit("/users/:id"), "/users/{id}");
        assert_eq!(
            convert_axum_path_to_matchit("/posts/:post_id/comments/:comment_id"),
            "/posts/{post_id}/comments/{comment_id}"
        );
        assert_eq!(convert_axum_path_to_matchit("/health"), "/health"); // No params
        assert_eq!(
            convert_axum_path_to_matchit("/api/v1/:resource/:id/status"),
            "/api/v1/{resource}/{id}/status"
        );
    }

    #[test]
    fn test_matchit_router_with_params() {
        // matchit 0.8 uses {param} syntax for path parameters (NOT :param)
        let mut router = matchit::Router::new();
        router.insert("/users/{id}", "user_route").unwrap();

        let result = router.at("/users/42");
        assert!(
            result.is_ok(),
            "matchit should match /users/{{id}} against /users/42"
        );
        assert_eq!(*result.unwrap().value, "user_route");
    }

    #[tokio::test]
    async fn explicit_public_route_with_path_params_returns_none() {
        let mut public_matchers = HashMap::new();
        let mut matcher = PublicRouteMatcher::new();
        // matchit 0.8 uses {param} syntax (Axum uses :param, so conversion needed in production)
        matcher.insert("/users/{id}").unwrap();

        public_matchers.insert(Method::GET, matcher);

        let policy = build_test_policy(HashMap::new(), public_matchers, true);

        // Path parameters should match concrete values
        let result = policy.resolve(&Method::GET, "/users/42").await;
        assert_eq!(result, AuthRequirement::None);
    }

    #[tokio::test]
    async fn explicit_public_route_exact_match_returns_none() {
        let mut public_matchers = HashMap::new();
        let mut matcher = PublicRouteMatcher::new();
        matcher.insert("/health").unwrap();
        public_matchers.insert(Method::GET, matcher);

        let policy = build_test_policy(HashMap::new(), public_matchers, true);

        let result = policy.resolve(&Method::GET, "/health").await;
        assert_eq!(result, AuthRequirement::None);
    }

    #[tokio::test]
    async fn explicit_secured_route_with_requirement_returns_required() {
        let mut route_matchers = HashMap::new();
        let mut matcher = RouteMatcher::new();
        let sec_req = SecRequirement::new("admin", "access");
        matcher.insert("/admin/metrics", sec_req.clone()).unwrap();
        route_matchers.insert(Method::GET, matcher);

        let policy = build_test_policy(route_matchers, HashMap::new(), false);

        let result = policy.resolve(&Method::GET, "/admin/metrics").await;
        match result {
            AuthRequirement::Required(Some(req)) => {
                assert_eq!(req.resource, "admin");
                assert_eq!(req.action, "access");
            }
            _ => panic!("Expected Required with SecRequirement"),
        }
    }

    #[tokio::test]
    async fn route_without_requirement_with_require_auth_by_default_returns_required_none() {
        let policy = build_test_policy(HashMap::new(), HashMap::new(), true);

        let result = policy.resolve(&Method::GET, "/profile").await;
        assert_eq!(result, AuthRequirement::Required(None));
    }

    #[tokio::test]
    async fn route_without_requirement_without_require_auth_by_default_returns_none() {
        let policy = build_test_policy(HashMap::new(), HashMap::new(), false);

        let result = policy.resolve(&Method::GET, "/profile").await;
        assert_eq!(result, AuthRequirement::None);
    }

    #[tokio::test]
    async fn unknown_route_with_require_auth_by_default_true_returns_required() {
        let policy = build_test_policy(HashMap::new(), HashMap::new(), true);

        let result = policy.resolve(&Method::POST, "/unknown").await;
        assert_eq!(result, AuthRequirement::Required(None));
    }

    #[tokio::test]
    async fn unknown_route_with_require_auth_by_default_false_returns_none() {
        let policy = build_test_policy(HashMap::new(), HashMap::new(), false);

        let result = policy.resolve(&Method::POST, "/unknown").await;
        assert_eq!(result, AuthRequirement::None);
    }

    #[tokio::test]
    async fn public_route_overrides_require_auth_by_default() {
        let mut public_matchers = HashMap::new();
        let mut matcher = PublicRouteMatcher::new();
        matcher.insert("/public").unwrap();
        public_matchers.insert(Method::GET, matcher);

        let policy = build_test_policy(HashMap::new(), public_matchers, true);

        let result = policy.resolve(&Method::GET, "/public").await;
        assert_eq!(result, AuthRequirement::None);
    }

    #[tokio::test]
    async fn secured_route_has_priority_over_default() {
        let mut route_matchers = HashMap::new();
        let mut matcher = RouteMatcher::new();
        let sec_req = SecRequirement::new("users", "read");
        // matchit 0.8 uses {param} syntax
        matcher.insert("/users/{id}", sec_req).unwrap();
        route_matchers.insert(Method::GET, matcher);

        let policy = build_test_policy(route_matchers, HashMap::new(), false);

        let result = policy.resolve(&Method::GET, "/users/123").await;
        match result {
            AuthRequirement::Required(Some(req)) => {
                assert_eq!(req.resource, "users");
                assert_eq!(req.action, "read");
            }
            _ => panic!("Expected Required with SecRequirement"),
        }
    }

    #[tokio::test]
    async fn different_methods_resolve_independently() {
        let mut route_matchers = HashMap::new();

        // GET /users is secured
        let mut get_matcher = RouteMatcher::new();
        let sec_req = SecRequirement::new("users", "read");
        get_matcher
            .insert("/user-management/v1/users", sec_req)
            .unwrap();
        route_matchers.insert(Method::GET, get_matcher);

        // POST /users is not in matchers
        let policy = build_test_policy(route_matchers, HashMap::new(), false);

        // GET should be secured
        let get_result = policy
            .resolve(&Method::GET, "/user-management/v1/users")
            .await;
        assert!(matches!(get_result, AuthRequirement::Required(Some(_))));

        // POST should be public (no requirement, require_auth_by_default=false)
        let post_result = policy
            .resolve(&Method::POST, "/user-management/v1/users")
            .await;
        assert_eq!(post_result, AuthRequirement::None);
    }
}
