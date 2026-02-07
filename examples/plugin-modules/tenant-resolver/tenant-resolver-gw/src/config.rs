use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TenantResolverConfig {
    /// Vendor selector used by the gateway to pick a plugin implementation.
    ///
    /// Example values: `Contoso_Inc`, `Fabrikam`.
    pub vendor: String,
}

impl Default for TenantResolverConfig {
    fn default() -> Self {
        Self {
            vendor: "Contoso_Inc".to_owned(),
        }
    }
}
