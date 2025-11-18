use async_trait::async_trait;
use std::sync::Arc;

use arc_swap::ArcSwap;
use dashmap::DashMap;

use anyhow::Result;
use axum::http::Method;
use axum::{middleware::from_fn, routing::get, Router};
use modkit::api::problem;
use modkit::api::OpenApiRegistry;
use modkit::lifecycle::ReadySignal;
use parking_lot::Mutex;
use std::net::SocketAddr;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tower_http::{
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    request_id::{PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
};
use utoipa::openapi::{
    content::ContentBuilder,
    info::InfoBuilder,
    path::{
        HttpMethod, OperationBuilder as UOperationBuilder, ParameterBuilder, ParameterIn,
        PathItemBuilder, PathsBuilder,
    },
    request_body::RequestBodyBuilder,
    response::{ResponseBuilder, ResponsesBuilder},
    schema::{ComponentsBuilder, ObjectBuilder, Schema, SchemaFormat, SchemaType},
    security::{HttpAuthScheme, HttpBuilder, SecurityRequirement, SecurityScheme},
    OpenApi, OpenApiBuilder, Ref, RefOr, Required,
};

mod assets;
mod auth;

mod config;
mod cors;
mod rate_limit;

pub mod error;
mod model;
pub mod request_id;
mod router_cache;
mod web;

pub use config::{ApiIngressConfig, CorsConfig};
use router_cache::RouterCache;

#[cfg(test)]
pub mod example_user_module;

use model::ComponentsRegistry;

/// Main API Ingress module — owns the HTTP server (rest_host) and collects
/// typed operation specs to emit a single OpenAPI document.
#[modkit::module(
	name = "api_ingress",
	capabilities = [rest_host, rest, stateful, system],
	lifecycle(entry = "serve", stop_timeout = "30s", await_ready)
)]
pub struct ApiIngress {
    // Lock-free config using arc-swap for read-mostly access
    config: ArcSwap<ApiIngressConfig>,
    // Lock-free components registry for read-mostly access
    components_registry: ArcSwap<ComponentsRegistry>,
    // Built router cache for zero-lock hot path access
    router_cache: RouterCache<axum::Router>,
    // Store the finalized router from REST phase for serving
    final_router: Mutex<Option<axum::Router>>,

    // Duplicate detection (per (method, path) and per handler id)
    registered_routes: DashMap<(Method, String), ()>,
    registered_handlers: DashMap<String, ()>,

    // Store operation specs for OpenAPI generation
    operation_specs: DashMap<String, modkit::api::OperationSpec>,
}

impl Default for ApiIngress {
    fn default() -> Self {
        let default_router = Router::new();
        Self {
            config: ArcSwap::from_pointee(ApiIngressConfig::default()),
            components_registry: ArcSwap::from_pointee(ComponentsRegistry::default()),
            router_cache: RouterCache::new(default_router),
            final_router: Mutex::new(None),
            registered_routes: DashMap::new(),
            registered_handlers: DashMap::new(),
            operation_specs: DashMap::new(),
        }
    }
}

impl ApiIngress {
    /// Create a new ApiIngress instance with the given configuration
    pub fn new(config: ApiIngressConfig) -> Self {
        let default_router = Router::new();
        Self {
            config: ArcSwap::from_pointee(config),
            router_cache: RouterCache::new(default_router),
            final_router: Mutex::new(None),
            operation_specs: DashMap::new(),
            ..Default::default()
        }
    }

    /// Get the current configuration (cheap clone from ArcSwap)
    pub fn get_config(&self) -> ApiIngressConfig {
        (**self.config.load()).clone()
    }

    /// Get cached configuration (lock-free with ArcSwap)
    pub fn get_cached_config(&self) -> ApiIngressConfig {
        (**self.config.load()).clone()
    }

    /// Get the cached router without rebuilding (useful for performance-critical paths)
    pub fn get_cached_router(&self) -> Arc<Router> {
        self.router_cache.load()
    }

    /// Force rebuild and cache of the router
    pub async fn rebuild_and_cache_router(&self) -> Result<()> {
        let new_router = self.build_router().await?;
        self.router_cache.store(new_router);
        Ok(())
    }

