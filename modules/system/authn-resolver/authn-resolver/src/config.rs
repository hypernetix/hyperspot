//! Configuration for the `AuthN` resolver.

use serde::Deserialize;

/// Configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AuthNResolverConfig {
    /// Vendor selector used to pick a plugin implementation.
    ///
    /// The resolver queries types-registry for plugin instances matching
    /// this vendor and selects the one with lowest priority.
    pub vendor: String,
}

impl Default for AuthNResolverConfig {
    fn default() -> Self {
        Self {
            vendor: "hyperspot".to_owned(),
        }
    }
}
