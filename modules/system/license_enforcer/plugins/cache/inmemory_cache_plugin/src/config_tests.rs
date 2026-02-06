//! Tests for configuration parsing.

#[cfg(test)]
mod tests {
    use crate::config::InMemoryCachePluginConfig;
    use std::time::Duration;

    #[test]
    fn test_config_default() {
        let config = InMemoryCachePluginConfig::default();
        assert_eq!(config.vendor, "hyperspot");
        assert_eq!(config.priority, 100);
        assert_eq!(config.ttl, Duration::from_secs(60));
        assert_eq!(config.max_entries, 10_000);
    }

    #[test]
    fn test_config_parse_ttl_from_string() {
        let yaml = r#"
vendor: "test-vendor"
priority: 50
ttl: "5m"
max_entries: 1000
"#;
        let config: InMemoryCachePluginConfig = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(config.vendor, "test-vendor");
        assert_eq!(config.priority, 50);
        assert_eq!(config.ttl, Duration::from_secs(300)); // 5 minutes
        assert_eq!(config.max_entries, 1000);
    }

    #[test]
    fn test_config_parse_ttl_seconds() {
        let yaml = r#"
ttl: "15s"
"#;
        let config: InMemoryCachePluginConfig = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(config.ttl, Duration::from_secs(15));
    }

    #[test]
    fn test_config_reject_unknown_fields() {
        let yaml = r#"
vendor: "test"
unknown_field: "should fail"
"#;
        let result: Result<InMemoryCachePluginConfig, _> = serde_saphyr::from_str(yaml);
        assert!(
            result.is_err(),
            "Config should reject unknown fields due to deny_unknown_fields"
        );
    }

    #[test]
    fn test_config_applies_defaults() {
        let yaml = r#"
vendor: "custom-vendor"
"#;
        let config: InMemoryCachePluginConfig = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(config.vendor, "custom-vendor");
        assert_eq!(config.priority, 100); // default
        assert_eq!(config.ttl, Duration::from_secs(60)); // default
        assert_eq!(config.max_entries, 10_000); // default
    }
}
