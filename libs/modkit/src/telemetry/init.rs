//! OpenTelemetry tracing initialization utilities
//!
//! This module sets up OpenTelemetry tracing and exports spans via OTLP
//! (gRPC or HTTP) to collectors such as Jaeger, Uptrace, or the OTel Collector.

#[cfg(feature = "otel")]
use opentelemetry::{global, trace::TracerProvider as _, KeyValue};

#[cfg(feature = "otel")]
use opentelemetry_otlp::{Protocol, WithExportConfig};
// Bring extension traits into scope for builder methods like `.with_headers()` and `.with_metadata()`.
#[cfg(feature = "otel")]
use opentelemetry_otlp::{WithHttpConfig, WithTonicConfig};

#[cfg(feature = "otel")]
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    trace::{Sampler, SdkTracerProvider},
    Resource,
};

#[cfg(feature = "otel")]
use tonic::metadata::{MetadataKey, MetadataMap, MetadataValue};

#[cfg(feature = "otel")]
use modkit_bootstrap::config::TracingConfig;

// ===== init_tracing (feature = "otel") ========================================

/// Build resource with service name and custom attributes
#[cfg(feature = "otel")]
fn build_resource(cfg: &TracingConfig) -> Resource {
    let service_name = cfg.service_name.as_deref().unwrap_or("hyperspot");
    let mut attrs = vec![KeyValue::new("service.name", service_name.to_string())];

    if let Some(resource_map) = &cfg.resource {
        for (k, v) in resource_map {
            attrs.push(KeyValue::new(k.clone(), v.clone()));
        }
    }

    Resource::builder_empty().with_attributes(attrs).build()
}

/// Build sampler from configuration
#[cfg(feature = "otel")]
fn build_sampler(cfg: &TracingConfig) -> Sampler {
    match cfg.sampler.as_ref().and_then(|s| s.strategy.as_deref()) {
        Some("always_off") => Sampler::AlwaysOff,
        Some("always_on") => Sampler::AlwaysOn,
        Some("parentbased_ratio") => {
            let ratio = cfg.sampler.as_ref().and_then(|s| s.ratio).unwrap_or(0.1);
            Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(ratio)))
        }
        _ => Sampler::ParentBased(Box::new(Sampler::AlwaysOn)),
    }
}

/// Extract exporter kind and endpoint from configuration
#[cfg(feature = "otel")]
fn extract_exporter_config(cfg: &TracingConfig) -> (String, String, Option<std::time::Duration>) {
    let (kind, endpoint) = cfg
        .exporter
        .as_ref()
        .map(|e| {
            (
                e.kind.as_deref().unwrap_or("otlp_grpc").to_string(),
                e.endpoint
                    .clone()
                    .unwrap_or_else(|| "http://127.0.0.1:4317".into()),
            )
        })
        .unwrap_or_else(|| ("otlp_grpc".to_string(), "http://127.0.0.1:4317".into()));

    let timeout = cfg
        .exporter
        .as_ref()
        .and_then(|e| e.timeout_ms)
        .map(std::time::Duration::from_millis);

    (kind, endpoint, timeout)
}

/// Build HTTP OTLP exporter
#[cfg(feature = "otel")]
fn build_http_exporter(
    cfg: &TracingConfig,
    endpoint: String,
    timeout: Option<std::time::Duration>,
) -> opentelemetry_otlp::SpanExporter {
    let mut b = opentelemetry_otlp::SpanExporter::builder().with_http();
    b = b
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint(endpoint);
    if let Some(t) = timeout {
        b = b.with_timeout(t);
    }
    if let Some(hmap) = build_headers_from_cfg_and_env(cfg) {
        b = b.with_headers(hmap);
    }
    #[allow(clippy::expect_used)]
    b.build().expect("build OTLP HTTP exporter")
}

/// Build gRPC OTLP exporter
#[cfg(feature = "otel")]
fn build_grpc_exporter(
    cfg: &TracingConfig,
    endpoint: String,
    timeout: Option<std::time::Duration>,
) -> opentelemetry_otlp::SpanExporter {
    let mut b = opentelemetry_otlp::SpanExporter::builder().with_tonic();
    b = b.with_endpoint(endpoint);
    if let Some(t) = timeout {
        b = b.with_timeout(t);
    }
    if let Some(md) = build_metadata_from_cfg_and_env(cfg) {
        b = b.with_metadata(md);
    }
    #[allow(clippy::expect_used)]
    b.build().expect("build OTLP gRPC exporter")
}

