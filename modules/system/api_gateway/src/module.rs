//! API Gateway Module definition
//!
//! Contains the `ApiGateway` module struct and its trait implementations.

use async_trait::async_trait;
use std::sync::Arc;

use arc_swap::ArcSwap;
use dashmap::DashMap;

use anyhow::Result;
use axum::extract::State;
use axum::http::Method;
use axum::middleware::from_fn_with_state;
use axum::{Router, extract::DefaultBodyLimit, middleware::from_fn, routing::get};
use license_enforcer_sdk::LicenseEnforcerGatewayClient;
use modkit::api::{OpenApiRegistry, OpenApiRegistryImpl};
use modkit::lifecycle::ReadySignal;
use parking_lot::Mutex;
use std::net::SocketAddr;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tower_http::{
    limit::RequestBodyLimitLayer,
    request_id::{PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
};
use tracing::debug;

use crate::auth;
use crate::config::ApiGatewayConfig;
use modkit_security::constants::{DEFAULT_SUBJECT_ID, DEFAULT_TENANT_ID};
use modkit_security::{PolicyEngineRef, SecurityContext};

use crate::middleware;
use crate::router_cache::RouterCache;
use crate::web;

/// Main API Gateway module — owns the HTTP server (`rest_host`) and collects
/// typed operation specs to emit a single `OpenAPI` document.
#[modkit::module(
	name = "api_gateway",
	capabilities = [rest_host, rest, stateful],
    deps = ["grpc_hub"],
	lifecycle(entry = "serve", stop_timeout = "30s", await_ready)
)]
pub struct ApiGateway {
    // Lock-free config using arc-swap for read-mostly access
    pub(crate) config: ArcSwap<ApiGatewayConfig>,
    // OpenAPI registry for operations and schemas
    pub(crate) openapi_registry: Arc<OpenApiRegistryImpl>,
    // Built router cache for zero-lock hot path access
    pub(crate) router_cache: RouterCache<axum::Router>,
    // Store the finalized router from REST phase for serving
    pub(crate) final_router: Mutex<Option<axum::Router>>,

    // Duplicate detection (per (method, path) and per handler id)
    pub(crate) registered_routes: DashMap<(Method, String), ()>,
    pub(crate) registered_handlers: DashMap<String, ()>,
}

impl Default for ApiGateway {
    fn default() -> Self {
        let default_router = Router::new();
        Self {
            config: ArcSwap::from_pointee(ApiGatewayConfig::default()),
            openapi_registry: Arc::new(OpenApiRegistryImpl::new()),
            router_cache: RouterCache::new(default_router),
            final_router: Mutex::new(None),
            registered_routes: DashMap::new(),
            registered_handlers: DashMap::new(),
        }
    }
}

impl ApiGateway {
    /// Create a new `ApiGateway` instance with the given configuration
    #[must_use]
    pub fn new(config: ApiGatewayConfig) -> Self {
        let default_router = Router::new();
        Self {
            config: ArcSwap::from_pointee(config),
            openapi_registry: Arc::new(OpenApiRegistryImpl::new()),
            router_cache: RouterCache::new(default_router),
            final_router: Mutex::new(None),
            registered_routes: DashMap::new(),
            registered_handlers: DashMap::new(),
        }
    }

    /// Get the current configuration (cheap clone from `ArcSwap`)
    pub fn get_config(&self) -> ApiGatewayConfig {
        (**self.config.load()).clone()
    }

    /// Get cached configuration (lock-free with `ArcSwap`)
    pub fn get_cached_config(&self) -> ApiGatewayConfig {
        (**self.config.load()).clone()
    }

    /// Get the cached router without rebuilding (useful for performance-critical paths)
    pub fn get_cached_router(&self) -> Arc<Router> {
        self.router_cache.load()
    }

    /// Force rebuild and cache of the router.
    ///
    /// # Errors
    /// Returns an error if router building fails.
    pub fn rebuild_and_cache_router(&self) -> Result<()> {
        let new_router = self.build_router()?;
        self.router_cache.store(new_router);
        Ok(())
    }

