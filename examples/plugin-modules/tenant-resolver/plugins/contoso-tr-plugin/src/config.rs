//! Configuration for Contoso tenant resolver plugin.

use serde::Deserialize;

/// Plugin configuration loaded from module config section.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ContosoPluginConfig {
    /// Vendor name for this plugin instance.
    pub vendor: String,
    /// Priority for plugin selection (lower = higher priority).
    pub priority: i16,
}

impl Default for ContosoPluginConfig {
    fn default() -> Self {
        Self {
            vendor: "Contoso".to_owned(),
            priority: 10,
        }
    }
}
