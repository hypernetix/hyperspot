//! Configuration for Fabrikam tenant resolver plugin.

use serde::Deserialize;
use tenant_resolver_example_sdk::TenantStatus;

/// Plugin configuration loaded from module config section.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FabrikamPluginConfig {
    /// Vendor name for this plugin instance.
    pub vendor: String,
    /// Priority for plugin selection (lower = higher priority).
    pub priority: i16,
    /// Tenant tree configuration.
    pub tenants: Vec<TenantConfig>,
}

/// Configuration for a single tenant in the tree.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TenantConfig {
    /// Unique tenant identifier (UUID hex).
    pub id: String,
    /// Parent tenant ID. `null`/missing/empty string means "root".
    #[serde(default)]
    pub parent_id: Option<String>,
    /// Tenant status.
    #[serde(default = "default_status")]
    pub status: TenantStatus,
    /// Whether parent can access this tenant.
    #[serde(default = "default_true")]
    pub is_accessible_by_parent: bool,
}

fn default_status() -> TenantStatus {
    TenantStatus::Active
}

fn default_true() -> bool {
    true
}

impl Default for FabrikamPluginConfig {
    fn default() -> Self {
        // Default: a simple 3-level tenant tree
        Self {
            vendor: "Fabrikam".to_owned(),
            priority: 20,
            tenants: vec![
                // Root tenant
                TenantConfig {
                    id: "00000000000000000000000000000001".to_owned(),
                    parent_id: None,
                    status: TenantStatus::Active,
                    is_accessible_by_parent: true,
                },
                // Level 1: Two children of root
                TenantConfig {
                    id: "00000000000000000000000000000010".to_owned(),
                    parent_id: Some("00000000000000000000000000000001".to_owned()),
                    status: TenantStatus::Active,
                    is_accessible_by_parent: true,
                },
                TenantConfig {
                    id: "00000000000000000000000000000011".to_owned(),
                    parent_id: Some("00000000000000000000000000000001".to_owned()),
                    status: TenantStatus::Active,
                    is_accessible_by_parent: true,
                },
                // Level 2: Children under first L1 tenant
                TenantConfig {
                    id: "00000000000000000000000000000100".to_owned(),
                    parent_id: Some("00000000000000000000000000000010".to_owned()),
                    status: TenantStatus::Active,
                    is_accessible_by_parent: true,
                },
                TenantConfig {
                    id: "00000000000000000000000000000101".to_owned(),
                    parent_id: Some("00000000000000000000000000000010".to_owned()),
                    status: TenantStatus::SoftDeleted,
                    is_accessible_by_parent: false,
                },
            ],
        }
    }
}