    /// Build auth state and route policy from operation specs
    fn build_auth_state_from_specs(&self) -> Result<(auth::AuthState, auth::GatewayRoutePolicy)> {
        let mut req_map = std::collections::HashMap::new();
        let mut public_routes = std::collections::HashSet::new();

        // Always mark built-in health check routes as public
        public_routes.insert((Method::GET, "/health".to_owned()));
        public_routes.insert((Method::GET, "/healthz".to_owned()));
        public_routes.insert((Method::GET, "/docs".to_owned()));
        public_routes.insert((Method::GET, "/openapi.json".to_owned()));

        for spec in &self.openapi_registry.operation_specs {
            let spec = spec.value();
            let route_key = (spec.method.clone(), spec.path.clone());

            if let Some(ref sec) = spec.sec_requirement {
                req_map.insert(
                    route_key.clone(),
                    auth::Requirement {
                        resource: sec.resource.clone(),
                        action: sec.action.clone(),
                    },
                );
            }

            if spec.is_public {
                public_routes.insert(route_key);
            }
        }

        let config = self.get_cached_config();
        let requirements_count = req_map.len();
        let public_routes_count = public_routes.len();

        let (auth_state, route_policy) = auth::build_auth_state(&config, req_map, public_routes)?;

        tracing::info!(
            auth_disabled = config.auth_disabled,
            require_auth_by_default = config.require_auth_by_default,
            requirements_count = requirements_count,
            public_routes_count = public_routes_count,
            "Auth state and route policy built from operation specs"
        );

        Ok((auth_state, route_policy))
    }

    /// Apply license validation layer to router.
    fn apply_license_layer(
        router: Router,
        license_client: Option<middleware::license_validation::LicenseClient>,
        specs: &[modkit::api::OperationSpec],
    ) -> Router {
        let license_map = middleware::license_validation::LicenseRequirementMap::from_specs(specs);

        if license_client.is_some() {
            tracing::info!("License enforcer client found, enabling dynamic license validation");
        } else {
            tracing::debug!(
                "License enforcer client not registered, using stub behavior (BASE feature only)"
            );
        }

        let license_state = middleware::license_validation::LicenseValidationState {
            client: license_client,
            map: license_map,
        };

        router.layer(from_fn(
            move |req: axum::extract::Request, next: axum::middleware::Next| {
                let state = license_state.clone();
                middleware::license_validation::license_validation_middleware(state, req, next)
            },
        ))
    }

    /// Apply auth layer to router based on configuration.
    fn apply_auth_layer(
        router: Router,
        config: &ApiGatewayConfig,
        auth_state: &auth::AuthState,
        route_policy: auth::GatewayRoutePolicy,
    ) -> Router {
        if config.auth_disabled {
            let default_security_context = SecurityContext::builder()
                .tenant_id(DEFAULT_TENANT_ID)
                .subject_id(DEFAULT_SUBJECT_ID)
                .build();

            tracing::warn!(
                "API Gateway auth is DISABLED: all requests will run with default tenant SecurityCtx. \
                 This mode bypasses authentication and is intended ONLY for single-user on-premises deployments without an IdP. \
                 Permission checks and secure ORM still apply. DO NOT use this mode in multi-tenant or production environments."
            );

            router.layer(from_fn(
                move |mut req: axum::extract::Request, next: axum::middleware::Next| {
                    let sec_context = default_security_context.clone();
                    async move {
                        req.extensions_mut().insert(sec_context);
                        next.run(req).await
                    }
                },
            ))
        } else {
            let validator = auth_state.validator.clone();
            let authorizer = auth_state.authorizer.clone();
            let policy = Arc::new(route_policy) as Arc<dyn modkit_auth::RoutePolicy>;

            router.layer(modkit_auth::axum_ext::AuthPolicyLayer::new(
                validator, authorizer, policy,
            ))
        }
    }

