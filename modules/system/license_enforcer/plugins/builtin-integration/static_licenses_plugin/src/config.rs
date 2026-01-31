//! Configuration for static licenses plugin.

use serde::{Deserialize, Serialize};

/// Static licenses plugin configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StaticLicensesPluginConfig {
    /// Vendor identifier for this plugin instance.
    #[serde(default = "default_vendor")]
    pub vendor: String,

    /// Priority for plugin selection (lower = higher priority).
    #[serde(default = "default_priority")]
    pub priority: i16,

    /// List of enabled global features (`HyperSpot` GTS IDs).
    ///
    /// The plugin will return these features in addition to the base feature.
    /// This field is REQUIRED. An empty list is valid (returns base feature only).
    ///
    /// **Type Note**: Uses `Vec<String>` instead of `Vec<LicenseFeatureID>` because
    /// the SDK models are intentionally transport-agnostic (no serde derives).
    /// GTS ID format validation (using the `gts` crate's `GtsID::is_valid()`) is
    /// performed during module initialization. This provides proper structure
    /// validation without requiring registry validation at config-load time.
    pub static_licenses_features: Vec<String>,
}

fn default_vendor() -> String {
    "hyperspot".to_owned()
}

fn default_priority() -> i16 {
    100
}

impl Default for StaticLicensesPluginConfig {
    fn default() -> Self {
        Self {
            vendor: default_vendor(),
            priority: default_priority(),
            // Empty list is a valid default for testing/bootstrap
            static_licenses_features: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_with_features() {
        let config_str = r#"
            {
                "static_licenses_features": [
                    "gts.x.core.lic.feat.v1~x.core.global.advanced_analytics.v1",
                    "gts.x.core.lic.feat.v1~x.core.global.export.v1"
                ]
            }
        "#;

        let cfg: StaticLicensesPluginConfig = serde_json::from_str(config_str).unwrap();
        assert_eq!(cfg.vendor, "hyperspot");
        assert_eq!(cfg.priority, 100);
        assert_eq!(cfg.static_licenses_features.len(), 2);
        assert_eq!(
            cfg.static_licenses_features[0],
            "gts.x.core.lic.feat.v1~x.core.global.advanced_analytics.v1"
        );
        assert_eq!(
            cfg.static_licenses_features[1],
            "gts.x.core.lic.feat.v1~x.core.global.export.v1"
        );
    }

    #[test]
    fn test_config_with_empty_features() {
        let config_str = r#"
            {
                "static_licenses_features": []
            }
        "#;

        let cfg: StaticLicensesPluginConfig = serde_json::from_str(config_str).unwrap();
        assert_eq!(cfg.vendor, "hyperspot");
        assert_eq!(cfg.priority, 100);
        assert!(cfg.static_licenses_features.is_empty());
    }

    #[test]
    fn test_config_with_custom_vendor_and_priority() {
        let config_str = r#"
            {
                "vendor": "custom-vendor",
                "priority": 50,
                "static_licenses_features": ["gts.x.core.lic.feat.v1~x.core.global.test.v1"]
            }
        "#;

        let cfg: StaticLicensesPluginConfig = serde_json::from_str(config_str).unwrap();
        assert_eq!(cfg.vendor, "custom-vendor");
        assert_eq!(cfg.priority, 50);
        assert_eq!(cfg.static_licenses_features.len(), 1);
        assert_eq!(
            cfg.static_licenses_features[0],
            "gts.x.core.lic.feat.v1~x.core.global.test.v1"
        );
    }

    #[test]
    fn test_config_missing_features_fails() {
        let config_str = r#"
            {
                "vendor": "hyperspot",
                "priority": 100
            }
        "#;

        let result: Result<StaticLicensesPluginConfig, _> = serde_json::from_str(config_str);
        assert!(
            result.is_err(),
            "Config without static_licenses_features should fail"
        );

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("static_licenses_features") || err_msg.contains("missing field"),
            "Error should mention missing field: {err_msg}"
        );
    }

    #[test]
    fn test_config_allows_non_gts_strings_at_parse_time() {
        // Config parsing only does structure validation, not GTS format validation.
        // Format validation happens at module init time.
        let config_str = r#"
            {
                "static_licenses_features": ["invalid-id-without-gts-prefix"]
            }
        "#;

        let cfg: StaticLicensesPluginConfig = serde_json::from_str(config_str).unwrap();
        assert_eq!(cfg.static_licenses_features.len(), 1);
        assert_eq!(
            cfg.static_licenses_features[0],
            "invalid-id-without-gts-prefix"
        );
    }
}
