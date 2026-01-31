//! Configuration for license enforcer gateway.

use serde::{Deserialize, Serialize};

/// License enforcer gateway configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LicenseEnforcerGatewayConfig {
    /// Vendor identifier for plugin selection.
    ///
    /// The gateway will select plugins from this vendor when multiple
    /// implementations are available.
    #[serde(default = "default_vendor")]
    pub vendor: String,
}

fn default_vendor() -> String {
    "hyperspot".to_owned()
}

impl Default for LicenseEnforcerGatewayConfig {
    fn default() -> Self {
        Self {
            vendor: default_vendor(),
        }
    }
}