    /// Apply trace layer with OpenTelemetry integration to router.
    fn apply_trace_layer(router: Router) -> Router {
        use modkit::http::otel;
        use tower_http::trace::TraceLayer;
        use tracing::field::Empty;

        router.layer(
            TraceLayer::new_for_http()
                .make_span_with(move |req: &axum::http::Request<axum::body::Body>| {
                    let hdr = middleware::request_id::header();
                    let rid = req
                        .headers()
                        .get(&hdr)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("n/a");

                    let span = tracing::info_span!(
                        "http_request",
                        method = %req.method(),
                        uri = %req.uri().path(),
                        version = ?req.version(),
                        module = "api_gateway",
                        endpoint = %req.uri().path(),
                        request_id = %rid,
                        status = Empty,
                        latency_ms = Empty,
                        "http.method" = %req.method(),
                        "http.target" = %req.uri().path(),
                        "http.scheme" = req.uri().scheme_str().unwrap_or("http"),
                        "http.host" = req.headers().get("host")
                            .and_then(|h| h.to_str().ok())
                            .unwrap_or("unknown"),
                        "user_agent.original" = req.headers().get("user-agent")
                            .and_then(|h| h.to_str().ok())
                            .unwrap_or("unknown"),
                        trace_id = Empty,
                        parent.trace_id = Empty
                    );

                    otel::set_parent_from_headers(&span, req.headers());
                    span
                })
                .on_response(
                    |res: &axum::http::Response<axum::body::Body>,
                     latency: std::time::Duration,
                     span: &tracing::Span| {
                        let ms = latency.as_millis();
                        span.record("status", res.status().as_u16());
                        span.record("latency_ms", ms);
                    },
                ),
        )
    }