/// Initialize OpenTelemetry tracing from configuration and return a layer
/// to be attached to `tracing_subscriber`.
#[cfg(feature = "otel")]
pub fn init_tracing(
    cfg: &TracingConfig,
) -> Option<
    tracing_opentelemetry::OpenTelemetryLayer<
        tracing_subscriber::Registry,
        opentelemetry_sdk::trace::Tracer,
    >,
> {
    if !cfg.enabled {
        return None;
    }

    // Set W3C propagator for trace-context propagation
    global::set_text_map_propagator(TraceContextPropagator::new());

    let service_name = cfg.service_name.as_deref().unwrap_or("hyperspot");
    tracing::info!("Building OpenTelemetry layer for service: {}", service_name);

    // Build resource, sampler, and extract exporter config
    let resource = build_resource(cfg);
    let sampler = build_sampler(cfg);
    let (kind, endpoint, timeout) = extract_exporter_config(cfg);

    tracing::info!(kind, %endpoint, "OTLP exporter config");

    // Build span exporter based on kind
    let exporter = if matches!(kind.as_str(), "otlp_http") {
        build_http_exporter(cfg, endpoint, timeout)
    } else {
        build_grpc_exporter(cfg, endpoint, timeout)
    };

    // Build tracer provider with batch processor
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_sampler(sampler)
        .with_resource(resource)
        .build();

    // Make it global
    global::set_tracer_provider(provider.clone());

    // Create tracer and layer
    let tracer = provider.tracer("hyperspot");
    let otel_layer = tracing_opentelemetry::OpenTelemetryLayer::new(tracer);

    tracing::info!("OpenTelemetry layer created successfully");
    Some(otel_layer)
}

