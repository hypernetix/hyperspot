//! Configuration for the static tenant resolver plugin.

use serde::Deserialize;
use tenant_resolver_sdk::TenantStatus;
use uuid::Uuid;

/// Plugin configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StaticTrPluginConfig {
    /// Vendor name for GTS instance registration.
    pub vendor: String,

    /// Plugin priority (lower = higher priority).
    pub priority: i16,

    /// Static tenant definitions.
    pub tenants: Vec<TenantConfig>,

    /// Static access rules.
    pub access_rules: Vec<AccessRuleConfig>,
}

impl Default for StaticTrPluginConfig {
    fn default() -> Self {
        Self {
            vendor: "hyperspot".to_owned(),
            priority: 100,
            tenants: Vec::new(),
            access_rules: Vec::new(),
        }
    }
}

/// Configuration for a single tenant.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TenantConfig {
    /// Tenant ID.
    pub id: Uuid,

    /// Tenant name.
    pub name: String,

    /// Tenant status (defaults to Active).
    #[serde(default)]
    pub status: TenantStatus,

    /// Tenant type classification.
    #[serde(rename = "type", default)]
    pub tenant_type: Option<String>,
}

/// Configuration for an access rule.
///
/// Defines that `source` tenant can access `target` tenant's data.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccessRuleConfig {
    /// Source tenant ID (the one requesting access).
    pub source: Uuid,

    /// Target tenant ID (the one being accessed).
    pub target: Uuid,
}