    /// Apply all middleware layers to a router.
    ///
    /// This function applies the complete middleware stack in reverse order (innermost to outermost).
    /// At runtime, requests flow through the stack from outermost to innermost:
    /// `SetRequestId` → `PropagateRequestId` → Trace → `push_req_id_to_extensions` → Timeout →
    /// `BodyLimit` → CORS → MIME → `RateLimit` → `ErrorMapping` → Auth → `PolicyEngine` →
    /// `LicenseValidation` → Router.
    ///
    /// # Arguments
    ///
    /// * `router` - The router to apply middleware to
    /// * `license_client` - Optional license enforcer client for license validation.
    ///   When `None`, license validation falls back to stub behavior (BASE feature only).
    pub(crate) fn apply_middleware_stack(
        &self,
        mut router: Router,
        license_client: Option<middleware::license_validation::LicenseClient>,
    ) -> Result<Router> {
        let (auth_state, route_policy) = self.build_auth_state_from_specs()?;
        let config = self.get_cached_config();

        let specs: Vec<_> = self
            .openapi_registry
            .operation_specs
            .iter()
            .map(|e| e.value().clone())
            .collect();

        // Layer registration order (reverse of execution): innermost -> outermost
        // Runtime execution order (outermost -> innermost):
        //   1. SetRequestId         - Generate/extract request ID
        //   2. PropagateRequestId   - Propagate request ID to response headers
        //   3. Trace                - OpenTelemetry tracing and logging
        //   4. push_req_id_to_extensions - Record request ID in span and extensions
        //   5. Timeout              - Request timeout (30s gateway timeout)
        //   6. BodyLimit            - Request body size limit
        //   7. CORS                 - Cross-origin resource sharing (if enabled)
        //   8. MIME                 - MIME type validation for request bodies
        //   9. RateLimit            - Per-route rate limiting
        //  10. ErrorMapping         - Map errors to Problem+JSON responses
        //  11. Auth                 - Authentication and authorization (JWT validation, RBAC)
        //  12. PolicyEngine         - Inject policy engine into request extensions
        //  13. LicenseValidation    - Feature license validation per route
        //  14. Router               - Route to handler (innermost)

        // 13) License validation (registered first, runs late in pipeline)
        router = Self::apply_license_layer(router, license_client, &specs);

        // 12) Inject Policy Engine
        let policy_engine = auth_state.policy_engine.clone();
        router = router.layer(from_fn_with_state(
            policy_engine,
            |State(engine): State<PolicyEngineRef>,
             mut req: axum::extract::Request,
             next: axum::middleware::Next| async move {
                req.extensions_mut().insert(engine);
                next.run(req).await
            },
        ));

        // 11) Auth
        router = Self::apply_auth_layer(router, &config, &auth_state, route_policy);

        // 10) Error mapping
        router = router.layer(from_fn(modkit::api::error_layer::error_mapping_middleware));

        // 9) Per-route rate limiting
        let rate_map = middleware::rate_limit::RateLimiterMap::from_specs(&specs, &config)?;
        router = router.layer(from_fn(
            move |req: axum::extract::Request, next: axum::middleware::Next| {
                let map = rate_map.clone();
                middleware::rate_limit::rate_limit_middleware(map, req, next)
            },
        ));

        // 8) MIME type validation
        let mime_map = middleware::mime_validation::build_mime_validation_map(&specs);
        router = router.layer(from_fn(
            move |req: axum::extract::Request, next: axum::middleware::Next| {
                let map = mime_map.clone();
                middleware::mime_validation::mime_validation_middleware(map, req, next)
            },
        ));

        // 7) CORS
        if config.cors_enabled {
            router = router.layer(crate::cors::build_cors_layer(&config));
        }

        // 6) Body limit
        router = router.layer(RequestBodyLimitLayer::new(config.defaults.body_limit_bytes));
        router = router.layer(DefaultBodyLimit::max(config.defaults.body_limit_bytes));

        // 5) Timeout
        router = router.layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::GATEWAY_TIMEOUT,
            Duration::from_secs(30),
        ));

        // 4) Record request_id into span + extensions
        router = router.layer(from_fn(middleware::request_id::push_req_id_to_extensions));

        // 3) Trace
        router = Self::apply_trace_layer(router);

        // 2) PropagateRequestId (propagate request ID to response headers)
        let x_request_id = crate::middleware::request_id::header();
        router = router.layer(PropagateRequestIdLayer::new(x_request_id.clone()));

        // 1) SetRequestId (registered last, runs first - outermost layer)
        router = router.layer(SetRequestIdLayer::new(
            x_request_id,
            crate::middleware::request_id::MakeReqId,
        ));

        Ok(router)
    }

    /// Build the HTTP router from registered routes and operations.
    ///
    /// # Errors
    /// Returns an error if router building or middleware setup fails.
    pub fn build_router(&self) -> Result<Router> {
        // If the cached router is currently held elsewhere (e.g., by the running server),
        // return it without rebuilding to avoid unnecessary allocations.
        let cached_router = self.router_cache.load();
        if Arc::strong_count(&cached_router) > 1 {
            tracing::debug!("Using cached router");
            return Ok((*cached_router).clone());
        }

        tracing::debug!("Building new router (standalone/fallback mode)");
        // In standalone mode (no REST pipeline), register both health endpoints here.
        // In normal operation, rest_prepare() registers these instead.
        let mut router = Router::new()
            .route("/health", get(web::health_check))
            .route("/healthz", get(|| async { "ok" }));

        // Apply all middleware layers including auth, above the router
        // Note: In standalone mode, we don't have access to ClientHub, so license
        // validation falls back to stub behavior (BASE feature only)
        router = self.apply_middleware_stack(router, None)?;

        // Cache the built router for future use
        self.router_cache.store(router.clone());

        Ok(router)
    }

    /// Build `OpenAPI` specification from registered routes and components.
    ///
    /// # Errors
    /// Returns an error if `OpenAPI` specification building fails.
    pub fn build_openapi(&self) -> Result<utoipa::openapi::OpenApi> {
        let config = self.get_cached_config();
        let info = modkit::api::OpenApiInfo {
            title: config.openapi.title.clone(),
            version: config.openapi.version.clone(),
            description: config.openapi.description,
        };
        self.openapi_registry.build_openapi(&info)
    }

    /// Parse bind address from configuration string.
    fn parse_bind_address(bind_addr: &str) -> anyhow::Result<SocketAddr> {
        bind_addr
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid bind address '{bind_addr}': {e}"))
    }

    /// Get the finalized router or build a default one.
    fn get_or_build_router(self: &Arc<Self>) -> anyhow::Result<Router> {
        let stored = { self.final_router.lock().take() };

        if let Some(router) = stored {
            tracing::debug!("Using router from REST phase");
            Ok(router)
        } else {
            tracing::debug!("No router from REST phase, building default router");
            self.build_router()
        }
    }

    /// Background HTTP server: bind, notify ready, serve until cancelled.
    ///
    /// This method is the lifecycle entry-point generated by the macro
    /// (`#[modkit::module(..., lifecycle(...))]`).
    pub(crate) async fn serve(
        self: Arc<Self>,
        cancel: CancellationToken,
        ready: ReadySignal,
    ) -> anyhow::Result<()> {
        let cfg = self.get_cached_config();
        let addr = Self::parse_bind_address(&cfg.bind_addr)?;
        let router = self.get_or_build_router()?;

        // Bind the socket, only now consider the service "ready"
        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("HTTP server bound on {}", addr);
        ready.notify(); // Starting -> Running

        // Graceful shutdown on cancel
        let shutdown = {
            let cancel = cancel.clone();
            async move {
                cancel.cancelled().await;
                tracing::info!("HTTP server shutting down gracefully (cancellation)");
            }
        };

        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Check if `handler_id` is already registered (returns true if duplicate)
    fn check_duplicate_handler(&self, spec: &modkit::api::OperationSpec) -> bool {
        if self
            .registered_handlers
            .insert(spec.handler_id.clone(), ())
            .is_some()
        {
            tracing::error!(
                handler_id = %spec.handler_id,
                method = %spec.method.as_str(),
                path = %spec.path,
                "Duplicate handler_id detected; ignoring subsequent registration"
            );
            return true;
        }
        false
    }

    /// Check if route (method, path) is already registered (returns true if duplicate)
    fn check_duplicate_route(&self, spec: &modkit::api::OperationSpec) -> bool {
        let route_key = (spec.method.clone(), spec.path.clone());
        if self.registered_routes.insert(route_key, ()).is_some() {
            tracing::error!(
                method = %spec.method.as_str(),
                path = %spec.path,
                "Duplicate (method, path) detected; ignoring subsequent registration"
            );
            return true;
        }
        false
    }

    /// Log successful operation registration
    fn log_operation_registration(&self, spec: &modkit::api::OperationSpec) {
        let current_count = self.openapi_registry.operation_specs.len();
        tracing::debug!(
            handler_id = %spec.handler_id,
            method = %spec.method.as_str(),
            path = %spec.path,
            summary = %spec.summary.as_deref().unwrap_or("No summary"),
            total_operations = current_count,
            "Registered API operation"
        );
    }

    /// Add `OpenAPI` documentation routes to the router
    fn add_openapi_routes(&self, mut router: axum::Router) -> anyhow::Result<axum::Router> {
        // Build once, serve as static JSON (no per-request parsing)
        let op_count = self.openapi_registry.operation_specs.len();
        tracing::info!(
            "rest_finalize: emitting OpenAPI with {} operations",
            op_count
        );

        let openapi_doc = Arc::new(self.build_openapi()?);

        router = router
            .route(
                "/openapi.json",
                get({
                    use axum::{Json, http::header, response::IntoResponse};
                    let doc = openapi_doc;
                    move || async move {
                        ([(header::CACHE_CONTROL, "no-store")], Json(doc.as_ref())).into_response()
                    }
                }),
            )
            .route("/docs", get(web::serve_docs));

        #[cfg(feature = "embed_elements")]
        {
            router = router.route(
                "/docs/assets/{*file}",
                get(crate::assets::serve_elements_asset),
            );
        }

        Ok(router)
    }
}

