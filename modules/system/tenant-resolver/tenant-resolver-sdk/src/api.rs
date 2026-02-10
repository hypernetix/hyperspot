//! Public API trait for the tenant resolver gateway.
//!
//! This trait defines the interface that consumers use to interact with
//! the tenant resolver. The gateway implements this trait and delegates
//! to the appropriate plugin.

use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::error::TenantResolverError;
use crate::models::{
    GetAncestorsOptions, GetAncestorsResponse, GetDescendantsOptions, GetDescendantsResponse,
    GetTenantsOptions, IsAncestorOptions, TenantId, TenantInfo,
};

/// Public API trait for the tenant resolver gateway.
///
/// This trait is registered in `ClientHub` by the gateway module and
/// can be consumed by other modules:
///
/// ```ignore
/// let resolver = hub.get::<dyn TenantResolverGatewayClient>()?;
///
/// // Get single tenant info
/// let tenant = resolver.get_tenant(&ctx, tenant_id).await?;
///
/// // Batch get tenants
/// let tenants = resolver.get_tenants(&ctx, &[id1, id2], &GetTenantsOptions::default()).await?;
///
/// // Get ancestor chain
/// let response = resolver.get_ancestors(&ctx, tenant_id, &GetAncestorsOptions::default()).await?;
///
/// // Get descendants subtree
/// let descendants = resolver.get_descendants(&ctx, tenant_id, &GetDescendantsOptions::default()).await?;
///
/// // Check ancestry
/// let is_anc = resolver.is_ancestor(&ctx, parent_id, child_id, &IsAncestorOptions::default()).await?;
/// ```
///
/// # Security Context
///
/// The gateway does **not** perform its own access-control checks.
/// The `SecurityContext` is passed through to the plugin, which decides
/// how (or whether) to enforce authorization. This is intentional —
/// different plugins may have different access-control semantics.
#[async_trait]
pub trait TenantResolverGatewayClient: Send + Sync {
    /// Get tenant information by ID.
    ///
    /// Returns tenant info regardless of status - the consumer can decide
    /// how to handle different statuses (active, suspended, deleted).
    ///
    /// # Errors
    ///
    /// - `TenantNotFound` if the tenant does not exist
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context (`tenant_id` used for access control)
    /// * `id` - The tenant ID to retrieve
    async fn get_tenant(
        &self,
        ctx: &SecurityContext,
        id: TenantId,
    ) -> Result<TenantInfo, TenantResolverError>;

    /// Get multiple tenants by IDs (batch).
    ///
    /// Returns only found tenants - missing IDs are silently skipped.
    /// This is useful for batch lookups where some tenants may not exist.
    ///
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
    /// direct parent to root. If the tenant is a root, `ancestors` will be empty.
    ///
    /// # Barrier Behavior
    ///
    /// By default (`BarrierMode::Respect`), traversal stops at barrier boundaries:
    /// - If the starting tenant itself has `self_managed = true`, `ancestors` is empty
    /// - If a tenant in the chain has `self_managed = true`, it acts as a barrier
    ///   and ancestors above it are not included
    ///
    /// Use `BarrierMode::Ignore` to traverse through barriers.
    ///
    /// # Errors
    ///
    /// - `TenantNotFound` if the tenant does not exist
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
    /// Returns the requested tenant along with all its descendant tenants
    /// (children, grandchildren, etc.) up to the specified depth.
    ///
    /// # Barrier Behavior
    ///
    /// By default (`BarrierMode::Respect`), self-managed tenants act as barriers:
    /// - The barrier tenant itself is NOT included in descendants
    /// - Its subtree is not traversed
    ///
    /// Use `BarrierMode::Ignore` to include barrier tenants and their subtrees.
    ///
    /// # Errors
    ///
    /// - `TenantNotFound` if the tenant does not exist
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
    /// By default (`BarrierMode::Respect`), barriers block ancestry checks:
    /// - If `descendant_id` is `self_managed`, returns `false` (barrier blocks parentage)
    /// - If there's a barrier tenant between ancestor and descendant, returns `false`
    ///
    /// Use `BarrierMode::Ignore` to ignore barriers.
    ///
    /// # Errors
    ///
    /// - `TenantNotFound` if either tenant does not exist
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
