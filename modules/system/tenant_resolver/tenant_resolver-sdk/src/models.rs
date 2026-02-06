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
    /// Parent tenant ID. `None` for root tenants.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<TenantId>,
    /// Whether this tenant is self-managed (barrier).
    /// When `true`, parent tenants cannot traverse into this subtree
    /// unless `BarrierMode::Ignore` is used.
    #[serde(default)]
    pub self_managed: bool,
}

/// Tenant reference for hierarchy operations (without name).
///
/// Used by `get_ancestors` and `get_descendants` to return tenant metadata
/// without the display name. If names are needed, use `get_tenants` with
/// the collected IDs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantRef {
    /// Unique tenant identifier.
    pub id: TenantId,
    /// Current status of the tenant.
    pub status: TenantStatus,
    /// Tenant type classification.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub tenant_type: Option<String>,
    /// Parent tenant ID. `None` for root tenants.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<TenantId>,
    /// Whether this tenant is self-managed (barrier).
    #[serde(default)]
    pub self_managed: bool,
}

impl From<TenantInfo> for TenantRef {
    fn from(info: TenantInfo) -> Self {
        Self {
            id: info.id,
            status: info.status,
            tenant_type: info.tenant_type,
            parent_id: info.parent_id,
            self_managed: info.self_managed,
        }
    }
}

impl From<&TenantInfo> for TenantRef {
    fn from(info: &TenantInfo) -> Self {
        Self {
            id: info.id,
            status: info.status,
            tenant_type: info.tenant_type.clone(),
            parent_id: info.parent_id,
            self_managed: info.self_managed,
        }
    }
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
/// Used by `get_tenants` and `get_descendants` to filter results.
/// An empty `status` vector means "no constraint" (include all statuses).
///
/// # Example
///
/// ```
/// use tenant_resolver_sdk::{TenantFilter, TenantStatus};
///
/// // No filter (all tenants)
/// let filter = TenantFilter::default();
///
/// // Only active tenants
/// let filter = TenantFilter {
///     status: vec![TenantStatus::Active],
/// };
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantFilter {
    /// Filter by tenant status. Empty means all statuses are included.
    pub status: Vec<TenantStatus>,
}

impl TenantFilter {
    /// Returns `true` if no filter criteria are set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.status.is_empty()
    }

    /// Returns `true` if the given tenant matches all filter criteria.
    ///
    /// An empty `status` vector means "no constraint" (include all).
    #[must_use]
    pub fn matches(&self, tenant: &TenantInfo) -> bool {
        if !self.status.is_empty() && !self.status.contains(&tenant.status) {
            return false;
        }
        true
    }

    /// Returns `true` if the given tenant ref matches all filter criteria.
    ///
    /// Same as [`matches`](Self::matches) but for [`TenantRef`].
    /// Intended for consumers that need to post-filter hierarchy responses
    /// (e.g., filtering `GetAncestorsResponse::ancestors`).
    #[must_use]
    pub fn matches_ref(&self, tenant: &TenantRef) -> bool {
        if !self.status.is_empty() && !self.status.contains(&tenant.status) {
            return false;
        }
        true
    }
}

/// Controls how barriers (self-managed tenants) are handled during hierarchy traversal.
///
/// A barrier is a tenant with `self_managed = true`. By default, traversal stops
/// at barrier boundaries - a parent tenant cannot see into a self-managed subtree.
///
/// # Example
///
/// ```
/// use tenant_resolver_sdk::{BarrierMode, HierarchyOptions};
///
/// // Default: respect all barriers
/// let opts = HierarchyOptions::default();
/// assert_eq!(opts.barrier_mode, BarrierMode::Respect);
///
/// // Ignore barriers (traverse through self-managed tenants)
/// let opts = HierarchyOptions {
///     barrier_mode: BarrierMode::Ignore,
/// };
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BarrierMode {
    /// Respect all barriers - stop traversal at barrier boundaries (default).
    #[default]
    Respect,
    /// Ignore barriers - traverse through self-managed tenants.
    Ignore,
}

/// Options for hierarchy traversal operations (`get_ancestors`, `get_descendants`, `is_ancestor`).
///
/// # Example
///
/// ```
/// use tenant_resolver_sdk::{BarrierMode, HierarchyOptions};
///
/// // Default options (respect barriers)
/// let opts = HierarchyOptions::default();
///
/// // Ignore barriers during traversal
/// let opts = HierarchyOptions {
///     barrier_mode: BarrierMode::Ignore,
/// };
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HierarchyOptions {
    /// How to handle barriers during traversal.
    pub barrier_mode: BarrierMode,
}

/// Response for `get_ancestors` containing the requested tenant and its ancestor chain.
///
/// Uses [`TenantRef`] (without name) for efficiency. If names are needed,
/// collect the IDs and call `get_tenants`.
///
/// # Example
///
/// Given hierarchy: `Root -> Parent -> Child`
///
/// `get_ancestors(Child)` returns:
/// - `tenant`: Child ref
/// - `ancestors`: [Parent, Root] (ordered from direct parent to root)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetAncestorsResponse {
    /// The requested tenant (without name).
    pub tenant: TenantRef,
    /// Parent chain ordered from direct parent to root.
    /// Empty if the tenant is a root tenant.
    pub ancestors: Vec<TenantRef>,
}

/// Response for `get_descendants` containing the requested tenant and its descendants.
///
/// Uses [`TenantRef`] (without name) for efficiency. If names are needed,
/// collect the IDs and call `get_tenants`.
///
/// # Example
///
/// Given hierarchy: `Root -> [Child1, Child2 -> Grandchild]`
///
/// `get_descendants(Root)` returns:
/// - `tenant`: Root ref
/// - `descendants`: [Child1, Child2, Grandchild] (pre-order traversal)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetDescendantsResponse {
    /// The requested tenant (without name).
    pub tenant: TenantRef,
    /// All descendants (children, grandchildren, etc.) in pre-order.
    /// Empty if the tenant has no children.
    pub descendants: Vec<TenantRef>,
}
