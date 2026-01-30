//! Configuration for no-cache plugin.

use serde::{Deserialize, Serialize};

/// No-cache plugin configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NoCachePluginConfig {
    /// Vendor identifier for this plugin instance.
    #[serde(default = "default_vendor")]
    pub vendor: String,

    /// Priority for plugin selection (lower = higher priority).
    #[serde(default = "default_priority")]
    pub priority: i16,
}

fn default_vendor() -> String {
    "hyperspot".to_owned()
}

fn default_priority() -> i16 {
    100
}

impl Default for NoCachePluginConfig {
    fn default() -> Self {
        Self {
            vendor: default_vendor(),
            priority: default_priority(),
        }
    }
}
