//! Plugin configuration.

use serde::Deserialize;

/// Configuration for the default OAGW plugin.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PluginConfig {
    /// Vendor name for the plugin.
    pub vendor: String,
    /// Priority for plugin selection (lower = higher priority).
    pub priority: i16,
    /// Default timeout for HTTP requests in milliseconds.
    pub default_timeout_ms: u64,
    /// Maximum response body size in bytes.
    pub max_response_size_bytes: usize,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            vendor: "x".to_string(),
            priority: 10,
            default_timeout_ms: 30_000,
            max_response_size_bytes: 104_857_600, // 100 MiB
        }
    }
}
