//! Repository traits for OAGW domain.

use async_trait::async_trait;
use modkit_odata::{ODataQuery, Page};
use modkit_security::AccessScope;
use oagw_sdk::{Link, LinkPatch, NewLink, NewRoute, Route, RoutePatch};
use uuid::Uuid;

/// Repository trait for Route entities.
#[async_trait]
pub trait RouteRepository: Send + Sync {
    /// Find a route by ID within the security scope.
    async fn find_by_id(&self, scope: &AccessScope, id: Uuid) -> anyhow::Result<Option<Route>>;

    /// List routes with OData pagination.
    async fn list_page(
        &self,
        scope: &AccessScope,
        query: ODataQuery,
    ) -> anyhow::Result<Page<Route>>;

    /// Insert a new route.
    async fn insert(&self, scope: &AccessScope, new_route: NewRoute) -> anyhow::Result<Route>;

    /// Update a route with patch data.
    async fn update(
        &self,
        scope: &AccessScope,
        id: Uuid,
        patch: RoutePatch,
    ) -> anyhow::Result<Route>;

    /// Delete a route by ID.
    async fn delete(&self, scope: &AccessScope, id: Uuid) -> anyhow::Result<bool>;

    /// Check if a route exists.
    async fn exists(&self, scope: &AccessScope, id: Uuid) -> anyhow::Result<bool>;
}

/// Repository trait for Link entities.
#[async_trait]
pub trait LinkRepository: Send + Sync {
    /// Find a link by ID within the security scope.
    async fn find_by_id(&self, scope: &AccessScope, id: Uuid) -> anyhow::Result<Option<Link>>;

    /// Find all enabled links for a route.
    async fn find_enabled_by_route(
        &self,
        scope: &AccessScope,
        route_id: Uuid,
    ) -> anyhow::Result<Vec<Link>>;

    /// List links with OData pagination.
    async fn list_page(&self, scope: &AccessScope, query: ODataQuery)
        -> anyhow::Result<Page<Link>>;

    /// Insert a new link.
    async fn insert(&self, scope: &AccessScope, new_link: NewLink) -> anyhow::Result<Link>;

    /// Update a link with patch data.
    async fn update(&self, scope: &AccessScope, id: Uuid, patch: LinkPatch)
        -> anyhow::Result<Link>;

    /// Delete a link by ID.
    async fn delete(&self, scope: &AccessScope, id: Uuid) -> anyhow::Result<bool>;

    /// Check if a link exists.
    async fn exists(&self, scope: &AccessScope, id: Uuid) -> anyhow::Result<bool>;
}
