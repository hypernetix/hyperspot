//! Configuration for in-memory cache plugin.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// In-memory cache plugin configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct InMemoryCachePluginConfig {
    /// Vendor identifier for this plugin instance.
    #[serde(default = "default_vendor")]
    pub vendor: String,

    /// Priority for plugin selection (lower = higher priority).
    #[serde(default = "default_priority")]
    pub priority: i16,

    /// Time-to-live for cached entries.
    #[serde(default = "default_ttl", with = "modkit_utils::humantime_serde")]
    pub ttl: Duration,

    /// Maximum number of entries in cache.
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
}

fn default_vendor() -> String {
    "hyperspot".to_owned()
}

fn default_priority() -> i16 {
    100
}

fn default_ttl() -> Duration {
    Duration::from_secs(60)
}

fn default_max_entries() -> usize {
    10_000
}

impl Default for InMemoryCachePluginConfig {
    fn default() -> Self {
        Self {
            vendor: default_vendor(),
            priority: default_priority(),
            ttl: default_ttl(),
            max_entries: default_max_entries(),
        }
    }
}
