//! Public API trait for the tenant resolver gateway.
//!
//! This trait defines the interface that consumers use to interact with
//! the tenant resolver. The gateway implements this trait and delegates
//! to the appropriate plugin.

use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::error::TenantResolverError;
use crate::models::{AccessOptions, TenantFilter, TenantId, TenantInfo};

/// Public API trait for the tenant resolver gateway.
///
/// This trait is registered in `ClientHub` by the gateway module and
/// can be consumed by other modules:
///
/// ```ignore
/// let resolver = hub.get::<dyn TenantResolverGatewayClient>()?;
///
/// // Get tenant info
/// let tenant = resolver.get_tenant(&ctx, tenant_id).await?;
///
/// // Check basic access
/// let can = resolver.can_access(&ctx, target_id, None).await?;
///
/// // Get accessible tenants with filter
/// let filter = TenantFilter { status: vec![TenantStatus::Active], ..Default::default() };
/// let tenants = resolver.get_accessible_tenants(&ctx, Some(&filter), None).await?;
/// ```
///
/// The source tenant for access checks is always taken from `ctx.tenant_id()`.
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

    /// Check if the current tenant can access the target tenant's data.
    ///
    /// The source tenant is taken from `ctx.tenant_id()`.
    /// Access rules (including status-based restrictions) are plugin-determined.
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
    /// # Access Rules
    ///
    /// - Self-access: A tenant can always access its own data
    /// - Non-transitive: A→B and B→C does NOT imply A→C
    /// - Non-symmetric: A→B does NOT imply B→A
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context (`tenant_id` is the source tenant)
    /// * `target` - The target tenant ID to check access for
    /// * `options` - Optional access options (e.g., required permissions)
    async fn can_access(
        &self,
        ctx: &SecurityContext,
        target: TenantId,
        options: Option<&AccessOptions>,
    ) -> Result<bool, TenantResolverError>;

    /// Get all tenants that the current tenant can access.
    ///
    /// The source tenant is taken from `ctx.tenant_id()`.
    /// The result includes the source tenant itself (self-access),
    /// provided it matches the filter criteria.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context (`tenant_id` is the source tenant)
    /// * `filter` - Optional filter criteria (e.g., id, status)
    /// * `options` - Optional access options (e.g., required permissions)
    async fn get_accessible_tenants(
        &self,
        ctx: &SecurityContext,
        filter: Option<&TenantFilter>,
        options: Option<&AccessOptions>,
    ) -> Result<Vec<TenantInfo>, TenantResolverError>;
}
