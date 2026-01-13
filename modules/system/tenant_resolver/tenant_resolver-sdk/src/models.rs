//! Domain models for the tenant resolver module.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a tenant.
pub type TenantId = Uuid;

/// Information about a tenant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantInfo {
    /// Unique tenant identifier.
    pub id: TenantId,
    /// Human-readable tenant name.
    pub name: String,
    /// Current status of the tenant.
    pub status: TenantStatus,
    /// Tenant type classification.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub tenant_type: Option<String>,
}

/// Tenant lifecycle status.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantStatus {
    /// Tenant is active and operational.
    #[default]
    Active,
    /// Tenant is temporarily suspended.
    Suspended,
    /// Tenant has been deleted (soft delete).
    Deleted,
}

/// Filter for tenant listing queries.
///
/// Used by `get_accessible_tenants` to filter results.
/// Empty vectors mean "no constraint" (include all).
///
/// # Example
///
/// ```
/// use hs_tenant_resolver_sdk::{TenantFilter, TenantStatus};
/// use uuid::Uuid;
///
/// // No filter (all tenants)
/// let filter = TenantFilter::default();
///
/// // Only active tenants
/// let filter = TenantFilter {
///     status: vec![TenantStatus::Active],
///     ..Default::default()
/// };
///
/// // Specific tenant IDs with active status
/// let tenant_a = Uuid::new_v4();
/// let tenant_b = Uuid::new_v4();
/// let filter = TenantFilter {
///     id: vec![tenant_a, tenant_b],
///     status: vec![TenantStatus::Active],
/// };
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TenantFilter {
    /// Filter by tenant IDs. Empty means all IDs are included.
    pub id: Vec<TenantId>,
    /// Filter by tenant status. Empty means all statuses are included.
    pub status: Vec<TenantStatus>,
}

impl TenantFilter {
    /// Returns `true` if no filter criteria are set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.id.is_empty() && self.status.is_empty()
    }
}

/// Options for access checking.
///
/// Used by `can_access` and `get_accessible_tenants` to specify
/// permission requirements. Empty vectors mean "no constraint".
///
/// # Example
///
/// ```
/// use hs_tenant_resolver_sdk::AccessOptions;
///
/// // Basic access check (no specific permission required)
/// let options = AccessOptions::default();
///
/// // Check for specific permission
/// let options = AccessOptions {
///     permission: vec!["read".to_string()],
/// };
///
/// // Check for multiple permissions (all required - AND semantics)
/// let options = AccessOptions {
///     permission: vec!["read".to_string(), "write".to_string()],
/// };
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccessOptions {
    /// Required permissions (all must be satisfied - AND semantics).
    /// Empty means no specific permission required.
    pub permission: Vec<String>,
}

impl AccessOptions {
    /// Returns `true` if no access options are set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.permission.is_empty()
    }
}
