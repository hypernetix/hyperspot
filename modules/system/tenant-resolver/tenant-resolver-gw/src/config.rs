//! Configuration for the tenant resolver gateway.

use serde::Deserialize;

/// Gateway configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TenantResolverGwConfig {
    /// Vendor selector used to pick a plugin implementation.
    ///
    /// The gateway queries types-registry for plugin instances matching
    /// this vendor and selects the one with lowest priority.
    pub vendor: String,
}

impl Default for TenantResolverGwConfig {
    fn default() -> Self {
        Self {
            vendor: "hyperspot".to_owned(),
        }
    }
}
