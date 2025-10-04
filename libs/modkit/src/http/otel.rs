//! OpenTelemetry helper functions for trace context extraction and injection
//!
//! This module provides utilities for working with W3C Trace Context headers
//! in HTTP requests and responses. Due to OpenTelemetry ecosystem complexity,
//! this currently uses the simple_otel module for basic trace propagation.

use super::simple_otel;
use http::HeaderMap;

/// Extract parent trace context from HTTP headers using W3C Trace Context format.
/// Returns a trace parent string if found.
pub fn extract_parent_from_headers(headers: &HeaderMap) -> Option<String> {
    simple_otel::extract_trace_parent(headers)
}

/// Inject trace context into HTTP headers using W3C Trace Context format.
/// This is a simplified implementation that generates a basic traceparent header.
pub fn inject_context_into_headers(headers: &mut HeaderMap, span: &tracing::Span) {
    simple_otel::inject_trace_context(headers, span);
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderMap;
    use tracing::info_span;

    #[test]
    fn test_extract_empty_headers_returns_none() {
        let headers = HeaderMap::new();
        let parent = extract_parent_from_headers(&headers);
        assert!(parent.is_none());
    }

    #[test]
    fn test_extract_trace_parent() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "traceparent",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
                .parse()
                .unwrap(),
        );

        let parent = extract_parent_from_headers(&headers);
        assert!(parent.is_some());
        assert_eq!(
            parent.unwrap(),
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
        );
    }

    #[test]
    fn test_inject_context_into_headers() {
        let mut headers = HeaderMap::new();
        let span = info_span!("test");

        inject_context_into_headers(&mut headers, &span);

        assert!(headers.contains_key("traceparent"));
        let header = headers.get("traceparent").unwrap().to_str().unwrap();
        assert!(header.starts_with("00-"));
        assert_eq!(header.matches('-').count(), 3); // Should have 3 dashes
    }

    #[test]
    fn test_headers_with_invalid_traceparent() {
        let mut headers = HeaderMap::new();
        headers.insert("traceparent", "invalid-trace-context".parse().unwrap());

        let parent = extract_parent_from_headers(&headers);
        // Should gracefully handle invalid trace context
        assert!(parent.is_some()); // Returns the invalid string as-is
    }
}
