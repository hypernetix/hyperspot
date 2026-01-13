//! Local client adapter implementing the SDK API trait.
//!
//! This adapter bridges the domain service to the public OagwApi trait,
//! allowing other modules to consume OAGW via ClientHub.

use std::sync::Arc;

use async_trait::async_trait;
use modkit_odata::{ODataQuery, Page};
use modkit_security::SecurityContext;
use oagw_sdk::{
    Link, LinkPatch, NewLink, NewRoute, OagwApi, OagwError, OagwInvokeRequest, OagwInvokeResponse,
    OagwResponseStream, Route, RoutePatch,
};
use uuid::Uuid;

use crate::domain::service::Service;

/// Local client adapter implementing the SDK API trait.
///
/// Registered in ClientHub during module init().
pub struct OagwLocalClient {
    service: Arc<Service>,
}

impl OagwLocalClient {
    /// Create a new local client adapter.
    pub fn new(service: Arc<Service>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl OagwApi for OagwLocalClient {
    // === Invocation Methods ===

    async fn invoke_unary(
        &self,
        ctx: &SecurityContext,
        req: OagwInvokeRequest,
    ) -> Result<OagwInvokeResponse, OagwError> {
        self.service
            .invoke_unary(ctx, req)
            .await
            .map_err(Into::into)
    }

    async fn invoke_stream(
        &self,
        ctx: &SecurityContext,
        req: OagwInvokeRequest,
    ) -> Result<OagwResponseStream, OagwError> {
        self.service
            .invoke_stream(ctx, req)
            .await
            .map_err(Into::into)
    }

    // === Route CRUD ===

    async fn create_route(
        &self,
        ctx: &SecurityContext,
        new_route: NewRoute,
    ) -> Result<Route, OagwError> {
        self.service
            .create_route(ctx, new_route)
            .await
            .map_err(Into::into)
    }

    async fn get_route(&self, ctx: &SecurityContext, id: Uuid) -> Result<Route, OagwError> {
        self.service.get_route(ctx, id).await.map_err(Into::into)
    }

    async fn list_routes(
        &self,
        ctx: &SecurityContext,
        query: ODataQuery,
    ) -> Result<Page<Route>, OagwError> {
        self.service
            .list_routes(ctx, query)
            .await
            .map_err(Into::into)
    }

    async fn update_route(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
        patch: RoutePatch,
    ) -> Result<Route, OagwError> {
        self.service
            .update_route(ctx, id, patch)
            .await
            .map_err(Into::into)
    }

    async fn delete_route(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), OagwError> {
        self.service.delete_route(ctx, id).await.map_err(Into::into)
    }

    // === Link CRUD ===

    async fn create_link(
        &self,
        ctx: &SecurityContext,
        new_link: NewLink,
    ) -> Result<Link, OagwError> {
        self.service
            .create_link(ctx, new_link)
            .await
            .map_err(Into::into)
    }

    async fn get_link(&self, ctx: &SecurityContext, id: Uuid) -> Result<Link, OagwError> {
        self.service.get_link(ctx, id).await.map_err(Into::into)
    }

    async fn list_links(
        &self,
        ctx: &SecurityContext,
        query: ODataQuery,
    ) -> Result<Page<Link>, OagwError> {
        self.service
            .list_links(ctx, query)
            .await
            .map_err(Into::into)
    }

    async fn update_link(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
        patch: LinkPatch,
    ) -> Result<Link, OagwError> {
        self.service
            .update_link(ctx, id, patch)
            .await
            .map_err(Into::into)
    }

    async fn delete_link(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), OagwError> {
        self.service.delete_link(ctx, id).await.map_err(Into::into)
    }
}