    /// Build auth state and route policy from operation specs
    fn build_auth_state_from_specs(&self) -> Result<(auth::AuthState, auth::IngressRoutePolicy)> {
        let mut req_map = std::collections::HashMap::new();
        let mut public_routes = std::collections::HashSet::new();

        // Always mark built-in health check routes as public
        public_routes.insert((Method::GET, "/health".to_string()));
        public_routes.insert((Method::GET, "/healthz".to_string()));
        public_routes.insert((Method::GET, "/docs".to_string()));
        public_routes.insert((Method::GET, "/openapi.json".to_string()));

        for spec in self.operation_specs.iter() {
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

    /// Apply all middleware layers to a router (request ID, tracing, timeout, body limit, CORS, rate limiting, error mapping, auth)
    fn apply_middleware_stack(&self, mut router: Router) -> Result<Router> {
        // Build auth state and route policy once
        let (auth_state, route_policy) = self.build_auth_state_from_specs()?;

        // Correct middleware order (outermost to innermost):
        // RequestId(Propagate -> Set) -> Trace -> push_req_id_to_extensions -> Timeout -> BodyLimit -> CORS -> RateLimit -> ErrorMapping -> Auth -> Router
        // Note: CORS must short-circuit OPTIONS before Auth/limits; tower-http does this when the layer is present above them.
        let x_request_id = crate::request_id::header();

        // 1. If client sent x-request-id, propagate it; otherwise we will set it
        router = router.layer(PropagateRequestIdLayer::new(x_request_id.clone()));

        // 2. Generate x-request-id when missing
        router = router.layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            crate::request_id::MakeReqId,
        ));

        // 3. Create the http span with request_id/status/latency
        router = router.layer({
            use crate::request_id::header;
            use modkit::http::otel;
            use tower_http::trace::TraceLayer;
            use tracing::field::Empty;

            TraceLayer::new_for_http()
                .make_span_with(move |req: &axum::http::Request<axum::body::Body>| {
                    let hdr = header();
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
                        module = "api_ingress",
                        endpoint = %req.uri().path(),
                        request_id = %rid,
                        status = Empty,
                        latency_ms = Empty,
                        // OpenTelemetry semantic conventions
                        "http.method" = %req.method(),
                        "http.target" = %req.uri().path(),
                        "http.scheme" = req.uri().scheme_str().unwrap_or("http"),
                        "http.host" = req.headers().get("host")
                            .and_then(|h| h.to_str().ok())
                            .unwrap_or("unknown"),
                        "user_agent.original" = req.headers().get("user-agent")
                            .and_then(|h| h.to_str().ok())
                            .unwrap_or("unknown"),
                        // Trace context placeholders (for log correlation)
                        trace_id = Empty,
                        parent.trace_id = Empty
                    );

                    // Set parent OTel trace context (W3C traceparent), if any
                    // This also populates trace_id and parent.trace_id from headers
                    otel::set_parent_from_headers(&span, req.headers());

                    span
                })
                .on_response(
                    |res: &axum::http::Response<axum::body::Body>,
                     latency: std::time::Duration,
                     span: &tracing::Span| {
                        let ms = (latency.as_secs_f64() * 1000.0) as u64;
                        span.record("status", res.status().as_u16());
                        span.record("latency_ms", ms);
                    },
                )
        });

        // 4. Record request_id into span + extensions (span must exist first)
        router = router.layer(from_fn(crate::request_id::push_req_id_to_extensions));

        // 5. Timeout layer - 30 second timeout for handlers
        router = router.layer(TimeoutLayer::new(Duration::from_secs(30)));

        // 6. Body limit layer - from config default
        let config = self.get_cached_config();
        router = router.layer(RequestBodyLimitLayer::new(config.defaults.body_limit_bytes));

        // 7. CORS layer (if enabled). Place after BodyLimit so preflight returns early.
        if config.cors_enabled {
            if let Some(layer) = crate::cors::build_cors_layer(&config) {
                router = router.layer(layer);
            } else {
                router = router.layer(CorsLayer::permissive());
            }
        }

        // 8. Per-route rate limiting & in-flight limits (after CORS, before auth)
        let specs: Vec<_> = self
            .operation_specs
            .iter()
            .map(|e| e.value().clone())
            .collect();
        let rate_map = crate::rate_limit::RateLimiterMap::from_specs(&specs, &config);
        router = router.layer(from_fn(
            move |req: axum::extract::Request, next: axum::middleware::Next| {
                let map = rate_map.clone();
                crate::rate_limit::rate_limit_middleware(map, req, next)
            },
        ));

        // 9. Error mapping layer (no-op converter for now; keeps order explicit)
        router = router.layer(from_fn(modkit::api::error_layer::error_mapping_middleware));