// Manual implementation of Module trait with config loading
#[async_trait]
impl modkit::Module for ApiGateway {
    async fn init(&self, ctx: &modkit::context::ModuleCtx) -> anyhow::Result<()> {
        debug!("Module initialized with context");
        let cfg = ctx.config::<crate::config::ApiGatewayConfig>()?;
        self.config.store(Arc::new(cfg.clone()));

        debug!(
            "Effective api_gateway configuration:\n{:#?}",
            self.config.load()
        );

        if cfg.auth_disabled {
            tracing::info!(
                tenant_id = %DEFAULT_TENANT_ID,
                "Auth-disabled mode enabled with default tenant"
            );
        }

        Ok(())
    }
}

// REST host role: prepare/finalize the router, but do not start the server here.
impl modkit::contracts::ApiGatewayCapability for ApiGateway {
    fn rest_prepare(
        &self,
        _ctx: &modkit::context::ModuleCtx,
        router: axum::Router,
    ) -> anyhow::Result<axum::Router> {
        // Add health check endpoints:
        // - /health: detailed JSON response with status and timestamp
        // - /healthz: simple "ok" liveness probe (Kubernetes-style)
        let router = router
            .route("/health", get(web::health_check))
            .route("/healthz", get(|| async { "ok" }));

        // You may attach global middlewares here (trace, compression, cors), but do not start server.
        tracing::debug!("REST host prepared base router with health check endpoints");
        Ok(router)
    }

    fn rest_finalize(
        &self,
        ctx: &modkit::context::ModuleCtx,
        mut router: axum::Router,
    ) -> anyhow::Result<axum::Router> {
        let config = self.get_cached_config();

        if config.enable_docs {
            router = self.add_openapi_routes(router)?;
        }

        // Apply middleware stack (including auth) to the final router
        // Resolve license client at composition time (graceful degradation if not available)
        let license_client = ctx
            .client_hub()
            .get::<dyn LicenseEnforcerGatewayClient>()
            .ok();
        tracing::debug!("Applying middleware stack to finalized router");
        router = self.apply_middleware_stack(router, license_client)?;

        // Keep the finalized router to be used by `serve()`
        *self.final_router.lock() = Some(router.clone());

        tracing::info!("REST host finalized router with OpenAPI endpoints and auth middleware");
        Ok(router)
    }

