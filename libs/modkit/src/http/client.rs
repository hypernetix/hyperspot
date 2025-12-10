//! Traced HTTP client that automatically injects OpenTelemetry context
//!
//! This module provides a wrapper around reqwest::Client that automatically
//! injects trace context headers (traceparent, tracestate) for distributed tracing.

use crate::http::otel;
use tracing::Level;

/// A traced HTTP client that automatically injects OpenTelemetry trace context
/// into outgoing requests for distributed tracing.
#[derive(Clone)]
pub struct TracedClient {
    inner: reqwest::Client,
}

impl TracedClient {
    /// Create a new TracedClient wrapping the provided reqwest::Client
    pub fn new(inner: reqwest::Client) -> Self {
        Self { inner }
    }

    /// Execute a built reqwest::Request, injecting trace headers from the current span.
    /// Creates a span for the outgoing HTTP request and injects the current trace context.
    pub async fn execute(&self, req: reqwest::Request) -> reqwest::Result<reqwest::Response> {
        let url = req.url().clone();
        let method = req.method().clone();

        let span = tracing::span!(
            Level::INFO, "outgoing_http",
            http.method = %method,
            http.url = %url,
            otel.kind = "client",
        );
        let _g = span.enter();

        // Inject trace context into headers using simplified approach
        let req = {
            let mut req = req.try_clone().unwrap_or(req);
            otel::inject_current_span(req.headers_mut());
            req
        };

        let response = self.inner.execute(req).await?;

        // Record response status in span
        span.record("http.status_code", response.status().as_u16());
        if response.status().is_client_error() || response.status().is_server_error() {
            span.record("error", true);
        }

        Ok(response)
    }

    /// Convenience method for GET requests
    pub async fn get(&self, url: &str) -> reqwest::Result<reqwest::Response> {
        let req = self.inner.get(url).build()?;
        self.execute(req).await
    }

    /// Convenience method for POST requests
    pub async fn post(&self, url: &str) -> reqwest::Result<reqwest::Response> {
        let req = self.inner.post(url).build()?;
        self.execute(req).await
    }

    /// Convenience method for PUT requests
    pub async fn put(&self, url: &str) -> reqwest::Result<reqwest::Response> {
        let req = self.inner.put(url).build()?;
        self.execute(req).await
    }

    /// Convenience method for PATCH requests
    pub async fn patch(&self, url: &str) -> reqwest::Result<reqwest::Response> {
        let req = self.inner.patch(url).build()?;
        self.execute(req).await
    }

    /// Convenience method for DELETE requests
    pub async fn delete(&self, url: &str) -> reqwest::Result<reqwest::Response> {
        let req = self.inner.delete(url).build()?;
        self.execute(req).await
    }

    /// Get a reference to the underlying reqwest::Client for advanced usage
    pub fn inner(&self) -> &reqwest::Client {
        &self.inner
    }

    /// Create a GET request builder
    pub fn request(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        self.inner.request(method, url)
    }
}

impl From<reqwest::Client> for TracedClient {
    fn from(c: reqwest::Client) -> Self {
        Self::new(c)
    }
}

impl Default for TracedClient {
    fn default() -> Self {
        Self::new(reqwest::Client::new())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    #[tokio::test]
    async fn test_traced_client_basic_functionality() {
        let client = TracedClient::default();

        // This test doesn't require a real server, just checks that the client can be created
        // and methods exist. We'll test actual tracing functionality with httpmock.
        assert!(client.inner().get("https://example.com").build().is_ok());
    }

    #[tokio::test]
    async fn test_traced_client_executes_request() {
        // Test that the client works without panicking (OTEL integration tested separately)
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET).path("/ping");
            then.status(200).body("ok");
        });

        let client = TracedClient::from(reqwest::Client::new());
        let url = format!("{}/ping", server.base_url());
        let resp = client.get(&url).await.unwrap();

        assert!(resp.status().is_success());
        // Note: With OTEL enabled, headers are injected if context is set up
        // Without OTEL, inject_current_span is a no-op
    }

    #[tokio::test]
    async fn test_all_http_methods() {
        let client = TracedClient::default();

        // Just verify all methods can be called (they'll fail without a server, but that's ok)
        let url = "http://example.com/test";

        // These will error due to no server, but we're just testing the API exists
        let _ = client.get(url).await;
        let _ = client.post(url).await;
        let _ = client.put(url).await;
        let _ = client.patch(url).await;
        let _ = client.delete(url).await;
    }
}
