//! Simple OpenTelemetry helpers that work with current ecosystem
//!
//! This module provides basic trace context propagation using manual header
//! manipulation to avoid version conflicts in the OpenTelemetry ecosystem.

use http::{HeaderMap, HeaderName, HeaderValue};
use tracing::Span;

/// W3C Trace Context header name
pub const TRACEPARENT: &str = "traceparent";

/// Extract trace information from headers and return a span that can be used as parent
pub fn extract_trace_parent(headers: &HeaderMap) -> Option<String> {
    headers
        .get(TRACEPARENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Inject current trace context into headers (simplified approach)
pub fn inject_trace_context(headers: &mut HeaderMap, _span: &Span) {
    // For now, we'll create a simple traceparent header
    // In a production implementation, this would extract the actual trace context from the span
    let span_id = format!("{:016x}", rand::random::<u64>());
    let trace_id = format!("{:032x}", rand::random::<u128>());
    let flags = "01"; // sampled

    let traceparent = format!("00-{}-{}-{}", trace_id, span_id, flags);

    if let Ok(header_value) = HeaderValue::from_str(&traceparent) {
        headers.insert(HeaderName::from_static(TRACEPARENT), header_value);
    }
}

/// Set span as child of the trace context from headers
pub fn set_parent_from_headers(span: &Span, headers: &HeaderMap) {
    if let Some(traceparent) = extract_trace_parent(headers) {
        // Record the parent trace ID for correlation
        if let Some(trace_id) = parse_trace_id(&traceparent) {
            span.record("trace_id", &trace_id);
            span.record("parent.trace_id", &trace_id);
        }
    }
}

/// Parse trace ID from traceparent header
fn parse_trace_id(traceparent: &str) -> Option<String> {
    let parts: Vec<&str> = traceparent.split('-').collect();
    if parts.len() >= 4 && parts[0] == "00" {
        Some(parts[1].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::info_span;

    #[test]
    fn test_extract_trace_parent() {
        let mut headers = HeaderMap::new();
        headers.insert(
            TRACEPARENT,
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
                .parse()
                .unwrap(),
        );

        let parent = extract_trace_parent(&headers);
        assert!(parent.is_some());
        assert_eq!(
            parent.unwrap(),
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
        );
    }

    #[test]
    fn test_extract_trace_parent_missing() {
        let headers = HeaderMap::new();
        let parent = extract_trace_parent(&headers);
        assert!(parent.is_none());
    }

    #[test]
    fn test_inject_trace_context() {
        let mut headers = HeaderMap::new();
        let span = info_span!("test");

        inject_trace_context(&mut headers, &span);

        assert!(headers.contains_key(TRACEPARENT));
        let header = headers.get(TRACEPARENT).unwrap().to_str().unwrap();
        assert!(header.starts_with("00-"));
        assert_eq!(header.matches('-').count(), 3); // Should have 3 dashes
    }

    #[test]
    fn test_parse_trace_id() {
        let traceparent = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let trace_id = parse_trace_id(traceparent);
        assert_eq!(trace_id.unwrap(), "4bf92f3577b34da6a3ce929d0e0e4736");
    }

    #[test]
    fn test_parse_trace_id_invalid() {
        let traceparent = "invalid";
        let trace_id = parse_trace_id(traceparent);
        assert!(trace_id.is_none());
    }

    #[test]
    fn test_set_parent_from_headers() {
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
        set_parent_from_headers(&span, &headers);

        // The span should now have trace ID recorded
        // In a real implementation, we'd verify the recording worked
    }
}
