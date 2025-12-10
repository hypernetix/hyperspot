//! OpenTelemetry trace context helpers for HTTP headers
//!
//! Provides W3C Trace Context propagation with optional OpenTelemetry integration.
//! - With `otel` feature: Uses proper OTEL propagators for distributed tracing
//! - Without `otel` feature: No-op implementations (graceful degradation)

use http::HeaderMap;

/// W3C Trace Context header name
pub const TRACEPARENT: &str = "traceparent";

// ========================================
// Shared helpers (available in all builds)
// ========================================

/// Extract traceparent header value from HTTP headers
pub fn get_traceparent(headers: &HeaderMap) -> Option<&str> {
    headers.get(TRACEPARENT)?.to_str().ok()
}

/// Parse trace ID from W3C traceparent header (format: "00-{trace_id}-{span_id}-{flags}")
pub fn parse_trace_id(traceparent: &str) -> Option<String> {
    let parts: Vec<&str> = traceparent.split('-').collect();
    if parts.len() >= 4 && parts[0] == "00" {
        Some(parts[1].to_string())
    } else {
        None
    }
}

// ========================================
// OpenTelemetry integration (feature-gated)
// ========================================

#[cfg(feature = "otel")]
mod imp {
    use super::{get_traceparent, parse_trace_id};
    use http::{HeaderMap, HeaderName, HeaderValue};
    use opentelemetry::{
        global,
        propagation::{Extractor, Injector},
        Context,
    };
    use tracing::Span;
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    /// Adapter for extracting W3C Trace Context from HTTP headers
    struct HeadersExtractor<'a>(&'a HeaderMap);

    impl<'a> Extractor for HeadersExtractor<'a> {
        fn get(&self, key: &str) -> Option<&str> {
            self.0.get(key).and_then(|v| v.to_str().ok())
        }

        fn keys(&self) -> Vec<&str> {
            self.0.keys().map(|k| k.as_str()).collect()
        }
    }

    /// Adapter for injecting W3C Trace Context into HTTP headers
    struct HeadersInjector<'a>(&'a mut HeaderMap);

    impl<'a> Injector for HeadersInjector<'a> {
        fn set(&mut self, key: &str, value: String) {
            if let Ok(name) = HeaderName::from_bytes(key.as_bytes()) {
                if let Ok(val) = HeaderValue::from_str(&value) {
                    self.0.insert(name, val);
                }
            }
        }
    }

    /// Inject current OpenTelemetry context into HTTP headers.
    /// Uses the global propagator to inject W3C Trace Context.
    pub fn inject_current_span(headers: &mut HeaderMap) {
        let cx = Context::current();
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut HeadersInjector(headers));
        });
    }

    /// Set span parent from W3C Trace Context headers.
    /// Extracts the trace context and sets it as the parent of the given span.
    pub fn set_parent_from_headers(span: &Span, headers: &HeaderMap) {
        // Extract parent context using OTEL propagator
        let parent_cx = global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeadersExtractor(headers))
        });

        // Set as parent of current span
        let _ = span.set_parent(parent_cx);

        // Also record trace IDs for log correlation
        if let Some(traceparent) = get_traceparent(headers) {
            if let Some(trace_id) = parse_trace_id(traceparent) {
                span.record("trace_id", &trace_id);
                span.record("parent.trace_id", &trace_id);
            }
        }
    }
}

#[cfg(not(feature = "otel"))]
mod imp {
    use super::{get_traceparent, parse_trace_id};
    use http::HeaderMap;
    use tracing::Span;

    /// No-op: OpenTelemetry is disabled
    pub fn inject_current_span(_headers: &mut HeaderMap) {
        // No-op when OTEL is disabled
    }

    /// No-op: OpenTelemetry is disabled
    /// Records trace IDs if present in headers for log correlation only.
    pub fn set_parent_from_headers(span: &Span, headers: &HeaderMap) {
        // Without OTEL, just record trace IDs for log correlation if present
        if let Some(traceparent) = get_traceparent(headers) {
            if let Some(trace_id) = parse_trace_id(traceparent) {
                span.record("trace_id", &trace_id);
                span.record("parent.trace_id", &trace_id);
            }
        }
    }
}

// ========================================
// Public API
// ========================================

pub use imp::{inject_current_span, set_parent_from_headers};

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use tracing::info_span;

    #[test]
    fn test_get_traceparent_none() {
        let headers = HeaderMap::new();
        assert!(get_traceparent(&headers).is_none());
    }

    #[test]
    fn test_get_traceparent_ok() {
        let mut headers = HeaderMap::new();
        headers.insert(
            TRACEPARENT,
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
                .parse()
                .unwrap(),
        );

        let tp = get_traceparent(&headers);
        assert!(tp.is_some());
        assert_eq!(
            tp.unwrap(),
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
        );
    }

    #[test]
    fn test_parse_trace_id_ok() {
        let traceparent = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let trace_id = parse_trace_id(traceparent);
        assert_eq!(
            trace_id,
            Some("4bf92f3577b34da6a3ce929d0e0e4736".to_string())
        );
    }

    #[test]
    fn test_parse_trace_id_invalid() {
        assert!(parse_trace_id("invalid").is_none());
        assert!(parse_trace_id("").is_none());
    }

    #[test]
    #[cfg(not(feature = "otel"))]
    fn test_inject_current_span_noop() {
        let mut headers = HeaderMap::new();
        inject_current_span(&mut headers);
        // Should be no-op, no headers added
        assert!(headers.is_empty());
    }

    #[test]
    #[cfg(feature = "otel")]
    fn test_inject_current_span_no_panic() {
        use opentelemetry::global;
        use opentelemetry_sdk::propagation::TraceContextPropagator;

        global::set_text_map_propagator(TraceContextPropagator::new());

        let mut headers = http::HeaderMap::new();
        let _span = tracing::info_span!("test").entered();
        // Without full OTEL setup, this may not inject anything, but shouldn't panic
        inject_current_span(&mut headers);
    }

    #[test]
    fn test_set_parent_from_headers_no_panic() {
        let mut headers = HeaderMap::new();
        headers.insert(
            TRACEPARENT,
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
                .parse()
                .unwrap(),
        );

        let span = info_span!(
            "test",
            trace_id = tracing::field::Empty,
            parent.trace_id = tracing::field::Empty
        );

        // Should not panic in either mode
        set_parent_from_headers(&span, &headers);
    }
}
