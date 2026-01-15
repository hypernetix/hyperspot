//! REST API DTOs for tenant resolver gateway.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use tenant_resolver_sdk::{GetParentsResponse, GtsSchemaId, Tenant, TenantStatus};

// ============================================================================
// Tenant DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TenantDto {
    pub id: String,
    pub parent_id: String,
    pub status: TenantStatusDto,
    #[serde(skip, default = "default_schema_id")]
    pub r#type: GtsSchemaId,
    pub is_accessible_by_parent: bool,
}

fn default_schema_id() -> GtsSchemaId {
    tenant_resolver_sdk::TenantSpecV1::<()>::gts_schema_id().clone()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TenantStatusDto {
    Unspecified,
    Active,
    SoftDeleted,
}

impl From<TenantStatus> for TenantStatusDto {
    fn from(v: TenantStatus) -> Self {
        match v {
            TenantStatus::Unspecified => Self::Unspecified,
            TenantStatus::Active => Self::Active,
            TenantStatus::SoftDeleted => Self::SoftDeleted,
        }
    }
}

impl From<TenantStatusDto> for TenantStatus {
    fn from(v: TenantStatusDto) -> Self {
        match v {
            TenantStatusDto::Unspecified => Self::Unspecified,
            TenantStatusDto::Active => Self::Active,
            TenantStatusDto::SoftDeleted => Self::SoftDeleted,
        }
    }
}

impl From<Tenant> for TenantDto {
    fn from(t: Tenant) -> Self {
        Self {
            id: t.id,
            parent_id: t.parent_id,
            status: t.status.into(),
            r#type: t.r#type,
            is_accessible_by_parent: t.is_accessible_by_parent,
        }
    }
}

// ============================================================================
// Request DTOs
// ============================================================================

/// Filter for tenant queries.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct TenantFilterDto {
    /// List of statuses to filter by. Empty = ACTIVE only.
    #[serde(default)]
    pub statuses: Vec<TenantStatusDto>,
}

impl From<TenantFilterDto> for tenant_resolver_sdk::TenantFilter {
    fn from(v: TenantFilterDto) -> Self {
        Self {
            statuses: v.statuses.into_iter().map(Into::into).collect(),
        }
    }
}

// ============================================================================
// Shared Helpers
// ============================================================================

/// Parses a comma-separated status string into a `Vec<TenantStatus>`.
///
/// Recognized values (case-insensitive): `ACTIVE`, `SOFT_DELETED`, `UNSPECIFIED`.
/// Unknown values are ignored.
fn parse_statuses_csv(statuses: Option<&str>) -> Vec<TenantStatus> {
    statuses
        .unwrap_or("")
        .split(',')
        .filter_map(|raw| {
            let s = raw.trim();
            if s.eq_ignore_ascii_case("ACTIVE") {
                Some(TenantStatus::Active)
            } else if s.eq_ignore_ascii_case("SOFT_DELETED") {
                Some(TenantStatus::SoftDeleted)
            } else if s.eq_ignore_ascii_case("UNSPECIFIED") {
                Some(TenantStatus::Unspecified)
            } else {
                None
            }
        })
        .collect()
}

/// Access control options.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct AccessOptionsDto {
    /// Ignore parent access constraints when resolving hierarchy.
    #[serde(default)]
    pub ignore_parent_access_constraints: bool,
}

impl From<AccessOptionsDto> for tenant_resolver_sdk::AccessOptions {
    fn from(v: AccessOptionsDto) -> Self {
        Self {
            ignore_parent_access_constraints: v.ignore_parent_access_constraints,
        }
    }
}

/// Query parameters for `list_tenants` endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListTenantsQuery {
    /// Comma-separated statuses (e.g., `ACTIVE,SOFT_DELETED`). Empty = ACTIVE only.
    pub statuses: Option<String>,
}

impl ListTenantsQuery {
    /// Converts query params to `TenantFilter`.
    #[must_use]
    pub fn to_filter(&self) -> tenant_resolver_sdk::TenantFilter {
        tenant_resolver_sdk::TenantFilter {
            statuses: parse_statuses_csv(self.statuses.as_deref()),
        }
    }
}

/// Query parameters for `get_parents` endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct GetParentsQuery {
    /// Comma-separated statuses (e.g., `ACTIVE,SOFT_DELETED`). Empty = ACTIVE only.
    pub statuses: Option<String>,
    /// Ignore parent access constraints.
    pub ignore_access: Option<bool>,
}

impl GetParentsQuery {
    /// Converts query params to `TenantFilter`.
    #[must_use]
    pub fn to_filter(&self) -> tenant_resolver_sdk::TenantFilter {
        tenant_resolver_sdk::TenantFilter {
            statuses: parse_statuses_csv(self.statuses.as_deref()),
        }
    }

    /// Converts query params to `AccessOptions`.
    #[must_use]
    pub fn to_access_options(&self) -> tenant_resolver_sdk::AccessOptions {
        tenant_resolver_sdk::AccessOptions {
            ignore_parent_access_constraints: self.ignore_access.unwrap_or(false),
        }
    }
}

/// Query parameters for `get_children` endpoint.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct GetChildrenQuery {
    /// Comma-separated statuses (e.g., `ACTIVE,SOFT_DELETED`). Empty = ACTIVE only.
    pub statuses: Option<String>,
    /// Ignore parent access constraints.
    pub ignore_access: Option<bool>,
    /// Max depth: 0 = unlimited, 1 = direct children only.
    pub max_depth: Option<u32>,
}

impl GetChildrenQuery {
    /// Converts query params to `TenantFilter`.
    #[must_use]
    pub fn to_filter(&self) -> tenant_resolver_sdk::TenantFilter {
        tenant_resolver_sdk::TenantFilter {
            statuses: parse_statuses_csv(self.statuses.as_deref()),
        }
    }

    /// Converts query params to `AccessOptions`.
    #[must_use]
    pub fn to_access_options(&self) -> tenant_resolver_sdk::AccessOptions {
        tenant_resolver_sdk::AccessOptions {
            ignore_parent_access_constraints: self.ignore_access.unwrap_or(false),
        }
    }
}

// ============================================================================
// Response DTOs
// ============================================================================

/// Response for `get_parents` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetParentsResponseDto {
    /// The target tenant.
    pub tenant: TenantDto,
    /// Parent chain from direct parent to root.
    pub parents: Vec<TenantDto>,
}

impl From<GetParentsResponse> for GetParentsResponseDto {
    fn from(r: GetParentsResponse) -> Self {
        Self {
            tenant: r.tenant.into(),
            parents: r.parents.into_iter().map(Into::into).collect(),
        }
    }
}

/// Response for `get_children` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetChildrenResponseDto {
    pub children: Vec<TenantDto>,
}
