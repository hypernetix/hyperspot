//! Configuration for the `AuthZ` resolver.

use serde::Deserialize;

/// Configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AuthZResolverConfig {
    /// Vendor selector used to pick a plugin implementation.
    pub vendor: String,
}

impl Default for AuthZResolverConfig {
    fn default() -> Self {
        Self {
            vendor: "hyperspot".to_owned(),
        }
    }
}