    fn as_registry(&self) -> &dyn modkit::contracts::OpenApiRegistry {
        self
    }
}

impl modkit::contracts::RestApiCapability for ApiGateway {
    fn register_rest(
        &self,
        _ctx: &modkit::context::ModuleCtx,
        router: axum::Router,
        _openapi: &dyn modkit::contracts::OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        // This module acts as both rest_host and rest, but actual REST endpoints
        // are handled in the host methods above.
        Ok(router)
    }
}

impl OpenApiRegistry for ApiGateway {
    fn register_operation(&self, spec: &modkit::api::OperationSpec) {
        // Reject duplicates with "first wins" policy (second registration = programmer error).
        if self.check_duplicate_handler(spec) {
            return;
        }

        if self.check_duplicate_route(spec) {
            return;
        }

        // Delegate to the internal registry
        self.openapi_registry.register_operation(spec);
        self.log_operation_registration(spec);
    }

    fn ensure_schema_raw(
        &self,
        root_name: &str,
        schemas: Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) -> String {
        // Delegate to the internal registry
        self.openapi_registry.ensure_schema_raw(root_name, schemas)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_generation() {
        let mut config = ApiGatewayConfig::default();
        config.openapi.title = "Test API".to_owned();
        config.openapi.version = "1.0.0".to_owned();
        config.openapi.description = Some("Test Description".to_owned());
        let api = ApiGateway::new(config);

        // Test that we can build OpenAPI without any operations
        let doc = api.build_openapi().unwrap();
        let json = serde_json::to_value(&doc).unwrap();

        // Verify it's valid OpenAPI document structure
        assert!(json.get("openapi").is_some());
        assert!(json.get("info").is_some());
        assert!(json.get("paths").is_some());

        // Verify info section
        let info = json.get("info").unwrap();
        assert_eq!(info.get("title").unwrap(), "Test API");
        assert_eq!(info.get("version").unwrap(), "1.0.0");
        assert_eq!(info.get("description").unwrap(), "Test Description");
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod problem_openapi_tests {
    use super::*;
    use axum::Json;
    use modkit::api::{Missing, OperationBuilder};
    use serde_json::Value;

    async fn dummy_handler() -> Json<Value> {
        Json(serde_json::json!({"ok": true}))
    }

    #[tokio::test]
    async fn openapi_includes_problem_schema_and_response() {
        let api = ApiGateway::default();
        let router = axum::Router::new();

        // Build a route with a problem+json response
        let _router = OperationBuilder::<Missing, Missing, ()>::get("/tests/v1/problem-demo")
            .public()
            .summary("Problem demo")
            .problem_response(&api, http::StatusCode::BAD_REQUEST, "Bad Request") // <-- registers Problem + sets content type
            .handler(dummy_handler)
            .register(router, &api);

        let doc = api.build_openapi().expect("openapi");
        let v = serde_json::to_value(&doc).expect("json");

        // 1) Problem exists in components.schemas
        let problem = v
            .pointer("/components/schemas/Problem")
            .expect("Problem schema missing");
        assert!(
            problem.get("$ref").is_none(),
            "Problem must be a real object, not a self-ref"
        );

        // 2) Response under /paths/... references Problem and has correct media type
        let path_obj = v
            .pointer("/paths/~1tests~1v1~1problem-demo/get/responses/400")
            .expect("400 response missing");

        // Check what content types exist
        let content_obj = path_obj.get("content").expect("content object missing");
        assert!(
            content_obj.get("application/problem+json").is_some(),
            "application/problem+json content missing. Available content: {}",
            serde_json::to_string_pretty(content_obj).unwrap()
        );

        let content = path_obj
            .pointer("/content/application~1problem+json")
            .expect("application/problem+json content missing");
        // $ref to Problem
        let schema_ref = content
            .pointer("/schema/$ref")
            .and_then(|r| r.as_str())
            .unwrap_or("");
        assert_eq!(schema_ref, "#/components/schemas/Problem");
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod sse_openapi_tests {
    use super::*;
    use axum::Json;
    use modkit::api::{Missing, OperationBuilder};
    use serde_json::Value;

    #[derive(Clone)]
    #[modkit_macros::api_dto(request, response)]
    struct UserEvent {
        id: u32,
        message: String,
    }

    async fn sse_handler() -> axum::response::sse::Sse<
        impl futures_core::Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>,
    > {
        let b = modkit::SseBroadcaster::<UserEvent>::new(4);
        b.sse_response()
    }

    #[tokio::test]
    async fn openapi_has_sse_content() {
        let api = ApiGateway::default();
        let router = axum::Router::new();

        let _router = OperationBuilder::<Missing, Missing, ()>::get("/tests/v1/demo/sse")
            .summary("Demo SSE")
            .handler(sse_handler)
            .public()
            .sse_json::<UserEvent>(&api, "SSE of UserEvent")
            .register(router, &api);

        let doc = api.build_openapi().expect("openapi");
        let v = serde_json::to_value(&doc).expect("json");

        // schema is materialized
        let schema = v
            .pointer("/components/schemas/UserEvent")
            .expect("UserEvent missing");
        assert!(schema.get("$ref").is_none());

        // content is text/event-stream with $ref to our schema
        let refp = v
            .pointer("/paths/~1tests~1v1~1demo~1sse/get/responses/200/content/text~1event-stream/schema/$ref")
            .and_then(|x| x.as_str())
            .unwrap_or_default();
        assert_eq!(refp, "#/components/schemas/UserEvent");
    }

    #[tokio::test]
    async fn openapi_sse_additional_response() {
        async fn mixed_handler() -> Json<Value> {
            Json(serde_json::json!({"ok": true}))
        }

        let api = ApiGateway::default();
        let router = axum::Router::new();

        let _router = OperationBuilder::<Missing, Missing, ()>::get("/tests/v1/demo/mixed")
            .summary("Mixed responses")
            .public()
            .handler(mixed_handler)
            .json_response(http::StatusCode::OK, "Success response")
            .sse_json::<UserEvent>(&api, "Additional SSE stream")
            .register(router, &api);

        let doc = api.build_openapi().expect("openapi");
        let v = serde_json::to_value(&doc).expect("json");

        // Check that both response types are present
        let responses = v
            .pointer("/paths/~1tests~1v1~1demo~1mixed/get/responses")
            .expect("responses");

        // JSON response exists
        assert!(responses.get("200").is_some());

        // SSE response exists (could be another 200 or different status)
        let response_content = responses.get("200").and_then(|r| r.get("content"));
        assert!(response_content.is_some());

        // UserEvent schema is registered
        let schema = v
            .pointer("/components/schemas/UserEvent")
            .expect("UserEvent missing");
        assert!(schema.get("$ref").is_none());
    }

    #[tokio::test]
    async fn test_axum_to_openapi_path_conversion() {
        // Define a route with path parameters using Axum 0.8+ style {id}
        async fn user_handler() -> Json<Value> {
            Json(serde_json::json!({"user_id": "123"}))
        }

        let api = ApiGateway::default();
        let router = axum::Router::new();

        let _router = OperationBuilder::<Missing, Missing, ()>::get("/tests/v1/users/{id}")
            .summary("Get user by ID")
            .public()
            .path_param("id", "User ID")
            .handler(user_handler)
            .json_response(http::StatusCode::OK, "User details")
            .register(router, &api);

        // Verify the operation was stored with {id} path (same for Axum 0.8 and OpenAPI)
        let ops: Vec<_> = api
            .openapi_registry
            .operation_specs
            .iter()
            .map(|e| e.value().clone())
            .collect();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].path, "/tests/v1/users/{id}");

        // Verify OpenAPI doc also has {id} (no conversion needed for regular params)
        let doc = api.build_openapi().expect("openapi");
        let v = serde_json::to_value(&doc).expect("json");

        let paths = v.get("paths").expect("paths");
        assert!(
            paths.get("/tests/v1/users/{id}").is_some(),
            "OpenAPI should use {{id}} placeholder"
        );
    }

    #[tokio::test]
    async fn test_multiple_path_params_conversion() {
        async fn item_handler() -> Json<Value> {
            Json(serde_json::json!({"ok": true}))
        }

        let api = ApiGateway::default();
        let router = axum::Router::new();

        let _router = OperationBuilder::<Missing, Missing, ()>::get(
            "/tests/v1/projects/{project_id}/items/{item_id}",
        )
        .summary("Get project item")
        .public()
        .path_param("project_id", "Project ID")
        .path_param("item_id", "Item ID")
        .handler(item_handler)
        .json_response(http::StatusCode::OK, "Item details")
        .register(router, &api);

        // Verify storage and OpenAPI both use {param} syntax
        let ops: Vec<_> = api
            .openapi_registry
            .operation_specs
            .iter()
            .map(|e| e.value().clone())
            .collect();
        assert_eq!(
            ops[0].path,
            "/tests/v1/projects/{project_id}/items/{item_id}"
        );

        let doc = api.build_openapi().expect("openapi");
        let v = serde_json::to_value(&doc).expect("json");
        let paths = v.get("paths").expect("paths");
        assert!(
            paths
                .get("/tests/v1/projects/{project_id}/items/{item_id}")
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_wildcard_path_conversion() {
        async fn static_handler() -> Json<Value> {
            Json(serde_json::json!({"ok": true}))
        }

        let api = ApiGateway::default();
        let router = axum::Router::new();

        // Axum 0.8 uses {*path} for wildcards
        let _router = OperationBuilder::<Missing, Missing, ()>::get("/tests/v1/static/{*path}")
            .summary("Serve static files")
            .public()
            .handler(static_handler)
            .json_response(http::StatusCode::OK, "File content")
            .register(router, &api);

        // Verify internal storage keeps Axum wildcard syntax {*path}
        let ops: Vec<_> = api
            .openapi_registry
            .operation_specs
            .iter()
            .map(|e| e.value().clone())
            .collect();
        assert_eq!(ops[0].path, "/tests/v1/static/{*path}");

        // Verify OpenAPI converts wildcard to {path} (without asterisk)
        let doc = api.build_openapi().expect("openapi");
        let v = serde_json::to_value(&doc).expect("json");
        let paths = v.get("paths").expect("paths");
        assert!(
            paths.get("/tests/v1/static/{path}").is_some(),
            "Wildcard {{*path}} should be converted to {{path}} in OpenAPI"
        );
        assert!(
            paths.get("/static/{*path}").is_none(),
            "OpenAPI should not have Axum-style {{*path}}"
        );
    }

    #[tokio::test]
    async fn test_multipart_file_upload_openapi() {
        async fn upload_handler() -> Json<Value> {
            Json(serde_json::json!({"uploaded": true}))
        }

        let api = ApiGateway::default();
        let router = axum::Router::new();

        let _router = OperationBuilder::<Missing, Missing, ()>::post("/tests/v1/files/upload")
            .operation_id("upload_file")
            .public()
            .summary("Upload a file")
            .multipart_file_request("file", Some("File to upload"))
            .handler(upload_handler)
            .json_response(http::StatusCode::OK, "Upload successful")
            .register(router, &api);

        // Build OpenAPI and verify multipart schema
        let doc = api.build_openapi().expect("openapi");
        let v = serde_json::to_value(&doc).expect("json");

        let paths = v.get("paths").expect("paths");
        let upload_path = paths
            .get("/tests/v1/files/upload")
            .expect("/tests/v1/files/upload path");
        let post_op = upload_path.get("post").expect("POST operation");

        // Verify request body exists
        let request_body = post_op.get("requestBody").expect("requestBody");
        let content = request_body.get("content").expect("content");
        let multipart = content
            .get("multipart/form-data")
            .expect("multipart/form-data content type");

        // Verify schema structure
        let schema = multipart.get("schema").expect("schema");
        assert_eq!(
            schema.get("type").and_then(|v| v.as_str()),
            Some("object"),
            "Schema should be of type object"
        );

        // Verify properties
        let properties = schema.get("properties").expect("properties");
        let file_prop = properties.get("file").expect("file property");
        assert_eq!(
            file_prop.get("type").and_then(|v| v.as_str()),
            Some("string"),
            "File field should be of type string"
        );
        assert_eq!(
            file_prop.get("format").and_then(|v| v.as_str()),
            Some("binary"),
            "File field should have format binary"
        );

        // Verify required fields
        let required = schema.get("required").expect("required");
        let required_arr = required.as_array().expect("required should be array");
        assert_eq!(required_arr.len(), 1);
        assert_eq!(required_arr[0].as_str(), Some("file"));
    }
}
