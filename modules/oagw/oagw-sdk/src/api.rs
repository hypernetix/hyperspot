//! OAGW API traits.
//!
//! This module defines both the public gateway API and the internal plugin API.

use async_trait::async_trait;
use modkit_odata::{ODataQuery, Page};
use modkit_security::SecurityContext;
use uuid::Uuid;

use crate::error::OagwError;
use crate::models::{
    Link, LinkPatch, NewLink, NewRoute, OagwInvokeRequest, OagwInvokeResponse, OagwResponseStream,
    Route, RoutePatch, Secret,
};

/// Public API trait for the OAGW gateway.
///
/// This trait is exposed by the gateway to other modules via `ClientHub`:
/// ```ignore
/// let client = hub.get::<dyn OagwApi>()?;
/// let response = client.invoke_unary(&ctx, request).await?;
/// ```
///
/// All methods require a `SecurityContext` for proper authorization and tenant isolation.
#[async_trait]
pub trait OagwApi: Send + Sync {
    // === Invocation Methods ===

    /// Invoke an outbound API with unary (request-response) semantics.
    ///
    /// # Arguments
    /// * `ctx` - Security context for authorization
    /// * `req` - Invocation request with route, method, path, etc.
    ///
    /// # Returns
    /// * `Ok(OagwInvokeResponse)` - Response from downstream API
    /// * `Err(OagwError)` - Error if invocation fails
    async fn invoke_unary(
        &self,
        ctx: &SecurityContext,
        req: OagwInvokeRequest,
    ) -> Result<OagwInvokeResponse, OagwError>;

    /// Invoke an outbound API with streaming response (e.g., SSE).
    ///
    /// # Arguments
    /// * `ctx` - Security context for authorization
    /// * `req` - Invocation request with route, method, path, etc.
    ///
    /// # Returns
    /// * `Ok(OagwResponseStream)` - Stream of response chunks
    /// * `Err(OagwError)` - Error if stream cannot be established
    ///
    /// # Note
    /// Streaming requests are never retried by OAGW. If the stream fails mid-way,
    /// it yields a terminal `OagwStreamAbort` with resume hints if applicable.
    async fn invoke_stream(
        &self,
        ctx: &SecurityContext,
        req: OagwInvokeRequest,
    ) -> Result<OagwResponseStream, OagwError>;

    // === Route CRUD ===

    /// Create a new outbound API route.
    async fn create_route(
        &self,
        ctx: &SecurityContext,
        new_route: NewRoute,
    ) -> Result<Route, OagwError>;

    /// Get a route by ID.
    async fn get_route(&self, ctx: &SecurityContext, id: Uuid) -> Result<Route, OagwError>;

    /// List routes with `OData` pagination.
    async fn list_routes(
        &self,
        ctx: &SecurityContext,
        query: ODataQuery,
    ) -> Result<Page<Route>, OagwError>;

    /// Update a route with partial data.
    async fn update_route(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
        patch: RoutePatch,
    ) -> Result<Route, OagwError>;

    /// Delete a route by ID.
    async fn delete_route(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), OagwError>;

    // === Link CRUD ===

    /// Create a new outbound API link.
    async fn create_link(
        &self,
        ctx: &SecurityContext,
        new_link: NewLink,
    ) -> Result<Link, OagwError>;

    /// Get a link by ID.
    async fn get_link(&self, ctx: &SecurityContext, id: Uuid) -> Result<Link, OagwError>;

    /// List links with `OData` pagination.
    async fn list_links(
        &self,
        ctx: &SecurityContext,
        query: ODataQuery,
    ) -> Result<Page<Link>, OagwError>;

    /// Update a link with partial data.
    async fn update_link(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
        patch: LinkPatch,
    ) -> Result<Link, OagwError>;

    /// Delete a link by ID.
    async fn delete_link(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), OagwError>;
}

/// Plugin API trait implemented by OAGW plugins.
///
/// Plugins register themselves with a scoped client in `ClientHub`:
/// ```ignore
/// let scope = ClientScope::gts_id(&instance_id);
/// hub.register_scoped::<dyn OagwPluginApi>(scope, plugin_impl);
/// ```
///
/// The gateway resolves plugins at runtime based on protocol and auth type requirements.
#[async_trait]
pub trait OagwPluginApi: Send + Sync {
    /// Get the list of supported protocol GTS IDs.
    fn supported_protocols(&self) -> &[String];

    /// Get the list of supported streaming protocol GTS IDs.
    fn supported_stream_protocols(&self) -> &[String];

    /// Get the list of supported auth type GTS IDs.
    fn supported_auth_types(&self) -> &[String];

    /// Get the list of supported strategy GTS IDs.
    fn supported_strategies(&self) -> &[String];

    /// Get the plugin priority (lower = higher priority).
    fn priority(&self) -> i16;

    /// Invoke an outbound API with unary semantics.
    ///
    /// # Arguments
    /// * `ctx` - Security context
    /// * `link` - Link configuration
    /// * `route` - Route configuration
    /// * `secret` - Secret material for authentication
    /// * `req` - Invocation request
    ///
    /// # Returns
    /// Response from downstream API or error
    async fn invoke_unary(
        &self,
        ctx: &SecurityContext,
        link: &Link,
        route: &Route,
        secret: &Secret,
        req: OagwInvokeRequest,
    ) -> Result<OagwInvokeResponse, OagwError>;

    /// Invoke an outbound API with streaming response.
    ///
    /// # Arguments
    /// * `ctx` - Security context
    /// * `link` - Link configuration
    /// * `route` - Route configuration
    /// * `secret` - Secret material for authentication
    /// * `req` - Invocation request
    ///
    /// # Returns
    /// Stream of response chunks or error
    async fn invoke_stream(
        &self,
        ctx: &SecurityContext,
        link: &Link,
        route: &Route,
        secret: &Secret,
        req: OagwInvokeRequest,
    ) -> Result<OagwResponseStream, OagwError>;
}
