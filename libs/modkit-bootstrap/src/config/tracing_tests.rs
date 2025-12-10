//! Tests for tracing configuration parsing

#[cfg(test)]
mod tests {
    use super::super::{Exporter, HttpOpts, LogsCorrelation, Propagation, Sampler, TracingConfig};
    use serde_saphyr;
    use std::collections::HashMap;

    #[test]
    fn test_parse_minimal_tracing_config() {
        let yaml = r#"
enabled: true
service_name: "test-service"
"#;
        let cfg: TracingConfig = serde_saphyr::from_str(yaml).unwrap();
        assert!(cfg.enabled);
        assert_eq!(cfg.service_name.as_deref(), Some("test-service"));
        assert!(cfg.exporter.is_none());
        assert!(cfg.sampler.is_none());
    }

    #[test]
    fn test_parse_full_tracing_config() {
        let yaml = r#"
enabled: true
service_name: "hyperspot-api"
exporter:
  kind: "otlp_grpc"
  endpoint: "http://127.0.0.1:4317"
  headers:
    authorization: "Bearer token123"
    x-custom: "value"
  timeout_ms: 5000
sampler:
  strategy: "parentbased_ratio"
  ratio: 0.2
propagation:
  w3c_trace_context: true
resource:
  service.version: "1.2.3"
  deployment.environment: "dev"
  service.namespace: "hyperspot"
http:
  inject_request_id_header: "x-request-id"
  record_headers: ["user-agent", "x-forwarded-for"]
logs_correlation:
  inject_trace_ids_into_logs: true
"#;
        let cfg: TracingConfig = serde_saphyr::from_str(yaml).unwrap();

        // Basic config
        assert!(cfg.enabled);
        assert_eq!(cfg.service_name.as_deref(), Some("hyperspot-api"));

        // Exporter config
        let exporter = cfg.exporter.as_ref().unwrap();
        assert_eq!(exporter.kind.as_deref(), Some("otlp_grpc"));
        assert_eq!(exporter.endpoint.as_deref(), Some("http://127.0.0.1:4317"));
        assert_eq!(exporter.timeout_ms, Some(5000));

        let headers = exporter.headers.as_ref().unwrap();
        assert_eq!(headers.get("authorization").unwrap(), "Bearer token123");
        assert_eq!(headers.get("x-custom").unwrap(), "value");

        // Sampler config
        let sampler = cfg.sampler.as_ref().unwrap();
        assert_eq!(sampler.strategy.as_deref(), Some("parentbased_ratio"));
        assert_eq!(sampler.ratio, Some(0.2));

        // Propagation config
        let propagation = cfg.propagation.as_ref().unwrap();
        assert_eq!(propagation.w3c_trace_context, Some(true));

        // Resource attributes
        let resource = cfg.resource.as_ref().unwrap();
        assert_eq!(resource.get("service.version").unwrap(), "1.2.3");
        assert_eq!(resource.get("deployment.environment").unwrap(), "dev");
        assert_eq!(resource.get("service.namespace").unwrap(), "hyperspot");

        // HTTP options
        let http = cfg.http.as_ref().unwrap();
        assert_eq!(
            http.inject_request_id_header.as_deref(),
            Some("x-request-id")
        );
        let headers = http.record_headers.as_ref().unwrap();
        assert_eq!(headers.len(), 2);
        assert!(headers.contains(&"user-agent".to_string()));
        assert!(headers.contains(&"x-forwarded-for".to_string()));

        // Logs correlation
        let logs = cfg.logs_correlation.as_ref().unwrap();
        assert_eq!(logs.inject_trace_ids_into_logs, Some(true));
    }

    #[test]
    fn test_parse_otlp_http_exporter() {
        let yaml = r#"
enabled: true
exporter:
  kind: "otlp_http"
  endpoint: "http://127.0.0.1:4318/v1/traces"
"#;
        let cfg: TracingConfig = serde_saphyr::from_str(yaml).unwrap();
        let exporter = cfg.exporter.as_ref().unwrap();
        assert_eq!(exporter.kind.as_deref(), Some("otlp_http"));
        assert_eq!(
            exporter.endpoint.as_deref(),
            Some("http://127.0.0.1:4318/v1/traces")
        );
    }

