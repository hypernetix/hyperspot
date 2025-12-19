//! Client models for `tenant_resolver` SDK.
//!
//! This module contains data transfer objects and client-facing models.
//! GTS schema types are defined separately in the `gts` module.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::TenantResolverError;

/// Tenant status enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TenantStatus {
    /// Status not specified.
    #[default]
    Unspecified,
    /// Tenant is active.
    Active,
    /// Tenant is soft-deleted.
    SoftDeleted,
}

/// Tenant model (example).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tenant {
    /// Unique tenant identifier.
    pub id: String,
    /// Parent tenant identifier (empty for root).
    pub parent_id: String,
    /// Current tenant status.
    pub status: TenantStatus,
    /// Tenant type (GTS identifier).
    pub r#type: String,
    /// Whether the tenant is accessible by parent.
    pub is_accessible_by_parent: bool,
}

/// Filter for tenant queries.
///
/// If `statuses` is empty, the server applies a default filter: ACTIVE tenants only.
/// If non-empty, returns tenants whose status is in this set.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TenantFilter {
    /// List of statuses to filter by (empty = ACTIVE only).
    #[serde(default)]
    pub statuses: Vec<TenantStatus>,
}

impl TenantFilter {
    /// Creates a filter that matches only ACTIVE tenants.
    #[must_use]
    pub fn active_only() -> Self {
        Self {
            statuses: vec![TenantStatus::Active],
        }
    }

    /// Checks if a tenant matches this filter.
    #[must_use]
    pub fn matches(&self, status: TenantStatus) -> bool {
        if self.statuses.is_empty() {
            // Default: ACTIVE only
            status == TenantStatus::Active
        } else {
            self.statuses.contains(&status)
        }
    }
}

/// Access control options for resolving tenant hierarchy.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccessOptions {
    /// If true, ignore `is_accessible_by_parent` constraints when resolving hierarchy.
    #[serde(default)]
    pub ignore_parent_access_constraints: bool,
}

/// Result for individual tenant lookup in batch operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantResult {
    /// The requested tenant ID.
    pub id: String,
    /// The tenant if found and accessible.
    pub tenant: Option<Tenant>,
    /// Error if lookup failed.
    pub error: Option<TenantResolverError>,
}

impl TenantResult {
    /// Creates a successful result.
    #[must_use]
    pub fn ok(tenant: Tenant) -> Self {
        Self {
            id: tenant.id.clone(),
            tenant: Some(tenant),
            error: None,
        }
    }

    /// Creates a not-found result.
    #[must_use]
    pub fn not_found(id: &str) -> Self {
        Self {
            id: id.to_owned(),
            tenant: None,
            error: Some(TenantResolverError::NotFound(format!(
                "tenant '{id}' not found"
            ))),
        }
    }

    /// Creates a permission-denied result.
    #[must_use]
    pub fn permission_denied(id: &str, reason: &str) -> Self {
        Self {
            id: id.to_owned(),
            tenant: None,
            error: Some(TenantResolverError::PermissionDenied(format!(
                "tenant '{id}': {reason}"
            ))),
        }
    }
}

/// Response for `get_parents` operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetParentsResponse {
    /// The target tenant.
    pub tenant: Tenant,
    /// Parent chain from direct parent to root (empty if tenant is root).
    pub parents: Vec<Tenant>,
}