        // 10. Auth middleware - MUST be after CORS; preflight short-circuits before this.
        let config = self.get_cached_config();
        if config.auth_disabled {
            tracing::warn!(
                "API Ingress auth is DISABLED: all requests will run with root SecurityCtx (SecurityCtx::root_ctx()). \
                 This mode bypasses authentication and is intended ONLY for single-user on-premises deployments without an IdP. \
                 Permission checks and secure ORM still apply. DO NOT use this mode in multi-tenant or production environments."
            );
            router = router.layer(from_fn(
                |mut req: axum::extract::Request, next: axum::middleware::Next| async move {
                    let sec = modkit_security::SecurityCtx::root_ctx();
                    req.extensions_mut().insert(sec);
                    next.run(req).await
                },
            ));
        } else {
            let validator = auth_state.validator.clone();
            let scope_builder = auth_state.scope_builder.clone();
            let authorizer = auth_state.authorizer.clone();
            let policy = Arc::new(route_policy) as Arc<dyn modkit_auth::RoutePolicy>;

            router = router.layer(from_fn(
                move |req: axum::extract::Request, next: axum::middleware::Next| {
                    let validator = validator.clone();
                    let scope_builder = scope_builder.clone();
                    let authorizer = authorizer.clone();
                    let policy = policy.clone();
                    async move {
                        modkit_auth::axum_ext::auth_with_policy(
                            axum::extract::State(validator),
                            axum::extract::State(scope_builder),
                            axum::extract::State(authorizer),
                            axum::extract::State(policy),
                            req,
                            next,
                        )
                        .await
                    }
                },
            ));
        }