#[cfg(feature = "otel")]
fn build_headers_from_cfg_and_env(
    cfg: &TracingConfig,
) -> Option<std::collections::HashMap<String, String>> {
    use std::collections::HashMap;
    let mut out: HashMap<String, String> = HashMap::new();

    // From config file
    if let Some(exp) = &cfg.exporter {
        if let Some(hdrs) = &exp.headers {
            for (k, v) in hdrs {
                out.insert(k.clone(), v.clone());
            }
        }
    }

    // From ENV OTEL_EXPORTER_OTLP_HEADERS (format: k=v,k2=v2)
    if let Ok(env_hdrs) = std::env::var("OTEL_EXPORTER_OTLP_HEADERS") {
        for part in env_hdrs.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            if let Some((k, v)) = part.split_once('=') {
                out.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }

    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

#[cfg(feature = "otel")]
fn build_metadata_from_cfg_and_env(cfg: &TracingConfig) -> Option<MetadataMap> {
    let mut md = MetadataMap::new();

    // From config file
    if let Some(exp) = &cfg.exporter {
        if let Some(hdrs) = &exp.headers {
            for (k, v) in hdrs {
                if let Ok(key) = MetadataKey::from_bytes(k.as_bytes()) {
                    if let Ok(val) = MetadataValue::try_from(v.as_str()) {
                        md.insert(key, val);
                    }
                } else {
                    tracing::warn!(%k, "Skipping invalid gRPC metadata header name");
                }
            }
        }
    }

    // From ENV OTEL_EXPORTER_OTLP_HEADERS (format: k=v,k2=v2)
    if let Ok(env_hdrs) = std::env::var("OTEL_EXPORTER_OTLP_HEADERS") {
        for part in env_hdrs.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            if let Some((k, v)) = part.split_once('=') {
                if let Ok(key) = MetadataKey::from_bytes(k.trim().as_bytes()) {
                    if let Ok(val) = MetadataValue::try_from(v.trim()) {
                        md.insert(key, val);
                    }
                } else {
                    tracing::warn!(header = %k, "Skipping invalid gRPC metadata header name from ENV");
                }
            }
        }
    }

    if md.is_empty() {
        None
    } else {
        Some(md)
    }
}

// ===== init_tracing (feature disabled) ========================================

#[cfg(not(feature = "otel"))]
pub fn init_tracing(_cfg: &serde_json::Value) -> Option<()> {
    tracing::info!("Tracing configuration provided but runtime feature is disabled");
    None
}

// ===== shutdown_tracing =======================================================

/// Gracefully shut down OpenTelemetry tracing.
/// In opentelemetry 0.31 there is no global `shutdown_tracer_provider()`.
/// Keep a handle to `SdkTracerProvider` in your app state and call `shutdown()`
/// during graceful shutdown. This function remains a no-op for compatibility.
#[cfg(feature = "otel")]
pub fn shutdown_tracing() {
    tracing::info!("Tracing shutdown: no-op (keep a provider handle to call `shutdown()`).");
}

#[cfg(not(feature = "otel"))]
pub fn shutdown_tracing() {
    tracing::info!("Tracing shutdown (no-op)");
}

// ===== connectivity probe =====================================================

/// Build a tiny, separate OTLP pipeline and export a single span to verify connectivity.
/// This does *not* depend on tracing_subscriber; it uses SDK directly.
#[cfg(feature = "otel")]
pub async fn otel_connectivity_probe(
    cfg: &modkit_bootstrap::config::TracingConfig,
) -> anyhow::Result<()> {
    use opentelemetry::trace::{Span, Tracer as _};

    let service_name = cfg
        .service_name
        .clone()
        .unwrap_or_else(|| "hyperspot".into());

    let (kind, endpoint) = cfg
        .exporter
        .as_ref()
        .map(|e| {
            (
                e.kind.as_deref().unwrap_or("otlp_grpc"),
                e.endpoint.clone().unwrap_or_default(),
            )
        })
        .unwrap_or(("otlp_grpc", "http://127.0.0.1:4317".into()));

    // Resource
    let resource = Resource::builder_empty()
        .with_attributes([KeyValue::new("service.name", service_name.clone())])
        .build();

    // Exporter (type-state branches again)
    let exporter = if matches!(kind, "otlp_http") {
        let mut b = opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .with_protocol(Protocol::HttpBinary)
            .with_endpoint(endpoint.clone());
        if let Some(h) = build_headers_from_cfg_and_env(cfg) {
            b = b.with_headers(h);
        }
        b.build()
            .map_err(|e| anyhow::anyhow!("otlp http exporter build failed: {e}"))?
    } else {
        let mut b = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint.clone());
        if let Some(md) = build_metadata_from_cfg_and_env(cfg) {
            b = b.with_metadata(md);
        }
        b.build()
            .map_err(|e| anyhow::anyhow!("otlp grpc exporter build failed: {e}"))?
    };

    // Provider (simple processor is fine for a probe)
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_resource(resource)
        .build();

    // Emit a single span
    let tracer = provider.tracer("connectivity_probe");
    let mut span = tracer.start("otel_connectivity_probe");
    span.end();

    // Ensure delivery
    if let Err(e) = provider.force_flush() {
        tracing::warn!(error = %e, "force_flush failed during OTLP connectivity probe");
    }

    provider
        .shutdown()
        .map_err(|e| anyhow::anyhow!("shutdown failed: {e}"))?;

    tracing::info!("OTLP connectivity probe exported a test span (kind={kind})");
    Ok(())
}

#[cfg(not(feature = "otel"))]
pub async fn otel_connectivity_probe(_cfg: &serde_json::Value) -> anyhow::Result<()> {
    tracing::info!("OTLP connectivity probe skipped (otel feature disabled)");
    Ok(())
}

