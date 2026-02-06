//! Plugin API trait for tenant resolver implementations.
//!
//! Plugins implement this trait to provide tenant data and access rules.
//! The gateway discovers plugins via GTS types-registry and delegates
//! API calls to the selected plugin.

use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::error::TenantResolverError;
use crate::models::{AccessOptions, TenantFilter, TenantId, TenantInfo};

/// Plugin API trait for tenant resolver implementations.
///
/// Each plugin registers this trait with a scoped `ClientHub` entry
/// using its GTS instance ID as the scope.
///
/// The gateway calls these methods after applying cross-cutting concerns
/// (e.g., self-access check).
#[async_trait]
pub trait TenantResolverPluginClient: Send + Sync {
    /// Get tenant information by ID.
    ///
    /// Returns tenant info regardless of status - status filtering is only
    /// applied in `get_accessible_tenants`.
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

    /// Check if source tenant can access target tenant's data.
    ///
    /// The source tenant is taken from `ctx.tenant_id()`.
    /// Access rules (including status-based and permission-based) are
    /// plugin-determined.
    ///
    /// Note: Gateway already handles self-access (source == target).
    /// Plugin should check its access rules for cross-tenant access.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if access is allowed
    /// - `Ok(false)` if target exists but access is denied
    ///
    /// # Errors
    ///
    /// - `TenantNotFound` if the target tenant doesn't exist
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context (source tenant from `ctx.tenant_id()`)
    /// * `target` - The target tenant ID to check
    /// * `options` - Optional access options (e.g., required permissions)
    async fn can_access(
        &self,
        ctx: &SecurityContext,
        target: TenantId,
        options: Option<&AccessOptions>,
    ) -> Result<bool, TenantResolverError>;

    /// Get all tenants accessible by the source tenant.
    ///
    /// The source tenant is taken from `ctx.tenant_id()`.
    ///
    /// Note: Gateway ensures the source tenant is included in the result.
    /// Plugin should return tenants accessible via its access rules,
    /// filtered by the provided criteria.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context (source tenant from `ctx.tenant_id()`)
    /// * `filter` - Optional filter criteria (e.g., id, status)
    /// * `options` - Optional access options (e.g., required permissions)
    async fn get_accessible_tenants(
        &self,
        ctx: &SecurityContext,
        filter: Option<&TenantFilter>,
        options: Option<&AccessOptions>,
    ) -> Result<Vec<TenantInfo>, TenantResolverError>;
}