        Ok(router)
    }

    /// Build the HTTP router from registered routes and operations
    pub async fn build_router(&self) -> Result<Router> {
        // If the cached router is currently held elsewhere (e.g., by the running server),
        // return it without rebuilding to avoid unnecessary allocations.
        let cached_router = self.router_cache.load();
        if Arc::strong_count(&cached_router) > 1 {
            tracing::debug!("Using cached router");
            return Ok((*cached_router).clone());
        }

        tracing::debug!("Building new router");
        let mut router = Router::new().route("/health", get(web::health_check));

        // Apply all middleware layers including auth, above the router
        router = self.apply_middleware_stack(router)?;

        // Cache the built router for future use
        self.router_cache.store(router.clone());

        Ok(router)
    }

    /// Build OpenAPI specification from registered routes and components using utoipa.
    pub fn build_openapi(&self) -> Result<OpenApi> {
        // Log operation count for visibility
        let op_count = self.operation_specs.len();
        tracing::info!("Building OpenAPI: found {op_count} registered operations");

        // 1) Paths
        let mut paths = PathsBuilder::new();

        for spec in self.operation_specs.iter().map(|e| e.value().clone()) {
            let mut op = UOperationBuilder::new()
                .operation_id(spec.operation_id.clone().or(Some(spec.handler_id.clone())))
                .summary(spec.summary.clone())
                .description(spec.description.clone());

            for tag in &spec.tags {
                op = op.tag(tag.clone());
            }

            // Vendor extensions for rate limit, if present (string values)
            if let Some(rl) = spec.rate_limit.as_ref() {
                let mut ext = utoipa::openapi::extensions::Extensions::default();
                ext.insert("x-rate-limit-rps".to_string(), serde_json::json!(rl.rps));
                ext.insert(
                    "x-rate-limit-burst".to_string(),
                    serde_json::json!(rl.burst),
                );
                ext.insert(
                    "x-in-flight-limit".to_string(),
                    serde_json::json!(rl.in_flight),
                );
                op = op.extensions(Some(ext));
            }

            // Parameters
            for p in &spec.params {
                let in_ = match p.location {
                    modkit::api::ParamLocation::Path => ParameterIn::Path,
                    modkit::api::ParamLocation::Query => ParameterIn::Query,
                    modkit::api::ParamLocation::Header => ParameterIn::Header,
                    modkit::api::ParamLocation::Cookie => ParameterIn::Cookie,
                };
                let required =
                    if matches!(p.location, modkit::api::ParamLocation::Path) || p.required {
                        Required::True
                    } else {
                        Required::False
                    };

                let schema_type = match p.param_type.as_str() {
                    "integer" => SchemaType::Type(utoipa::openapi::schema::Type::Integer),
                    "number" => SchemaType::Type(utoipa::openapi::schema::Type::Number),
                    "boolean" => SchemaType::Type(utoipa::openapi::schema::Type::Boolean),
                    _ => SchemaType::Type(utoipa::openapi::schema::Type::String),
                };
                let schema = Schema::Object(ObjectBuilder::new().schema_type(schema_type).build());

                let param = ParameterBuilder::new()
                    .name(&p.name)
                    .parameter_in(in_)
                    .required(required)
                    .description(p.description.clone())
                    .schema(Some(schema))
                    .build();

                op = op.parameter(param);
            }

            // Request body
            if let Some(rb) = &spec.request_body {
                let content = if let Some(name) = &rb.schema_name {
                    ContentBuilder::new()
                        .schema(Some(RefOr::Ref(Ref::from_schema_name(name.clone()))))
                        .build()
                } else {
                    ContentBuilder::new()
                        .schema(Some(Schema::Object(ObjectBuilder::new().build())))
                        .build()
                };
                let mut rbld = RequestBodyBuilder::new()
                    .description(rb.description.clone())
                    .content(rb.content_type.to_string(), content);
                if rb.required {
                    rbld = rbld.required(Some(Required::True));
                }
                op = op.request_body(Some(rbld.build()));
            }

            // Responses
            let mut responses = ResponsesBuilder::new();
            for r in &spec.responses {
                let is_json_like = r.content_type == "application/json"
                    || r.content_type == problem::APPLICATION_PROBLEM_JSON
                    || r.content_type == "text/event-stream";
                let resp = if is_json_like {
                    if let Some(name) = &r.schema_name {
                        // Manually build content to preserve the correct content type
                        let content = ContentBuilder::new()
                            .schema(Some(RefOr::Ref(Ref::new(format!(
                                "#/components/schemas/{}",
                                name
                            )))))
                            .build();
                        ResponseBuilder::new()
                            .description(&r.description)
                            .content(r.content_type, content)
                            .build()
                    } else {
                        let content = ContentBuilder::new()
                            .schema(Some(Schema::Object(ObjectBuilder::new().build())))
                            .build();
                        ResponseBuilder::new()
                            .description(&r.description)
                            .content(r.content_type, content)
                            .build()
                    }
                } else {
                    let schema = Schema::Object(
                        ObjectBuilder::new()
                            .schema_type(SchemaType::Type(utoipa::openapi::schema::Type::String))
                            .format(Some(SchemaFormat::Custom(r.content_type.into())))
                            .build(),
                    );
                    let content = ContentBuilder::new().schema(Some(schema)).build();
                    ResponseBuilder::new()
                        .description(&r.description)
                        .content(r.content_type, content)
                        .build()
                };
                responses = responses.response(r.status.to_string(), resp);
            }
            op = op.responses(responses.build());

            // Add security requirement if operation has explicit auth metadata
            if spec.sec_requirement.is_some() {
                let sec_req = SecurityRequirement::new("bearerAuth", Vec::<String>::new());
                op = op.security(sec_req);
            }

            let method = match spec.method {
                Method::GET => HttpMethod::Get,
                Method::POST => HttpMethod::Post,
                Method::PUT => HttpMethod::Put,
                Method::DELETE => HttpMethod::Delete,
                Method::PATCH => HttpMethod::Patch,
                _ => HttpMethod::Get,
            };

            let item = PathItemBuilder::new().operation(method, op.build()).build();
            paths = paths.path(spec.path.clone(), item);
        }

        // 2) Components (from our registry)
        let mut components = ComponentsBuilder::new();
        for (name, schema) in self.components_registry.load().iter() {
            components = components.schema(name.clone(), schema.clone());
        }

        // Add bearer auth security scheme
        components = components.security_scheme(
            "bearerAuth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );

        // 3) Info & final OpenAPI doc
        let info = InfoBuilder::new()
            .title("HyperSpot API")
            .version("0.1.0")
            .description(Some("HyperSpot Server API Documentation"))
            .build();

        let openapi = OpenApiBuilder::new()
            .info(info)
            .paths(paths.build())
            .components(Some(components.build()))
            .build();

        Ok(openapi)
    }

    /// Background HTTP server: bind, notify ready, serve until cancelled.
    ///
    /// This method is the lifecycle entry-point generated by the macro
    /// (`#[modkit::module(..., lifecycle(...))]`).
    async fn serve(
        self: Arc<Self>,
        cancel: CancellationToken,
        ready: ReadySignal,
    ) -> anyhow::Result<()> {
        let cfg = self.get_cached_config();
        let addr: SocketAddr = cfg
            .bind_addr
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid bind address '{}': {}", cfg.bind_addr, e))?;

        // Take the finalized router so the MutexGuard is dropped before awaits
        let stored = { self.final_router.lock().take() };
        let router = if let Some(r) = stored {
            tracing::debug!("Using router from REST phase");
            r
        } else {
            tracing::debug!("No router from REST phase, building default router");
            self.build_router().await?
        };

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
}

// Manual implementation of Module trait with config loading
#[async_trait]
impl modkit::Module for ApiIngress {
    async fn init(&self, ctx: &modkit::context::ModuleCtx) -> anyhow::Result<()> {
        tracing::debug!(module = "api_ingress", "Module initialized with context");
        let cfg = ctx.config::<crate::config::ApiIngressConfig>()?;
        self.config.store(Arc::new(cfg));
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Test that the module is properly registered via inventory
#[cfg(test)]
mod tests {
    use super::*;
    use modkit::ModuleRegistry;

    #[test]
    fn test_module_registration() {
        // Ensure the module is discoverable via inventory
        let registry = ModuleRegistry::discover_and_build().expect("Failed to build registry");
        let module = registry.modules().iter().find(|m| m.name == "api_ingress");
        assert!(
            module.is_some(),
            "api_ingress module should be registered via inventory"
        );
    }

    #[test]
    fn test_module_capabilities() {
        let registry = ModuleRegistry::discover_and_build().expect("Failed to build registry");
        let module = registry
            .modules()
            .iter()
            .find(|m| m.name == "api_ingress")
            .expect("api_ingress should be registered");

        // Verify module properties
        assert_eq!(module.name, "api_ingress");

        // Downcast to verify the actual type behind the module
        if let Some(_api_module) = module.core.as_any().downcast_ref::<ApiIngress>() {
            // With lifecycle(...) on the type, stateful capability is provided via WithLifecycle
            assert!(
                module.stateful.is_some(),
                "Module should have stateful capability"
            );
        } else {
            panic!("Failed to downcast to ApiIngress - module not registered correctly");
        }
    }

    #[test]
    fn test_openapi_generation() {
        let api = ApiIngress::default();

        // Test that we can build OpenAPI without any operations
        let doc = api.build_openapi().unwrap();
        let json = serde_json::to_value(&doc).unwrap();

        // Verify it's valid OpenAPI document structure
        assert!(json.get("openapi").is_some());
        assert!(json.get("info").is_some());
        assert!(json.get("paths").is_some());

        // Verify info section
        let info = json.get("info").unwrap();
        assert_eq!(info.get("title").unwrap(), "HyperSpot API");
        assert_eq!(info.get("version").unwrap(), "0.1.0");
    }
}

// REST host role: prepare/finalize the router, but do not start the server here.
impl modkit::contracts::RestHostModule for ApiIngress {
    fn rest_prepare(
        &self,
        _ctx: &modkit::context::ModuleCtx,
        router: axum::Router,
    ) -> anyhow::Result<axum::Router> {
        // Add basic health check endpoint and any global middlewares
        let router = router.route("/healthz", get(|| async { "ok" }));

        // You may attach global middlewares here (trace, compression, cors), but do not start server.
        tracing::debug!("REST host prepared base router with health check");
        Ok(router)
    }

    fn rest_finalize(
        &self,
        _ctx: &modkit::context::ModuleCtx,
        mut router: axum::Router,
    ) -> anyhow::Result<axum::Router> {
        let config = self.get_cached_config();

        if config.enable_docs {
            // Build once, serve as static JSON (no per-request parsing)
            let op_count = self.operation_specs.len();
            tracing::info!(
                "rest_finalize: emitting OpenAPI with {} operations",
                op_count
            );

            let openapi_doc = Arc::new(self.build_openapi()?);

            router = router
                .route(
                    "/openapi.json",
                    get({
                        use axum::{http::header, response::IntoResponse, Json};
                        let doc = openapi_doc.clone();
                        move || async move {
                            ([(header::CACHE_CONTROL, "no-store")], Json(doc.as_ref()))
                                .into_response()
                        }
                    }),
                )
                .route("/docs", get(web::serve_docs));

            #[cfg(feature = "embed_elements")]
            {
                router = router.route("/docs/assets/{*file}", get(assets::serve_elements_asset));
            }
        }

        // Apply middleware stack (including auth) to the final router
        tracing::debug!("Applying middleware stack to finalized router");
        router = self.apply_middleware_stack(router)?;

        // Keep the finalized router to be used by `serve()`
        *self.final_router.lock() = Some(router.clone());

        tracing::info!("REST host finalized router with OpenAPI endpoints and auth middleware");
        Ok(router)
    }

    fn as_registry(&self) -> &dyn modkit::contracts::OpenApiRegistry {
        self
    }
}

impl modkit::contracts::RestfulModule for ApiIngress {
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

impl OpenApiRegistry for ApiIngress {
    fn register_operation(&self, spec: &modkit::api::OperationSpec) {
        // Reject duplicates with "first wins" policy (second registration = programmer error).
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
            return;
        }

        let route_key = (spec.method.clone(), spec.path.clone());
        if self.registered_routes.insert(route_key, ()).is_some() {
            tracing::error!(
                method = %spec.method.as_str(),
                path = %spec.path,
                "Duplicate (method, path) detected; ignoring subsequent registration"
            );
            return;
        }

        let operation_key = format!("{}:{}", spec.method.as_str(), spec.path);
        self.operation_specs
            .insert(operation_key.clone(), spec.clone());

        let current_count = self.operation_specs.len();
        tracing::debug!(
            handler_id = %spec.handler_id,
            method = %spec.method.as_str(),
            path = %spec.path,
            summary = %spec.summary.as_deref().unwrap_or("No summary"),
            operation_key = %operation_key,
            total_operations = current_count,
            "Registered API operation"
        );
    }

    fn ensure_schema_raw(&self, root_name: &str, schemas: Vec<(String, RefOr<Schema>)>) -> String {
        // Snapshot & copy-on-write
        let current = self.components_registry.load();
        let mut reg = (**current).clone();

        for (name, schema) in schemas {
            // Conflict policy: identical → no-op; different → warn & override
            if let Some(existing) = reg.get(&name) {
                let a = serde_json::to_value(existing).ok();
                let b = serde_json::to_value(&schema).ok();
                if a == b {
                    continue; // Skip identical schemas
                } else {
                    tracing::warn!(%name, "Schema content conflict; overriding with latest");
                }
            }
            reg.insert_schema(name, schema);
        }

        self.components_registry.store(Arc::new(reg));
        root_name.to_string()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
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
        let api = ApiIngress::default();
        let router = axum::Router::new();

        // Build a route with a problem+json response
        let _router = OperationBuilder::<Missing, Missing, ()>::get("/problem-demo")
            .summary("Problem demo")
            .problem_response(&api, 400, "Bad Request") // <-- registers Problem + sets content type
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
            .pointer("/paths/~1problem-demo/get/responses/400")
            .expect("400 response missing");

        // Check what content types exist
        let content_obj = path_obj.get("content").expect("content object missing");
        if content_obj.get("application/problem+json").is_none() {
            // Print available content types for debugging
            panic!(
                "application/problem+json content missing. Available content: {}",
                serde_json::to_string_pretty(content_obj).unwrap()
            );
        }

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
mod sse_openapi_tests {
    use super::*;
    use axum::Json;
    use modkit::api::{Missing, OperationBuilder};
    use serde_json::Value;

    #[derive(Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    struct UserEvent {
        id: u32,
        message: String,
    }

    async fn sse_handler() -> axum::response::sse::Sse<
        impl futures::Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>,
    > {
        let b = modkit::SseBroadcaster::<UserEvent>::new(4);
        b.sse_response()
    }

    #[tokio::test]
    async fn openapi_has_sse_content() {
        let api = ApiIngress::default();
        let router = axum::Router::new();

        let _router = OperationBuilder::<Missing, Missing, ()>::get("/demo/sse")
            .summary("Demo SSE")
            .handler(sse_handler)
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
            .pointer("/paths/~1demo~1sse/get/responses/200/content/text~1event-stream/schema/$ref")
            .and_then(|x| x.as_str())
            .unwrap_or_default();
        assert_eq!(refp, "#/components/schemas/UserEvent");
    }

    #[tokio::test]
    async fn openapi_sse_additional_response() {
        let api = ApiIngress::default();
        let router = axum::Router::new();

        async fn mixed_handler() -> Json<Value> {
            Json(serde_json::json!({"ok": true}))
        }

        let _router = OperationBuilder::<Missing, Missing, ()>::get("/demo/mixed")
            .summary("Mixed responses")
            .handler(mixed_handler)
            .json_response(200, "Success response")
            .sse_json::<UserEvent>(&api, "Additional SSE stream")
            .register(router, &api);

        let doc = api.build_openapi().expect("openapi");
        let v = serde_json::to_value(&doc).expect("json");

        // Check that both response types are present
        let responses = v
            .pointer("/paths/~1demo~1mixed/get/responses")
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
}