    #[test]
    fn test_parse_different_sampler_strategies() {
        // Test always_on
        let yaml = r#"
enabled: true
sampler:
  strategy: "always_on"
"#;
        let cfg: TracingConfig = serde_saphyr::from_str(yaml).unwrap();
        let sampler = cfg.sampler.as_ref().unwrap();
        assert_eq!(sampler.strategy.as_deref(), Some("always_on"));
        assert!(sampler.ratio.is_none());

        // Test always_off
        let yaml = r#"
enabled: true
sampler:
  strategy: "always_off"
"#;
        let cfg: TracingConfig = serde_saphyr::from_str(yaml).unwrap();
        let sampler = cfg.sampler.as_ref().unwrap();
        assert_eq!(sampler.strategy.as_deref(), Some("always_off"));

        // Test ratio
        let yaml = r#"
enabled: true
sampler:
  strategy: "ratio"
  ratio: 0.5
"#;
        let cfg: TracingConfig = serde_saphyr::from_str(yaml).unwrap();
        let sampler = cfg.sampler.as_ref().unwrap();
        assert_eq!(sampler.strategy.as_deref(), Some("ratio"));
        assert_eq!(sampler.ratio, Some(0.5));
    }

    #[test]
    fn test_disabled_tracing_config() {
        let yaml = r#"
enabled: false
service_name: "test-service"
exporter:
  kind: "otlp_grpc"
  endpoint: "http://127.0.0.1:4317"
"#;
        let cfg: TracingConfig = serde_saphyr::from_str(yaml).unwrap();
        assert!(!cfg.enabled);
        // Even when disabled, other config should still parse
        assert_eq!(cfg.service_name.as_deref(), Some("test-service"));
        assert!(cfg.exporter.is_some());
    }

    #[test]
    fn test_default_tracing_config() {
        let cfg = TracingConfig::default();
        assert!(!cfg.enabled); // Disabled by default
        assert!(cfg.service_name.is_none());
        assert!(cfg.exporter.is_none());
        assert!(cfg.sampler.is_none());
        assert!(cfg.propagation.is_none());
        assert!(cfg.resource.is_none());
        assert!(cfg.http.is_none());
        assert!(cfg.logs_correlation.is_none());
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let mut resource = HashMap::new();
        resource.insert("service.version".to_string(), "1.0.0".to_string());

        let mut headers = HashMap::new();
        headers.insert("auth".to_string(), "token".to_string());

        let original = TracingConfig {
            enabled: true,
            service_name: Some("test".to_string()),
            exporter: Some(Exporter {
                kind: Some("otlp_grpc".to_string()),
                endpoint: Some("http://localhost:4317".to_string()),
                headers: Some(headers),
                timeout_ms: Some(1000),
            }),
            sampler: Some(Sampler {
                strategy: Some("always_on".to_string()),
                ratio: None,
            }),
            propagation: Some(Propagation {
                w3c_trace_context: Some(true),
            }),
            resource: Some(resource),
            http: Some(HttpOpts {
                inject_request_id_header: Some("x-req-id".to_string()),
                record_headers: Some(vec!["user-agent".to_string()]),
            }),
            logs_correlation: Some(LogsCorrelation {
                inject_trace_ids_into_logs: Some(false),
            }),
        };

        // Serialize to YAML
        let yaml = serde_saphyr::to_string(&original).unwrap();

        // Deserialize back
        let roundtrip: TracingConfig = serde_saphyr::from_str(&yaml).unwrap();

        // Compare
        assert_eq!(original.enabled, roundtrip.enabled);
        assert_eq!(original.service_name, roundtrip.service_name);
        assert_eq!(
            original.exporter.as_ref().unwrap().kind,
            roundtrip.exporter.as_ref().unwrap().kind
        );
        assert_eq!(
            original.sampler.as_ref().unwrap().strategy,
            roundtrip.sampler.as_ref().unwrap().strategy
        );
        assert_eq!(
            original.propagation.as_ref().unwrap().w3c_trace_context,
            roundtrip.propagation.as_ref().unwrap().w3c_trace_context
        );
    }

    #[test]
    fn test_invalid_yaml_graceful_failure() {
        let invalid_yaml = r#"
enabled: true
sampler:
  strategy: "invalid_strategy"
  ratio: "not_a_number"
"#;

        // This should fail to parse due to invalid ratio type
        let result: Result<TracingConfig, _> = serde_saphyr::from_str(invalid_yaml);
        assert!(result.is_err());
    }
}
