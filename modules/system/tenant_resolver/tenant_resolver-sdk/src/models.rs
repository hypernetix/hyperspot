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

/// Trait for types that expose a [`TenantStatus`].
///
/// Used by [`matches_status`] to filter both [`TenantInfo`] and
/// [`TenantRef`] without duplicating logic.
pub trait HasStatus {
    /// Returns the tenant's current status.
    fn status(&self) -> TenantStatus;
}

/// Returns `true` if the tenant matches the given status filter.
///
/// An empty `statuses` slice means "no constraint" (include all).
/// Works with any type implementing [`HasStatus`].
#[must_use]
pub fn matches_status<T: HasStatus>(tenant: &T, statuses: &[TenantStatus]) -> bool {
    statuses.is_empty() || statuses.contains(&tenant.status())
}

impl HasStatus for TenantInfo {
    fn status(&self) -> TenantStatus {
        self.status
    }
}

impl HasStatus for TenantRef {
    fn status(&self) -> TenantStatus {
        self.status
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
/// use tenant_resolver_sdk::BarrierMode;
///
/// // Default: respect all barriers
/// let mode = BarrierMode::default();
/// assert_eq!(mode, BarrierMode::Respect);
///
/// // Ignore barriers (traverse through self-managed tenants)
/// let mode = BarrierMode::Ignore;
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BarrierMode {
    /// Respect all barriers - stop traversal at barrier boundaries (default).
    #[default]
    Respect,
    /// Ignore barriers - traverse through self-managed tenants.
    Ignore,
}

/// Request parameters for [`get_ancestors`](crate::TenantResolverGatewayClient::get_ancestors).
///
/// # Example
///
/// ```
/// use tenant_resolver_sdk::{BarrierMode, GetAncestorsOptions};
///
/// // Default: respect barriers
/// let req = GetAncestorsOptions::default();
///
/// // Ignore barriers during traversal
/// let req = GetAncestorsOptions {
///     barrier_mode: BarrierMode::Ignore,
/// };
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetAncestorsOptions {
    /// How to handle barriers during traversal.
    pub barrier_mode: BarrierMode,
}

/// Options for [`get_tenants`](crate::TenantResolverGatewayClient::get_tenants).
///
/// # Example
///
/// ```
/// use tenant_resolver_sdk::{GetTenantsOptions, TenantStatus};
///
/// // Default: no filter (all statuses)
/// let opts = GetTenantsOptions::default();
///
/// // Only active tenants
/// let opts = GetTenantsOptions {
///     status: vec![TenantStatus::Active],
/// };
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetTenantsOptions {
    /// Filter by tenant status. Empty means all statuses are included.
    #[serde(default)]
    pub status: Vec<TenantStatus>,
}

/// Options for [`get_descendants`](crate::TenantResolverGatewayClient::get_descendants).
///
/// # Example
///
/// ```
/// use tenant_resolver_sdk::{BarrierMode, GetDescendantsOptions, TenantStatus};
///
/// // Default: no filter, respect barriers, unlimited depth
/// let opts = GetDescendantsOptions::default();
///
/// // Active tenants only, ignore barriers, depth 2
/// let opts = GetDescendantsOptions {
///     status: vec![TenantStatus::Active],
///     barrier_mode: BarrierMode::Ignore,
///     max_depth: Some(2),
/// };
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetDescendantsOptions {
    /// Filter descendants by status. Empty means all statuses are included.
    /// Does NOT apply to the starting tenant.
    #[serde(default)]
    pub status: Vec<TenantStatus>,
    /// How to handle barriers during traversal.
    pub barrier_mode: BarrierMode,
    /// Maximum depth to traverse (`None` = unlimited, `Some(1)` = direct children only).
    pub max_depth: Option<u32>,
}

/// Request parameters for [`is_ancestor`](crate::TenantResolverGatewayClient::is_ancestor).
///
/// # Example
///
/// ```
/// use tenant_resolver_sdk::{BarrierMode, IsAncestorOptions};
///
/// // Default: respect barriers
/// let req = IsAncestorOptions::default();
///
/// // Ignore barriers
/// let req = IsAncestorOptions {
///     barrier_mode: BarrierMode::Ignore,
/// };
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IsAncestorOptions {
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