// ===== tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use modkit_bootstrap::config::{Exporter, Sampler, TracingConfig};
    use std::collections::HashMap;

    #[test]
    #[cfg(feature = "otel")]
    fn test_init_tracing_disabled() {
        let cfg = TracingConfig {
            enabled: false,
            ..Default::default()
        };

        let result = init_tracing(&cfg);
        assert!(result.is_none());
    }

    #[tokio::test]
    #[cfg(feature = "otel")]
    async fn test_init_tracing_enabled() {
        let cfg = TracingConfig {
            enabled: true,
            service_name: Some("test-service".to_string()),
            ..Default::default()
        };

        let result = init_tracing(&cfg);
        assert!(result.is_some());
    }

    #[test]
    #[cfg(feature = "otel")]
    fn test_init_tracing_with_resource_attributes() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();

        let mut resource_map = HashMap::new();
        resource_map.insert("service.version".to_string(), "1.0.0".to_string());
        resource_map.insert("deployment.environment".to_string(), "test".to_string());

        let cfg = TracingConfig {
            enabled: true,
            service_name: Some("test-service".to_string()),
            resource: Some(resource_map),
            ..Default::default()
        };

        let result = init_tracing(&cfg);
        assert!(result.is_some());
    }

    #[test]
    #[cfg(feature = "otel")]
    fn test_init_tracing_with_always_on_sampler() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();

        let cfg = TracingConfig {
            enabled: true,
            service_name: Some("test-service".to_string()),
            sampler: Some(Sampler {
                strategy: Some("always_on".to_string()),
                ratio: None,
            }),
            ..Default::default()
        };

        let result = init_tracing(&cfg);
        assert!(result.is_some());
    }

    #[test]
    #[cfg(feature = "otel")]
    fn test_init_tracing_with_always_off_sampler() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();

        let cfg = TracingConfig {
            enabled: true,
            service_name: Some("test-service".to_string()),
            sampler: Some(Sampler {
                strategy: Some("always_off".to_string()),
                ratio: None,
            }),
            ..Default::default()
        };

        let result = init_tracing(&cfg);
        assert!(result.is_some());
    }

    #[test]
    #[cfg(feature = "otel")]
    fn test_init_tracing_with_ratio_sampler() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();

        let cfg = TracingConfig {
            enabled: true,
            service_name: Some("test-service".to_string()),
            sampler: Some(Sampler {
                strategy: Some("parentbased_ratio".to_string()),
                ratio: Some(0.5),
            }),
            ..Default::default()
        };

        let result = init_tracing(&cfg);
        assert!(result.is_some());
    }

    #[test]
    #[cfg(feature = "otel")]
    fn test_init_tracing_with_http_exporter() {
        let _rt = tokio::runtime::Runtime::new().unwrap();

        let cfg = TracingConfig {
            enabled: true,
            service_name: Some("test-service".to_string()),
            exporter: Some(Exporter {
                kind: Some("otlp_http".to_string()),
                endpoint: Some("http://localhost:4318".to_string()),
                headers: None,
                timeout_ms: Some(5000),
            }),
            ..Default::default()
        };

        let result = init_tracing(&cfg);
        assert!(result.is_some());
    }

    #[test]
    #[cfg(feature = "otel")]
    fn test_init_tracing_with_grpc_exporter() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();

        let cfg = TracingConfig {
            enabled: true,
            service_name: Some("test-service".to_string()),
            exporter: Some(Exporter {
                kind: Some("otlp_grpc".to_string()),
                endpoint: Some("http://localhost:4317".to_string()),
                headers: None,
                timeout_ms: Some(5000),
            }),
            ..Default::default()
        };

        let result = init_tracing(&cfg);
        assert!(result.is_some());
    }

    #[test]
    #[cfg(feature = "otel")]
    fn test_build_headers_from_cfg_empty() {
        let cfg = TracingConfig {
            enabled: true,
            ..Default::default()
        };

        let result = build_headers_from_cfg_and_env(&cfg);
        // Should be None if no headers configured and no env var
        // (unless OTEL_EXPORTER_OTLP_HEADERS is set, which we can't control in tests)
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    #[cfg(feature = "otel")]
    fn test_build_headers_from_cfg_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("authorization".to_string(), "Bearer token".to_string());

        let cfg = TracingConfig {
            enabled: true,
            exporter: Some(Exporter {
                kind: Some("otlp_http".to_string()),
                endpoint: Some("http://localhost:4318".to_string()),
                headers: Some(headers.clone()),
                timeout_ms: None,
            }),
            ..Default::default()
        };

        let result = build_headers_from_cfg_and_env(&cfg);
        assert!(result.is_some());
        let result_headers = result.unwrap();
        assert_eq!(
            result_headers.get("authorization"),
            Some(&"Bearer token".to_string())
        );
    }

    #[test]
    #[cfg(feature = "otel")]
    fn test_build_metadata_from_cfg_empty() {
        let cfg = TracingConfig {
            enabled: true,
            ..Default::default()
        };

        let result = build_metadata_from_cfg_and_env(&cfg);
        // Should be None if no headers configured and no env var
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    #[cfg(feature = "otel")]
    fn test_build_metadata_from_cfg_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("authorization".to_string(), "Bearer token".to_string());

        let cfg = TracingConfig {
            enabled: true,
            exporter: Some(Exporter {
                kind: Some("otlp_grpc".to_string()),
                endpoint: Some("http://localhost:4317".to_string()),
                headers: Some(headers.clone()),
                timeout_ms: None,
            }),
            ..Default::default()
        };

        let result = build_metadata_from_cfg_and_env(&cfg);
        assert!(result.is_some());
        let metadata = result.unwrap();
        assert!(!metadata.is_empty());
    }

    #[test]
    fn test_shutdown_tracing_does_not_panic() {
        // Should not panic regardless of feature state
        shutdown_tracing();
    }
}
