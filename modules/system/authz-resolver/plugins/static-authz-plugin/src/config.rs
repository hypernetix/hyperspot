//! Configuration for the static `AuthZ` resolver plugin.

use serde::Deserialize;

/// Plugin configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StaticAuthZPluginConfig {
    /// Vendor name for GTS instance registration.
    pub vendor: String,

    /// Plugin priority (lower = higher priority).
    pub priority: i16,

    /// Authorization mode.
    pub mode: AuthZMode,
}

impl Default for StaticAuthZPluginConfig {
    fn default() -> Self {
        Self {
            vendor: "hyperspot".to_owned(),
            priority: 100,
            mode: AuthZMode::AllowAll,
        }
    }
}

/// Authorization mode.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuthZMode {
    /// Allow all requests. For constrained operations, scope to context tenant.
    #[default]
    AllowAll,
}
