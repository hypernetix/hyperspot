//! Domain service for OAGW.
//!
//! This service orchestrates all business logic for outbound API invocations,
//! route/link management, and plugin selection.

use std::sync::Arc;

use modkit::client_hub::ClientHub;
use modkit_odata::{ODataQuery, Page};
use modkit_security::{AccessScope, PolicyEngineRef, SecurityContext};
use oagw_sdk::{
    Link, LinkPatch, NewLink, NewRoute, OagwInvokeRequest, OagwInvokeResponse, OagwPluginApi,
    OagwResponseStream, Route, RoutePatch,
};
use tracing::{info_span, instrument, Instrument};
use uuid::Uuid;

use super::error::DomainError;
use super::ports::SecretResolver;
use super::repo::{LinkRepository, RouteRepository};
use crate::config::OagwConfig;

/// Service configuration extracted from module config.
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// Default connection timeout in milliseconds.
    pub default_connection_timeout_ms: u64,
    /// Default request timeout in milliseconds.
    pub default_request_timeout_ms: u64,
}

impl From<&OagwConfig> for ServiceConfig {
    fn from(cfg: &OagwConfig) -> Self {
        Self {
            default_connection_timeout_ms: cfg.default_connection_timeout_ms,
            default_request_timeout_ms: cfg.default_request_timeout_ms,
        }
    }
}

/// Domain service for OAGW operations.
pub struct Service {
    policy_engine: PolicyEngineRef,
    route_repo: Arc<dyn RouteRepository>,
    link_repo: Arc<dyn LinkRepository>,
    secret_resolver: Arc<dyn SecretResolver>,
    client_hub: Arc<ClientHub>,
    config: ServiceConfig,
}

impl Service {
    /// Create a new service instance.
    pub fn new(
        route_repo: Arc<dyn RouteRepository>,
        link_repo: Arc<dyn LinkRepository>,
        secret_resolver: Arc<dyn SecretResolver>,
        client_hub: Arc<ClientHub>,
        config: ServiceConfig,
    ) -> Self {
        Self {
            policy_engine: Arc::new(modkit_security::DummyPolicyEngine),
            route_repo,
            link_repo,
            secret_resolver,
            client_hub,
            config,
        }
    }

    /// Convert SecurityContext to AccessScope for repository operations.
    async fn prepare_scope(&self, ctx: &SecurityContext) -> Result<AccessScope, DomainError> {
        ctx.scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await
            .map_err(|e| DomainError::Authorization(e.to_string()))
    }

    // === Invocation Methods ===

    /// Invoke an outbound API with unary semantics.
    #[instrument(skip(self, ctx, req), fields(route_id = %req.route_id, method = %req.method))]
    pub async fn invoke_unary(
        &self,
        ctx: &SecurityContext,
        req: OagwInvokeRequest,
    ) -> Result<OagwInvokeResponse, DomainError> {
        let start = std::time::Instant::now();
        let scope = self.prepare_scope(ctx).await?;

        // Step 1.3: Route Resolution
        let route = self.resolve_route(&scope, req.route_id).await?;

        // Step 1.4: Link Selection
        let link = self.select_link(&scope, &route, req.link_id).await?;

        // TODO(v2): Step 1.5 - Rate Limit Check
        // TODO(v3): Check circuit breaker state

        // Step 1.6: Plugin Selection
        let plugin = self.select_plugin()?;

        // Step 1.7: Secret Resolution
        let secret = self
            .secret_resolver
            .get_secret(ctx, link.secret_ref)
            .await?;

        // Step 1.8: Authentication Preparation is handled by plugin

        // Step 1.9-1.11: Request Construction and Plugin Invocation
        let timeout_ms = req
            .timeout_ms
            .unwrap_or(self.config.default_request_timeout_ms);

        let invoke_req = OagwInvokeRequest {
            timeout_ms: Some(timeout_ms),
            ..req
        };

        let response = plugin
            .invoke_unary(ctx, &link, &route, &secret, invoke_req)
            .instrument(info_span!("plugin_invoke"))
            .await
            .map_err(|e| match e {
                oagw_sdk::OagwError::ConnectionTimeout => DomainError::ConnectionTimeout,
                oagw_sdk::OagwError::RequestTimeout => DomainError::RequestTimeout,
                oagw_sdk::OagwError::DownstreamError {
                    status_code,
                    retry_after_sec,
                } => DomainError::DownstreamError {
                    status_code,
                    retry_after_sec,
                },
                other => DomainError::Database(anyhow::anyhow!("{other}")),
            })?;

        // TODO(v3): Step 1.12 - Circuit Breaker Update
        // TODO(v2): Step 1.13 - Audit Logging
        // TODO(v2): Step 1.14 - Metrics Recording

        // Duration in ms is always small enough for u64 in practice
        #[allow(clippy::cast_possible_truncation)]
        let duration_ms = start.elapsed().as_millis() as u64;
        tracing::info!(
            duration_ms,
            status_code = response.status_code,
            link_id = %link.id,
            "Invocation completed"
        );

        Ok(response)
    }

