//! Plugin API trait for tenant resolver implementations.
//!
//! Plugins implement this trait to provide tenant data and hierarchy traversal.
//! The gateway discovers plugins via GTS types-registry and delegates
//! API calls to the selected plugin.

use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::error::TenantResolverError;
use crate::models::{
    GetAncestorsOptions, GetAncestorsResponse, GetDescendantsOptions, GetDescendantsResponse,
    GetTenantsOptions, IsAncestorOptions, TenantId, TenantInfo,
};

/// Plugin API trait for tenant resolver implementations.
///
/// Each plugin registers this trait with a scoped `ClientHub` entry
/// using its GTS instance ID as the scope.
///
/// The gateway delegates to these methods. Cross-cutting concerns (logging,
/// metrics) may be added at the gateway level in the future.
///
/// # Security Context
///
/// The gateway passes the `SecurityContext` through without validation.
/// Each plugin decides how (or whether) to enforce authorization.
#[async_trait]
pub trait TenantResolverPluginClient: Send + Sync {
    /// Get tenant information by ID.
    ///
    /// Returns tenant info regardless of status - status filtering is only
    /// applied in listing operations.
    ///
    /// # Errors
    ///
    /// - `TenantNotFound` if the tenant doesn't exist in the plugin's data source
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context
    /// * `id` - The tenant ID to retrieve
    async fn get_tenant(
        &self,
        ctx: &SecurityContext,
        id: TenantId,
    ) -> Result<TenantInfo, TenantResolverError>;

    /// Get multiple tenants by IDs (batch).
    ///
    /// Returns only found tenants - missing IDs are silently skipped.
    /// Output order is not guaranteed. Duplicate IDs are deduplicated.
    /// Returns an empty list when `ids` is empty.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context
    /// * `ids` - The tenant IDs to retrieve
    /// * `options` - Filter options (e.g., status)
    async fn get_tenants(
        &self,
        ctx: &SecurityContext,
        ids: &[TenantId],
        options: &GetTenantsOptions,
    ) -> Result<Vec<TenantInfo>, TenantResolverError>;

    /// Get ancestor chain from tenant to root.
    ///
    /// Returns the requested tenant along with its ancestors ordered from
    /// direct parent to root.
    ///
    /// # Barrier Behavior
    ///
    /// With `BarrierMode::Respect` (default):
    /// - If the starting tenant is `self_managed`, return empty ancestors
    /// - If an ancestor in the chain is `self_managed`, include it but stop traversal
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context
    /// * `id` - The tenant ID to get ancestors for
    /// * `options` - Hierarchy traversal options
    async fn get_ancestors(
        &self,
        ctx: &SecurityContext,
        id: TenantId,
        options: &GetAncestorsOptions,
    ) -> Result<GetAncestorsResponse, TenantResolverError>;

    /// Get descendants subtree of the given tenant.
    ///
    /// Returns the requested tenant along with all its descendant tenants.
    ///
    /// # Barrier Behavior
    ///
    /// With `BarrierMode::Respect` (default):
    /// - Self-managed children are NOT included in descendants
    /// - Their subtrees are not traversed
    ///
    /// # Errors
    ///
    /// - `TenantNotFound` if the tenant doesn't exist in the plugin's data source
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context
    /// * `id` - The tenant ID to get descendants for
    /// * `options` - Filter, barrier mode, and depth options
    async fn get_descendants(
        &self,
        ctx: &SecurityContext,
        id: TenantId,
        options: &GetDescendantsOptions,
    ) -> Result<GetDescendantsResponse, TenantResolverError>;

    /// Check if `ancestor_id` is an ancestor of `descendant_id`.
    ///
    /// Returns `true` if `ancestor_id` is in the parent chain of `descendant_id`.
    /// Returns `false` if `ancestor_id == descendant_id` (self is not an ancestor of self).
    ///
    /// # Barrier Behavior
    ///
    /// With `BarrierMode::Respect` (default):
    /// - If `descendant_id` is `self_managed`, return `false` (barrier blocks parentage)
    /// - If a `self_managed` tenant lies between ancestor and descendant, return `false`
    ///
    /// # Errors
    ///
    /// - `TenantNotFound` if either tenant doesn't exist in the plugin's data source
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context
    /// * `ancestor_id` - The potential ancestor tenant ID
    /// * `descendant_id` - The potential descendant tenant ID
    /// * `options` - Hierarchy traversal options
    async fn is_ancestor(
        &self,
        ctx: &SecurityContext,
        ancestor_id: TenantId,
        descendant_id: TenantId,
        options: &IsAncestorOptions,
    ) -> Result<bool, TenantResolverError>;
}