    /// Invoke an outbound API with streaming response.
    #[instrument(skip(self, ctx, req), fields(route_id = %req.route_id, method = %req.method))]
    pub async fn invoke_stream(
        &self,
        ctx: &SecurityContext,
        req: OagwInvokeRequest,
    ) -> Result<OagwResponseStream, DomainError> {
        // TODO(v2): Implement streaming invocation
        // For v1, streaming is not supported
        tracing::warn!("Streaming invocation not yet implemented (v2)");

        let scope = self.prepare_scope(ctx).await?;

        // Resolve route and link for validation
        let route = self.resolve_route(&scope, req.route_id).await?;
        let link = self.select_link(&scope, &route, req.link_id).await?;
        let plugin = self.select_plugin()?;
        let secret = self
            .secret_resolver
            .get_secret(ctx, link.secret_ref)
            .await?;

        let timeout_ms = req
            .timeout_ms
            .unwrap_or(self.config.default_request_timeout_ms);

        let invoke_req = OagwInvokeRequest {
            timeout_ms: Some(timeout_ms),
            ..req
        };

        plugin
            .invoke_stream(ctx, &link, &route, &secret, invoke_req)
            .await
            .map_err(|e| DomainError::Database(anyhow::anyhow!("{e}")))
    }

    // === Route Resolution ===

    /// Resolve a route by ID with tenant scoping.
    async fn resolve_route(
        &self,
        scope: &AccessScope,
        route_id: Uuid,
    ) -> Result<Route, DomainError> {
        self.route_repo
            .find_by_id(scope, route_id)
            .await?
            .ok_or(DomainError::RouteNotFound { id: route_id })
    }

    // === Link Selection ===

    /// Select a link for invocation.
    ///
    /// If `link_id` is provided, use that specific link.
    /// Otherwise, select the best available link based on priority.
    async fn select_link(
        &self,
        scope: &AccessScope,
        route: &Route,
        link_id: Option<Uuid>,
    ) -> Result<Link, DomainError> {
        if let Some(id) = link_id {
            // Step 1.4a: Use specified link directly
            let link = self
                .link_repo
                .find_by_id(scope, id)
                .await?
                .ok_or(DomainError::LinkNotFound { id })?;

            // Verify link is enabled and belongs to the route
            if !link.enabled {
                return Err(DomainError::LinkNotFound { id });
            }
            if link.route_id != route.id {
                return Err(DomainError::Validation {
                    field: "link_id".to_string(),
                    message: format!("Link {id} does not belong to route {}", route.id),
                });
            }
            return Ok(link);
        }

        // Step 1.4b: Auto-select link
        let links = self
            .link_repo
            .find_enabled_by_route(scope, route.id)
            .await?;

        if links.is_empty() {
            return Err(DomainError::LinkUnavailable { route_id: route.id });
        }

        // TODO(v3): Filter out links with open circuit breakers
        // TODO(v4): Apply strategy-based selection (sticky session, round robin)

        // v1: Select first available link by priority (already sorted)
        let mut sorted_links = links;
        sorted_links.sort_by_key(|l| l.priority);

        sorted_links
            .into_iter()
            .next()
            .ok_or(DomainError::LinkUnavailable { route_id: route.id })
    }

    // === Plugin Selection ===

    /// Select a plugin that supports the route's protocol and auth type.
    ///
    /// TODO(v1): Implement full plugin discovery via types-registry.
    /// For now, returns the first registered plugin from ClientHub.
    fn select_plugin(&self) -> Result<Arc<dyn OagwPluginApi>, DomainError> {
        // TODO(v1): Query types-registry for plugin instances matching OagwPluginSpecV1 schema
        // TODO(v1): Filter plugins by supported_protocols and supported_auth_types
        // TODO(v1): Sort by priority and select best match
        //
        // For v1 skeleton, we attempt to get any registered OagwPluginApi from ClientHub.
        // This works when there's exactly one plugin registered (the default plugin).

        self.client_hub
            .get::<dyn OagwPluginApi>()
            .map_err(|e| DomainError::PluginNotFound {
                protocol: "any".to_string(),
                auth_type: format!("no plugin registered: {e}"),
            })
    }

    // === Route CRUD ===

    /// Create a new route.
    #[instrument(skip(self, ctx, new_route), fields(base_url = %new_route.base_url))]
    pub async fn create_route(
        &self,
        ctx: &SecurityContext,
        new_route: NewRoute,
    ) -> Result<Route, DomainError> {
        let scope = self.prepare_scope(ctx).await?;

        // Validate base URL
        if new_route.base_url.is_empty() {
            return Err(DomainError::validation("base_url", "cannot be empty"));
        }

        // Validate auth type GTS ID
        if new_route.auth_type_gts_id.is_empty() {
            return Err(DomainError::validation(
                "auth_type_gts_id",
                "cannot be empty",
            ));
        }

        // TODO(v1): Validate auth_type_gts_id exists in types_registry
        // TODO(v2): Validate protocol_gts_ids exist in types_registry

        let route = self.route_repo.insert(&scope, new_route).await?;

        tracing::info!(route_id = %route.id, "Route created");
        Ok(route)
    }

    /// Get a route by ID.
    pub async fn get_route(&self, ctx: &SecurityContext, id: Uuid) -> Result<Route, DomainError> {
        let scope = self.prepare_scope(ctx).await?;
        self.route_repo
            .find_by_id(&scope, id)
            .await?
            .ok_or(DomainError::RouteNotFound { id })
    }

    /// List routes with OData pagination.
    pub async fn list_routes(
        &self,
        ctx: &SecurityContext,
        query: ODataQuery,
    ) -> Result<Page<Route>, DomainError> {
        let scope = self.prepare_scope(ctx).await?;
        self.route_repo
            .list_page(&scope, query)
            .await
            .map_err(Into::into)
    }

    /// Update a route with partial data.
    #[instrument(skip(self, ctx, patch), fields(route_id = %id))]
    pub async fn update_route(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
        patch: RoutePatch,
    ) -> Result<Route, DomainError> {
        let scope = self.prepare_scope(ctx).await?;

        // Verify route exists
        if !self.route_repo.exists(&scope, id).await? {
            return Err(DomainError::RouteNotFound { id });
        }

        // Validate patch
        if let Some(ref base_url) = patch.base_url {
            if base_url.is_empty() {
                return Err(DomainError::validation("base_url", "cannot be empty"));
            }
        }

        let route = self.route_repo.update(&scope, id, patch).await?;

        tracing::info!(route_id = %route.id, "Route updated");
        Ok(route)
    }

    /// Delete a route by ID.
    #[instrument(skip(self, ctx), fields(route_id = %id))]
    pub async fn delete_route(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), DomainError> {
        let scope = self.prepare_scope(ctx).await?;
        let deleted = self.route_repo.delete(&scope, id).await?;
        if !deleted {
            return Err(DomainError::RouteNotFound { id });
        }

        tracing::info!(route_id = %id, "Route deleted");
        Ok(())
    }

    // === Link CRUD ===

    /// Create a new link.
    #[instrument(skip(self, ctx, new_link), fields(route_id = %new_link.route_id))]
    pub async fn create_link(
        &self,
        ctx: &SecurityContext,
        new_link: NewLink,
    ) -> Result<Link, DomainError> {
        let scope = self.prepare_scope(ctx).await?;

        // Validate route exists
        if !self.route_repo.exists(&scope, new_link.route_id).await? {
            return Err(DomainError::RouteNotFound {
                id: new_link.route_id,
            });
        }

        // Validate secret_ref
        // TODO(v2): Validate secret exists in cred_store

        // Validate strategy GTS ID
        if new_link.strategy_gts_id.is_empty() {
            return Err(DomainError::validation(
                "strategy_gts_id",
                "cannot be empty",
            ));
        }

        let link = self.link_repo.insert(&scope, new_link).await?;

        tracing::info!(link_id = %link.id, "Link created");
        Ok(link)
    }

    /// Get a link by ID.
    pub async fn get_link(&self, ctx: &SecurityContext, id: Uuid) -> Result<Link, DomainError> {
        let scope = self.prepare_scope(ctx).await?;
        self.link_repo
            .find_by_id(&scope, id)
            .await?
            .ok_or(DomainError::LinkNotFound { id })
    }

    /// List links with OData pagination.
    pub async fn list_links(
        &self,
        ctx: &SecurityContext,
        query: ODataQuery,
    ) -> Result<Page<Link>, DomainError> {
        let scope = self.prepare_scope(ctx).await?;
        self.link_repo
            .list_page(&scope, query)
            .await
            .map_err(Into::into)
    }

    /// Update a link with partial data.
    #[instrument(skip(self, ctx, patch), fields(link_id = %id))]
    pub async fn update_link(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
        patch: LinkPatch,
    ) -> Result<Link, DomainError> {
        let scope = self.prepare_scope(ctx).await?;

        // Verify link exists
        if !self.link_repo.exists(&scope, id).await? {
            return Err(DomainError::LinkNotFound { id });
        }

        let link = self.link_repo.update(&scope, id, patch).await?;

        tracing::info!(link_id = %link.id, "Link updated");
        Ok(link)
    }

    /// Delete a link by ID.
    #[instrument(skip(self, ctx), fields(link_id = %id))]
    pub async fn delete_link(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), DomainError> {
        let scope = self.prepare_scope(ctx).await?;
        let deleted = self.link_repo.delete(&scope, id).await?;
        if !deleted {
            return Err(DomainError::LinkNotFound { id });
        }

        tracing::info!(link_id = %id, "Link deleted");
        Ok(())
    }
}
