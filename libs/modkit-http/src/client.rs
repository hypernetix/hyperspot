use crate::builder::HttpClientBuilder;
use crate::config::TransportSecurity;
use crate::error::HttpError;
use crate::request::RequestBuilder;
use crate::response::ResponseBody;
use bytes::Bytes;
use http::{Request, Response};
use http_body_util::Full;
use std::future::Future;
use std::pin::Pin;
use tower::Service;
use tower::buffer::Buffer;

/// Type alias for the future type of the inner service
pub type ServiceFuture =
    Pin<Box<dyn Future<Output = Result<Response<ResponseBody>, HttpError>> + Send>>;

/// Type alias for the buffered service
/// Buffer<Req, F> in tower 0.5 where Req is the request type and F is the service future type
pub type BufferedService = Buffer<Request<Full<Bytes>>, ServiceFuture>;

/// HTTP client with tower middleware stack
///
/// This client provides a clean interface over a tower service stack that includes:
/// - Timeout handling
/// - Automatic retries with exponential backoff
/// - User-Agent header injection
/// - Concurrency limiting (optional)
///
/// Use [`HttpClientBuilder`] to construct instances with custom configuration.
///
/// # Thread Safety
///
/// `HttpClient` is `Clone + Send + Sync`. Cloning is cheap (internal channel clone).
/// The client uses `tower::buffer::Buffer` internally, which allows true concurrent
/// access without any mutex serialization. Callers do NOT need to wrap `HttpClient`
/// in `Mutex` or `Arc<Mutex<_>>`.
///
/// # Example
///
/// ```ignore
/// // Just store the client directly - no Mutex needed!
/// struct MyService {
///     http: HttpClient,
/// }
///
/// impl MyService {
///     async fn fetch(&self) -> Result<Data, HttpError> {
///         // reqwest-like API: response has body-reading methods
///         self.http.get("https://example.com/api").await?.json().await
///     }
/// }
/// ```
#[derive(Clone)]
pub struct HttpClient {
    pub(crate) service: BufferedService,
    pub(crate) max_body_size: usize,
    pub(crate) transport_security: TransportSecurity,
}

impl HttpClient {
    /// Create a new HTTP client with default configuration
    ///
    /// # Errors
    /// Returns an error if TLS initialization fails
    pub fn new() -> Result<Self, HttpError> {
        HttpClientBuilder::new().build()
    }

    /// Create a builder for configuring the HTTP client
    #[must_use]
    pub fn builder() -> HttpClientBuilder {
        HttpClientBuilder::new()
    }

    /// Create a GET request builder
    ///
    /// Returns a [`RequestBuilder`] that can be configured with headers
    /// before sending with `.send().await`.
    ///
    /// # URL Requirements
    ///
    /// The URL must be an absolute URI with scheme and authority (host).
    /// Relative URLs like `/path` or `example.com/path` are rejected with
    /// [`HttpError::InvalidUri`].
    ///
    /// Valid examples:
    /// - `https://api.example.com/users`
    /// - `http://localhost:8080/health` (requires [`TransportSecurity::AllowInsecureHttp`])
    ///
    /// # URL Construction
    ///
    /// Query parameters must be encoded into the URL externally (e.g. via `url::Url`):
    ///
    /// ```ignore
    /// use url::Url;
    ///
    /// let mut url = Url::parse("https://api.example.com/search")?;
    /// url.query_pairs_mut().append_pair("q", "rust").append_pair("page", "1");
    ///
    /// let resp = client
    ///     .get(url.as_str())
    ///     .header("authorization", "Bearer token")
    ///     .send()
    ///     .await?;
    /// ```
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Simple GET
    /// let resp = client.get("https://api.example.com/data").send().await?;
    /// ```
    ///
    /// [`HttpError::InvalidUri`]: crate::error::HttpError::InvalidUri
    /// [`TransportSecurity::AllowInsecureHttp`]: crate::config::TransportSecurity::AllowInsecureHttp
    pub fn get(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(
            self.service.clone(),
            self.max_body_size,
            http::Method::GET,
            url.to_owned(),
            self.transport_security,
        )
    }

    /// Create a POST request builder
    ///
    /// Returns a [`RequestBuilder`] that can be configured with headers,
    /// body (JSON, form, bytes), etc. before sending with `.send().await`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // POST with JSON body
    /// let resp = client
    ///     .post("https://api.example.com/users")
    ///     .json(&NewUser { name: "Alice" })?
    ///     .send()
    ///     .await?;
    ///
    /// // POST with form body
    /// let resp = client
    ///     .post("https://auth.example.com/token")
    ///     .form(&[("grant_type", "client_credentials")])?
    ///     .send()
    ///     .await?;
    /// ```
    pub fn post(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(
            self.service.clone(),
            self.max_body_size,
            http::Method::POST,
            url.to_owned(),
            self.transport_security,
        )
    }

    /// Create a PUT request builder
    ///
    /// Returns a [`RequestBuilder`] that can be configured with headers,
    /// body (JSON, form, bytes), etc. before sending with `.send().await`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let resp = client
    ///     .put("https://api.example.com/resource/1")
    ///     .json(&UpdateData { value: 42 })?
    ///     .send()
    ///     .await?;
    /// ```
    pub fn put(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(
            self.service.clone(),
            self.max_body_size,
            http::Method::PUT,
            url.to_owned(),
            self.transport_security,
        )
    }

    /// Create a PATCH request builder
    ///
    /// Returns a [`RequestBuilder`] that can be configured with headers,
    /// body (JSON, form, bytes), etc. before sending with `.send().await`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let resp = client
    ///     .patch("https://api.example.com/resource/1")
    ///     .json(&PatchData { field: "new_value" })?
    ///     .send()
    ///     .await?;
    /// ```
    pub fn patch(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(
            self.service.clone(),
            self.max_body_size,
            http::Method::PATCH,
            url.to_owned(),
            self.transport_security,
        )
    }

    /// Create a DELETE request builder
    ///
    /// Returns a [`RequestBuilder`] that can be configured with headers
    /// before sending with `.send().await`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let resp = client
    ///     .delete("https://api.example.com/resource/42")
    ///     .header("authorization", "Bearer token")
    ///     .send()
    ///     .await?;
    /// ```
    pub fn delete(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(
            self.service.clone(),
            self.max_body_size,
            http::Method::DELETE,
            url.to_owned(),
            self.transport_security,
        )
    }
}

/// Map buffer errors to `HttpError`
///
/// Buffer can return `ServiceError` which wraps the inner service error,
/// or `Closed` if the buffer worker has shut down.
pub fn map_buffer_error(err: tower::BoxError) -> HttpError {
    // Try to downcast to HttpError (from inner service)
    match err.downcast::<HttpError>() {
        Ok(http_err) => *http_err,
        Err(err) => {
            // Buffer closed or other internal failure.
            // This happens when buffer worker panics or channel is dropped.
            //
            // Return ServiceClosed (not Overloaded) to distinguish from normal
            // overload (buffer full). This is a serious condition indicating
            // the background worker has died unexpectedly.
            tracing::error!(
                error = %err,
                "buffer worker closed unexpectedly; service unavailable"
            );
            HttpError::ServiceClosed
        }
    }
}

/// Try to acquire a buffer slot with fail-fast semantics.
///
/// If the buffer is full, returns `HttpError::Overloaded` immediately instead
/// of blocking. This prevents request pile-up under load.
pub async fn try_acquire_buffer_slot(service: &mut BufferedService) -> Result<(), HttpError> {
    use std::task::Poll;

    // Poll once to check if buffer has space available
    let poll_result = std::future::poll_fn(|cx| match service.poll_ready(cx) {
        Poll::Ready(result) => Poll::Ready(Some(result)),
        Poll::Pending => Poll::Ready(None), // Buffer full, don't block
    })
    .await;

    match poll_result {
        Some(Ok(())) => Ok(()),
        Some(Err(e)) => Err(map_buffer_error(e)),
        None => Err(HttpError::Overloaded), // Buffer full, fail fast
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::error::HttpError;
    use httpmock::prelude::*;
    use serde_json::json;

    fn test_client() -> HttpClient {
        HttpClientBuilder::new()
            .allow_insecure_http()
            .retry(None)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_http_client_get() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::GET).path("/test");
            then.status(200).json_body(json!({"success": true}));
        });

        let client = test_client();
        let url = format!("{}/test", server.base_url());
        let resp = client.get(&url).send().await.unwrap();

        assert_eq!(resp.status(), hyper::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_http_client_post() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::POST).path("/action");
            then.status(200).json_body(json!({"ok": true}));
        });

        let client = test_client();
        let url = format!("{}/action", server.base_url());
        let resp = client.post(&url).send().await.unwrap();

        assert_eq!(resp.status(), hyper::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_http_client_post_form() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::POST)
                .path("/submit")
                .header("content-type", "application/x-www-form-urlencoded")
                .body("key1=value1&key2=value2");
            then.status(200).json_body(json!({"received": true}));
        });

        let client = test_client();
        let url = format!("{}/submit", server.base_url());

        let resp = client
            .post(&url)
            .form(&[("key1", "value1"), ("key2", "value2")])
            .unwrap()
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), hyper::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_json_body_parsing() {
        #[derive(serde::Deserialize)]
        struct TestResponse {
            name: String,
            value: i32,
        }

        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::GET).path("/json");
            then.status(200)
                .json_body(json!({"name": "test", "value": 42}));
        });

        let client = test_client();
        let url = format!("{}/json", server.base_url());

        let data: TestResponse = client.get(&url).send().await.unwrap().json().await.unwrap();
        assert_eq!(data.name, "test");
        assert_eq!(data.value, 42);
    }

    #[tokio::test]
    async fn test_body_size_limit() {
        let server = MockServer::start();
        let large_body = "x".repeat(1024 * 1024); // 1MB
        let _m = server.mock(|when, then| {
            when.method(Method::GET).path("/large");
            then.status(200).body(&large_body);
        });

        let client = HttpClientBuilder::new()
            .allow_insecure_http()
            .retry(None)
            .max_body_size(1024) // 1KB limit
            .build()
            .unwrap();

        let url = format!("{}/large", server.base_url());
        let result = client.get(&url).send().await.unwrap().bytes().await;

        assert!(matches!(result, Err(HttpError::BodyTooLarge { .. })));
    }

    #[tokio::test]
    async fn test_custom_user_agent() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::GET)
                .path("/test")
                .header("user-agent", "custom/1.0");
            then.status(200);
        });

        let client = HttpClientBuilder::new()
            .allow_insecure_http()
            .retry(None)
            .user_agent("custom/1.0")
            .build()
            .unwrap();

        let url = format!("{}/test", server.base_url());
        let resp = client.get(&url).send().await.unwrap();
        assert_eq!(resp.status(), hyper::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_non_2xx_returns_http_status_error() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::GET).path("/error");
            then.status(404)
                .header("content-type", "application/json")
                .body(r#"{"error": "not found"}"#);
        });

        let client = test_client();
        let url = format!("{}/error", server.base_url());

        let result: Result<serde_json::Value, _> =
            client.get(&url).send().await.unwrap().json().await;
        match result {
            Err(HttpError::HttpStatus {
                status,
                body_preview,
                content_type,
                ..
            }) => {
                assert_eq!(status, hyper::StatusCode::NOT_FOUND);
                assert!(body_preview.contains("not found"));
                assert_eq!(content_type, Some("application/json".to_owned()));
            }
            other => panic!("Expected HttpStatus error, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_checked_body_success() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::GET).path("/data");
            then.status(200).body("hello world");
        });

        let client = test_client();
        let url = format!("{}/data", server.base_url());

        let body = client
            .get(&url)
            .send()
            .await
            .unwrap()
            .checked_bytes()
            .await
            .unwrap();
        assert_eq!(&body[..], b"hello world");
    }

    #[tokio::test]
    async fn test_client_is_clone() {
        let client = test_client();
        let client2 = client.clone();

        // Both should work independently
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::GET).path("/test");
            then.status(200);
        });

        let url = format!("{}/test", server.base_url());
        let resp1 = client.get(&url).send().await.unwrap();
        let resp2 = client2.get(&url).send().await.unwrap();

        assert_eq!(resp1.status(), hyper::StatusCode::OK);
        assert_eq!(resp2.status(), hyper::StatusCode::OK);
    }

    /// Compile-time assertion that `HttpClient` is `Send + Sync`
    ///
    /// This test ensures callers do NOT need to wrap `HttpClient` in `Mutex`.
    #[test]
    fn test_http_client_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<HttpClient>();
    }

    /// Test that 50 concurrent requests all succeed
    #[tokio::test]
    async fn test_concurrent_requests_50() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::GET).path("/concurrent");
            then.status(200).body("ok");
        });

        let client = test_client();
        let url = format!("{}/concurrent", server.base_url());

        // Spawn 50 concurrent requests
        let handles: Vec<_> = (0..50)
            .map(|_| {
                let client = client.clone();
                let url = url.clone();
                tokio::spawn(async move { client.get(&url).send().await })
            })
            .collect();

        // All should succeed
        for handle in handles {
            let resp = handle.await.unwrap().unwrap();
            assert_eq!(resp.status(), hyper::StatusCode::OK);
        }
    }

    /// Test small buffer capacity with fail-fast behavior
    ///
    /// With fail-fast buffer semantics, some requests may fail with Overloaded
    /// when buffer is full. This test verifies:
    /// 1. No deadlock (all complete within timeout)
    /// 2. At least some requests succeed
    /// 3. Failed requests get Overloaded error (not other errors)
    #[tokio::test]
    async fn test_small_buffer_capacity_no_deadlock() {
        use crate::config::HttpClientConfig;

        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::GET).path("/test");
            then.status(200).body("ok");
        });

        // Create client with very small buffer (capacity 2)
        let config = HttpClientConfig {
            transport: crate::config::TransportSecurity::AllowInsecureHttp,
            retry: None,
            rate_limit: None,
            buffer_capacity: 2,
            ..Default::default()
        };

        let client = HttpClientBuilder::with_config(config).build().unwrap();
        let url = format!("{}/test", server.base_url());

        // Fire 10 concurrent requests - some may fail with Overloaded (fail-fast)
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let client = client.clone();
                let url = url.clone();
                tokio::spawn(async move { client.get(&url).send().await })
            })
            .collect();

        // All should complete (not hang) within timeout
        let timeout_result = tokio::time::timeout(std::time::Duration::from_secs(10), async {
            let mut results = Vec::new();
            for handle in handles {
                results.push(handle.await);
            }
            results
        })
        .await;

        let results = timeout_result.expect("requests should complete within timeout");

        let mut success_count = 0;
        let mut overloaded_count = 0;
        for result in results {
            match result.unwrap() {
                Ok(resp) => {
                    assert_eq!(resp.status(), hyper::StatusCode::OK);
                    success_count += 1;
                }
                Err(HttpError::Overloaded) => {
                    overloaded_count += 1;
                }
                Err(e) => panic!("unexpected error: {e:?}"),
            }
        }

        // At least some should succeed (buffer processes requests)
        assert!(success_count > 0, "at least one request should succeed");
        // Total should be 10
        assert_eq!(success_count + overloaded_count, 10);
    }

    /// Test buffer overflow returns Overloaded error immediately (fail-fast)
    ///
    /// Verifies that when buffer is full and inner service is blocked,
    /// new requests fail immediately with Overloaded instead of hanging.
    #[tokio::test]
    async fn test_buffer_overflow_returns_overloaded() {
        use crate::config::HttpClientConfig;

        let server = MockServer::start();

        let _m = server.mock(|when, then| {
            when.method(Method::GET).path("/slow");
            then.status(200).body("ok");
        });

        // Create client with buffer capacity of 1
        let config = HttpClientConfig {
            transport: crate::config::TransportSecurity::AllowInsecureHttp,
            retry: None,
            rate_limit: None,
            buffer_capacity: 1,
            ..Default::default()
        };

        let client = HttpClientBuilder::with_config(config).build().unwrap();
        let url = format!("{}/slow", server.base_url());

        // First request - will occupy the single buffer slot
        let client1 = client.clone();
        let url1 = url.clone();
        let handle1 = tokio::spawn(async move { client1.get(&url1).send().await });

        // Give first request time to acquire buffer slot
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Second request - should fail immediately with Overloaded (buffer full)
        let result2 = tokio::time::timeout(
            std::time::Duration::from_millis(50),
            client.get(&url).send(),
        )
        .await;

        // Should complete immediately (not timeout) with Overloaded
        let inner_result = result2.expect("request should not timeout waiting for buffer");
        match inner_result {
            // Expected: buffer full (fail-fast) or request got through (timing dependent)
            Err(HttpError::Overloaded) | Ok(_) => {}
            Err(e) => panic!("unexpected error: {e:?}"),
        }

        // Let first request complete
        _ = handle1.await;
    }

    /// Test that large body reading doesn't cause deadlock
    #[tokio::test]
    async fn test_large_body_no_deadlock() {
        let server = MockServer::start();
        let large_body = "x".repeat(100 * 1024); // 100KB
        let _m = server.mock(|when, then| {
            when.method(Method::GET).path("/large");
            then.status(200).body(&large_body);
        });

        let client = HttpClientBuilder::new()
            .allow_insecure_http()
            .retry(None)
            .max_body_size(1024 * 1024) // 1MB limit
            .build()
            .unwrap();

        let url = format!("{}/large", server.base_url());

        // Fire multiple concurrent requests that read large bodies
        let handles: Vec<_> = (0..5)
            .map(|_| {
                let client = client.clone();
                let url = url.clone();
                tokio::spawn(async move { client.get(&url).send().await?.checked_bytes().await })
            })
            .collect();

        // All should complete
        let timeout_result = tokio::time::timeout(std::time::Duration::from_secs(10), async {
            let mut results = Vec::new();
            for handle in handles {
                results.push(handle.await);
            }
            results
        })
        .await;

        let results = timeout_result.expect("body reads should complete within timeout");
        for result in results {
            let body = result.unwrap().unwrap();
            assert_eq!(body.len(), 100 * 1024);
        }
    }

    /// Test that `token_endpoint` config does NOT retry POST requests
    ///
    /// `OAuth2` token endpoints use POST, and we must not retry POST to avoid
    /// duplicate token requests. This test verifies the retry config in
    /// `HttpClientConfig::token_endpoint()` only retries GET.
    #[tokio::test]
    async fn test_token_endpoint_post_not_retried() {
        use crate::config::HttpClientConfig;

        let server = MockServer::start();

        // Mock that always returns 500 (retriable error)
        let mock = server.mock(|when, then| {
            when.method(Method::POST).path("/token");
            then.status(500).body("server error");
        });

        // Use token_endpoint config (retry enabled but only for GET)
        let mut config = HttpClientConfig::token_endpoint();
        config.transport = crate::config::TransportSecurity::AllowInsecureHttp; // Allow HTTP for test server

        let client = HttpClientBuilder::with_config(config).build().unwrap();
        let url = format!("{}/token", server.base_url());

        // POST form to token endpoint
        let result = client
            .post(&url)
            .form(&[("grant_type", "client_credentials"), ("client_id", "test")])
            .unwrap()
            .send()
            .await;

        // Request should fail (500 error)
        assert!(result.is_ok()); // HTTP request succeeded
        let response = result.unwrap();
        assert_eq!(response.status(), hyper::StatusCode::INTERNAL_SERVER_ERROR);

        // Verify mock was called exactly once (no retries)
        // httpmock tracks calls internally
        assert_eq!(
            mock.calls(),
            1,
            "POST should not be retried; expected 1 call, got {}",
            mock.calls()
        );
    }

    // NOTE: GET retry behavior is tested at the layer level in
    // `layers::tests::test_retry_layer_retries_transport_errors` which uses
    // a mock service to simulate transport errors. HTTP status codes (like 500)
    // don't trigger retries since they're returned as Ok(Response), not Err.

    #[tokio::test]
    async fn test_http_client_put() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::PUT).path("/resource");
            then.status(200).json_body(json!({"updated": true}));
        });

        let client = test_client();
        let url = format!("{}/resource", server.base_url());
        let resp = client.put(&url).send().await.unwrap();

        assert_eq!(resp.status(), hyper::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_http_client_put_form() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::PUT)
                .path("/resource")
                .header("content-type", "application/x-www-form-urlencoded")
                .body("name=updated&value=123");
            then.status(200).json_body(json!({"updated": true}));
        });

        let client = test_client();
        let url = format!("{}/resource", server.base_url());

        let resp = client
            .put(&url)
            .form(&[("name", "updated"), ("value", "123")])
            .unwrap()
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), hyper::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_http_client_patch() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::PATCH).path("/resource/1");
            then.status(200).json_body(json!({"patched": true}));
        });

        let client = test_client();
        let url = format!("{}/resource/1", server.base_url());
        let resp = client.patch(&url).send().await.unwrap();

        assert_eq!(resp.status(), hyper::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_http_client_patch_form() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::PATCH)
                .path("/resource/1")
                .header("content-type", "application/x-www-form-urlencoded")
                .body("field=patched");
            then.status(200).json_body(json!({"patched": true}));
        });

        let client = test_client();
        let url = format!("{}/resource/1", server.base_url());

        let resp = client
            .patch(&url)
            .form(&[("field", "patched")])
            .unwrap()
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), hyper::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_http_client_delete() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::DELETE).path("/resource/42");
            then.status(204);
        });

        let client = test_client();
        let url = format!("{}/resource/42", server.base_url());
        let resp = client.delete(&url).send().await.unwrap();

        assert_eq!(resp.status(), hyper::StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_http_client_delete_returns_200() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::DELETE).path("/resource/99");
            then.status(200).json_body(json!({"deleted": true}));
        });

        let client = test_client();
        let url = format!("{}/resource/99", server.base_url());
        let resp = client.delete(&url).send().await.unwrap();

        assert_eq!(resp.status(), hyper::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_put_form_with_custom_headers() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::PUT)
                .path("/api/data")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("x-custom-header", "custom-value")
                .body("key=value");
            then.status(200);
        });

        let client = test_client();
        let url = format!("{}/api/data", server.base_url());

        let resp = client
            .put(&url)
            .header("x-custom-header", "custom-value")
            .form(&[("key", "value")])
            .unwrap()
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), hyper::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_patch_form_with_custom_headers() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::PATCH)
                .path("/api/data")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("authorization", "Bearer token123")
                .body("status=active");
            then.status(200);
        });

        let client = test_client();
        let url = format!("{}/api/data", server.base_url());

        let resp = client
            .patch(&url)
            .header("authorization", "Bearer token123")
            .form(&[("status", "active")])
            .unwrap()
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), hyper::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_request_builder_json_body() {
        #[derive(serde::Serialize)]
        struct CreateUser {
            name: String,
            email: String,
        }

        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::POST)
                .path("/users")
                .header("content-type", "application/json")
                .json_body(json!({"name": "Alice", "email": "alice@example.com"}));
            then.status(201).json_body(json!({"id": 1}));
        });

        let client = test_client();
        let url = format!("{}/users", server.base_url());

        let resp = client
            .post(&url)
            .json(&CreateUser {
                name: "Alice".into(),
                email: "alice@example.com".into(),
            })
            .unwrap()
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), hyper::StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_request_builder_body_bytes() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::POST)
                .path("/upload")
                .body("raw binary data");
            then.status(200);
        });

        let client = test_client();
        let url = format!("{}/upload", server.base_url());

        let resp = client
            .post(&url)
            .body_bytes(bytes::Bytes::from("raw binary data"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), hyper::StatusCode::OK);
    }

    /// Test that user-provided Content-Type is not duplicated when using `json()`.
    ///
    /// When the user supplies a Content-Type header before calling `.json()`,
    /// the default `application/json` should NOT be added. The final request
    /// should have exactly one Content-Type header with the user's value.
    #[tokio::test]
    async fn test_content_type_not_duplicated_with_json() {
        #[derive(serde::Serialize)]
        struct TestData {
            value: i32,
        }

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(Method::POST)
                .path("/custom-content-type")
                // Match the custom Content-Type (not application/json)
                .header("content-type", "application/vnd.custom+json");
            then.status(200);
        });

        let client = test_client();
        let url = format!("{}/custom-content-type", server.base_url());

        let resp = client
            .post(&url)
            .header("content-type", "application/vnd.custom+json") // Custom Content-Type
            .json(&TestData { value: 42 })
            .unwrap()
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), hyper::StatusCode::OK);
        assert_eq!(
            mock.calls(),
            1,
            "Request with custom Content-Type should match"
        );
    }

    /// Test that user-provided Content-Type is not duplicated when using `form()`.
    #[tokio::test]
    async fn test_content_type_not_duplicated_with_form() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(Method::POST)
                .path("/custom-form-type")
                // Match the custom Content-Type (not application/x-www-form-urlencoded)
                .header("content-type", "application/x-custom-form");
            then.status(200);
        });

        let client = test_client();
        let url = format!("{}/custom-form-type", server.base_url());

        let resp = client
            .post(&url)
            .header("content-type", "application/x-custom-form") // Custom Content-Type
            .form(&[("key", "value")])
            .unwrap()
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), hyper::StatusCode::OK);
        assert_eq!(
            mock.calls(),
            1,
            "Request with custom Content-Type should match"
        );
    }

    #[tokio::test]
    async fn test_request_builder_body_string() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::POST)
                .path("/text")
                .body("Hello, World!");
            then.status(200);
        });

        let client = test_client();
        let url = format!("{}/text", server.base_url());

        let resp = client
            .post(&url)
            .body_string("Hello, World!".into())
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), hyper::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_response_text_method() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::GET).path("/text");
            then.status(200).body("Hello, World!");
        });

        let client = test_client();
        let url = format!("{}/text", server.base_url());

        let text = client.get(&url).send().await.unwrap().text().await.unwrap();
        assert_eq!(text, "Hello, World!");
    }

    #[tokio::test]
    async fn test_request_builder_multiple_headers() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::GET)
                .path("/headers")
                .header("x-first", "one")
                .header("x-second", "two");
            then.status(200);
        });

        let client = test_client();
        let url = format!("{}/headers", server.base_url());

        let resp = client
            .get(&url)
            .header("x-first", "one")
            .header("x-second", "two")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), hyper::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_request_builder_headers_vec() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::GET)
                .path("/headers")
                .header("x-first", "one")
                .header("x-second", "two");
            then.status(200);
        });

        let client = test_client();
        let url = format!("{}/headers", server.base_url());

        let resp = client
            .get(&url)
            .headers(vec![
                ("x-first".to_owned(), "one".to_owned()),
                ("x-second".to_owned(), "two".to_owned()),
            ])
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), hyper::StatusCode::OK);
    }

    /// Test that `checked_bytes` returns `HttpStatus` error (not `BodyTooLarge`) when
    /// a non-2xx response has a body larger than the preview limit.
    #[tokio::test]
    async fn test_error_response_with_large_body_returns_http_status() {
        use crate::security::ERROR_BODY_PREVIEW_LIMIT;

        let server = MockServer::start();

        // Create a body larger than ERROR_BODY_PREVIEW_LIMIT (8KB)
        let large_body = "x".repeat(ERROR_BODY_PREVIEW_LIMIT + 1000);

        let _m = server.mock(|when, then| {
            when.method(Method::GET).path("/error-with-large-body");
            then.status(500).body(&large_body);
        });

        let client = test_client();
        let url = format!("{}/error-with-large-body", server.base_url());

        let result = client.get(&url).send().await.unwrap().checked_bytes().await;

        // Should return HttpStatus error, NOT BodyTooLarge
        match result {
            Err(HttpError::HttpStatus {
                status,
                body_preview,
                ..
            }) => {
                assert_eq!(status, hyper::StatusCode::INTERNAL_SERVER_ERROR);
                // Body preview should indicate it was too large
                assert_eq!(body_preview, "<body too large for preview>");
            }
            Err(HttpError::BodyTooLarge { .. }) => {
                panic!("Should return HttpStatus, not BodyTooLarge for non-2xx responses");
            }
            Err(other) => panic!("Unexpected error: {other:?}"),
            Ok(_) => panic!("Should have returned an error for 500 status"),
        }
    }

    // ==========================================================================
    // Gzip/Br/Deflate Decompression Tests
    // ==========================================================================

    /// Helper to gzip-compress data
    fn gzip_compress(data: &[u8]) -> Vec<u8> {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap()
    }

    /// Test that gzip-encoded response is automatically decompressed.
    ///
    /// Server returns a gzip-compressed body with `Content-Encoding: gzip`.
    /// Client should automatically decompress and return the original bytes.
    #[tokio::test]
    async fn test_gzip_decompression_basic() {
        let server = MockServer::start();

        let original_body = b"Hello, this is a test body that will be gzip compressed!";
        let compressed_body = gzip_compress(original_body);

        let _m = server.mock(|when, then| {
            when.method(Method::GET).path("/gzip");
            then.status(200)
                .header("content-encoding", "gzip")
                .body(compressed_body);
        });

        let client = test_client();
        let url = format!("{}/gzip", server.base_url());

        let body = client
            .get(&url)
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();

        assert_eq!(
            body.as_ref(),
            original_body,
            "Decompressed body should match original"
        );
    }

    /// Test that gzip-compressed JSON can be parsed via `response.json()`.
    ///
    /// Server returns gzipped JSON with `Content-Encoding: gzip`.
    /// Client should decompress and successfully parse the JSON.
    #[tokio::test]
    async fn test_gzip_decompression_json() {
        #[derive(serde::Deserialize, PartialEq, Debug)]
        struct TestData {
            name: String,
            value: i32,
            nested: NestedData,
        }

        #[derive(serde::Deserialize, PartialEq, Debug)]
        struct NestedData {
            items: Vec<String>,
        }

        let server = MockServer::start();

        let json_body = r#"{"name":"test","value":42,"nested":{"items":["a","b","c"]}}"#;
        let compressed_body = gzip_compress(json_body.as_bytes());

        let _m = server.mock(|when, then| {
            when.method(Method::GET).path("/gzip-json");
            then.status(200)
                .header("content-type", "application/json")
                .header("content-encoding", "gzip")
                .body(compressed_body);
        });

        let client = test_client();
        let url = format!("{}/gzip-json", server.base_url());

        let data: TestData = client.get(&url).send().await.unwrap().json().await.unwrap();

        assert_eq!(data.name, "test");
        assert_eq!(data.value, 42);
        assert_eq!(data.nested.items, vec!["a", "b", "c"]);
    }

    /// Test that body size limit is enforced on DECOMPRESSED bytes, not compressed.
    ///
    /// This protects against "zip bombs" where a small compressed payload
    /// expands to a huge decompressed size.
    ///
    /// The test creates a highly compressible payload (repeated 'x' chars)
    /// that compresses small but expands beyond the `max_body_size` limit.
    #[tokio::test]
    async fn test_gzip_decompression_body_size_limit() {
        let server = MockServer::start();

        // Create a body that compresses well but is large when decompressed.
        // 100KB of repeated 'x' compresses to a few hundred bytes.
        let large_decompressed = vec![b'x'; 100 * 1024]; // 100KB
        let compressed_body = gzip_compress(&large_decompressed);

        // Verify compression is significant (sanity check)
        assert!(
            compressed_body.len() < 2000,
            "Compressed body should be small (got {} bytes)",
            compressed_body.len()
        );

        let _m = server.mock(|when, then| {
            when.method(Method::GET).path("/gzip-bomb");
            then.status(200)
                .header("content-encoding", "gzip")
                .body(compressed_body);
        });

        // Create client with 10KB body limit - smaller than decompressed size
        let client = HttpClientBuilder::new()
            .allow_insecure_http()
            .retry(None)
            .max_body_size(10 * 1024) // 10KB limit
            .build()
            .unwrap();

        let url = format!("{}/gzip-bomb", server.base_url());
        let result = client.get(&url).send().await.unwrap().bytes().await;

        // Should fail with BodyTooLarge because decompressed size exceeds limit
        match result {
            Err(HttpError::BodyTooLarge { limit, actual }) => {
                assert_eq!(limit, 10 * 1024, "Limit should be 10KB");
                assert!(
                    actual > limit,
                    "Actual size ({actual}) should exceed limit ({limit})"
                );
            }
            Err(other) => panic!("Expected BodyTooLarge error, got: {other:?}"),
            Ok(body) => panic!(
                "Expected BodyTooLarge error, but got {} bytes of body",
                body.len()
            ),
        }
    }

    /// Test that Accept-Encoding header is automatically set by the client.
    ///
    /// The `DecompressionLayer` automatically adds `Accept-Encoding: gzip, br, deflate`
    /// to outgoing requests.
    #[tokio::test]
    async fn test_accept_encoding_header_sent() {
        let server = MockServer::start();

        // Mock that requires Accept-Encoding header to be present
        let mock = server.mock(|when, then| {
            when.method(Method::GET)
                .path("/check-accept-encoding")
                .header_exists("accept-encoding");
            then.status(200).body("ok");
        });

        let client = test_client();
        let url = format!("{}/check-accept-encoding", server.base_url());

        let resp = client.get(&url).send().await.unwrap();
        assert_eq!(resp.status(), hyper::StatusCode::OK);

        // Verify the mock was hit (meaning Accept-Encoding was present)
        assert_eq!(
            mock.calls(),
            1,
            "Request should have included Accept-Encoding header"
        );
    }

    /// Test that non-compressed responses still work normally.
    ///
    /// When server doesn't return Content-Encoding, the body should pass through unchanged.
    #[tokio::test]
    async fn test_no_compression_passthrough() {
        let server = MockServer::start();

        let plain_body = b"This is plain text, not compressed";

        let _m = server.mock(|when, then| {
            when.method(Method::GET).path("/plain");
            then.status(200)
                .header("content-type", "text/plain")
                .body(plain_body.as_slice());
        });

        let client = test_client();
        let url = format!("{}/plain", server.base_url());

        let body = client
            .get(&url)
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();

        assert_eq!(
            body.as_ref(),
            plain_body,
            "Plain body should pass through unchanged"
        );
    }

    /// Test that `checked_bytes` works correctly with gzip decompression.
    #[tokio::test]
    async fn test_gzip_decompression_checked_bytes() {
        let server = MockServer::start();

        let original_body = b"Checked bytes test with gzip";
        let compressed_body = gzip_compress(original_body);

        let _m = server.mock(|when, then| {
            when.method(Method::GET).path("/gzip-checked");
            then.status(200)
                .header("content-encoding", "gzip")
                .body(compressed_body);
        });

        let client = test_client();
        let url = format!("{}/gzip-checked", server.base_url());

        let body = client
            .get(&url)
            .send()
            .await
            .unwrap()
            .checked_bytes()
            .await
            .unwrap();

        assert_eq!(
            body.as_ref(),
            original_body,
            "checked_bytes should return decompressed content"
        );
    }

    /// Test that `text()` method works correctly with gzip decompression.
    #[tokio::test]
    async fn test_gzip_decompression_text() {
        let server = MockServer::start();

        let original_text = "Hello, World! \u{1F600}"; // Contains emoji
        let compressed_body = gzip_compress(original_text.as_bytes());

        let _m = server.mock(|when, then| {
            when.method(Method::GET).path("/gzip-text");
            then.status(200)
                .header("content-type", "text/plain; charset=utf-8")
                .header("content-encoding", "gzip")
                .body(compressed_body);
        });

        let client = test_client();
        let url = format!("{}/gzip-text", server.base_url());

        let text = client.get(&url).send().await.unwrap().text().await.unwrap();

        assert_eq!(
            text, original_text,
            "text() should return decompressed UTF-8 content"
        );
    }

    // ==========================================================================
    // Buffer Error Mapping Tests
    // ==========================================================================

    /// Test that `map_buffer_error` returns inner `HttpError` when present.
    #[test]
    fn test_map_buffer_error_passes_through_http_error() {
        let http_err = HttpError::Timeout(std::time::Duration::from_secs(10));
        let boxed: tower::BoxError = Box::new(http_err);
        let result = map_buffer_error(boxed);

        assert!(
            matches!(result, HttpError::Timeout(_)),
            "Should pass through HttpError::Timeout, got: {result:?}"
        );
    }

    /// Test that `map_buffer_error` returns `ServiceClosed` for non-HttpError.
    ///
    /// This covers the case where buffer is closed or worker panicked.
    /// The error log is emitted (verified by code coverage, not assertion).
    #[test]
    fn test_map_buffer_error_returns_service_closed_for_unknown_error() {
        // Simulate a buffer closed error (any non-HttpError box)
        let other_err: tower::BoxError = Box::new(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "buffer worker died",
        ));
        let result = map_buffer_error(other_err);

        assert!(
            matches!(result, HttpError::ServiceClosed),
            "Should return ServiceClosed for non-HttpError, got: {result:?}"
        );
    }

    // ==========================================================================
    // Status-based Retry Integration Tests
    //
    // These tests verify that retry-on-status works END-TO-END with real HTTP
    // responses (not just mock services that return Err directly).
    //
    // Key insight: hyper returns Ok(Response) for all HTTP statuses.
    // RetryLayer handles retries on Ok(Response) by checking status codes,
    // then returns Ok(Response) with the final status after retries exhaust.
    // send() NEVER returns Err(HttpStatus) - that's only created by error_for_status().
    // ==========================================================================

    /// Test: GET request with 500 errors is retried.
    ///
    /// Server always returns 500. Asserts total calls == `max_retries` + 1.
    /// After retries exhaust, returns Ok(Response) with 500 status.
    #[tokio::test]
    async fn test_status_retry_get_500_retried() {
        use crate::config::{ExponentialBackoff, HttpClientConfig, RetryConfig};

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(Method::GET).path("/retry-500");
            then.status(500).body("server error");
        });

        let config = HttpClientConfig {
            transport: crate::config::TransportSecurity::AllowInsecureHttp,
            retry: Some(RetryConfig {
                max_retries: 2, // 1 initial + 2 retries = 3 total attempts
                backoff: ExponentialBackoff::fast(),
                ..RetryConfig::default() // 500 is in idempotent_retry
            }),
            rate_limit: None,
            ..Default::default()
        };

        let client = HttpClientBuilder::with_config(config).build().unwrap();
        let url = format!("{}/retry-500", server.base_url());

        let result = client.get(&url).send().await;

        // GET on 500 SHOULD be retried (GET is idempotent, 500 is in idempotent_retry)
        assert_eq!(
            mock.calls(),
            3,
            "GET should retry on 500; expected 3 calls (1 + 2 retries), got {}",
            mock.calls()
        );

        // After retries exhaust: returns Ok(Response) with 500 status
        let response = result.expect("send() should return Ok(Response) after retries exhaust");
        assert_eq!(response.status(), hyper::StatusCode::INTERNAL_SERVER_ERROR);

        // User can convert to error via error_for_status()
        let err = response.error_for_status().unwrap_err();
        assert!(
            matches!(err, HttpError::HttpStatus { status, .. } if status == hyper::StatusCode::INTERNAL_SERVER_ERROR)
        );
    }

    /// Test: POST request with 500 is NOT retried and returns Ok(Response).
    ///
    /// With default retry config, 500 is only retried for idempotent methods.
    /// POST is not idempotent, so:
    /// 1. No retry (calls == 1)
    /// 2. Returns Ok(Response) with status 500 (not converted to Err)
    ///
    /// User can use `.error_for_status()` or `.json()` to handle the error.
    #[tokio::test]
    async fn test_status_retry_post_500_not_retried() {
        use crate::config::{ExponentialBackoff, HttpClientConfig, RetryConfig};

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(Method::POST).path("/post-500");
            then.status(500).body("server error");
        });

        let config = HttpClientConfig {
            transport: crate::config::TransportSecurity::AllowInsecureHttp,
            retry: Some(RetryConfig {
                max_retries: 3,
                backoff: ExponentialBackoff::fast(),
                ..RetryConfig::default() // 500 is in idempotent_retry, not always_retry
            }),
            rate_limit: None,
            ..Default::default()
        };

        let client = HttpClientBuilder::with_config(config).build().unwrap();
        let url = format!("{}/post-500", server.base_url());

        let result = client.post(&url).send().await;

        // POST on 500 should NOT be retried (only idempotent methods)
        assert_eq!(
            mock.calls(),
            1,
            "POST should not be retried on 500; expected 1 call, got {}",
            mock.calls()
        );

        // Result should be Ok(Response) with status 500 - NOT converted to error
        // because 500 is not retryable for non-idempotent methods
        let response = result.expect("POST + 500 should return Ok(Response), not Err");
        assert_eq!(
            response.status(),
            hyper::StatusCode::INTERNAL_SERVER_ERROR,
            "Response should have 500 status"
        );

        // User can still use error_for_status() to convert to error if needed
    }

    /// Test: POST request with 429 IS retried (`always_retry` policy).
    ///
    /// 429 (Too Many Requests) is in `always_retry` set, so it's retried
    /// regardless of HTTP method. Asserts calls == `max_retries` + 1.
    /// After retries exhaust, returns Ok(Response) with 429 status.
    #[tokio::test]
    async fn test_status_retry_post_429_retried() {
        use crate::config::{ExponentialBackoff, HttpClientConfig, RetryConfig};

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(Method::POST).path("/post-429");
            then.status(429).body("rate limited");
        });

        let config = HttpClientConfig {
            transport: crate::config::TransportSecurity::AllowInsecureHttp,
            retry: Some(RetryConfig {
                max_retries: 2, // 1 initial + 2 retries = 3 total
                backoff: ExponentialBackoff::fast(),
                ..RetryConfig::default() // 429 is in always_retry
            }),
            rate_limit: None,
            ..Default::default()
        };

        let client = HttpClientBuilder::with_config(config).build().unwrap();
        let url = format!("{}/post-429", server.base_url());

        let result = client.post(&url).send().await;

        // POST on 429 SHOULD be retried (429 is in always_retry)
        assert_eq!(
            mock.calls(),
            3,
            "POST should retry on 429; expected 3 calls (1 + 2 retries), got {}",
            mock.calls()
        );

        // After retries exhaust: returns Ok(Response) with 429 status
        let response = result.expect("send() should return Ok(Response) after retries exhaust");
        assert_eq!(response.status(), hyper::StatusCode::TOO_MANY_REQUESTS);

        // User can convert to error via error_for_status()
        let err = response.error_for_status().unwrap_err();
        assert!(
            matches!(err, HttpError::HttpStatus { status, .. } if status == hyper::StatusCode::TOO_MANY_REQUESTS)
        );
    }

    /// Test: Retry-After header is preserved and accessible via `error_for_status()`.
    ///
    /// Server returns 429 with `Retry-After: 60`. `send()` returns Ok(Response).
    /// User calls `error_for_status()` which parses Retry-After from headers.
    #[tokio::test]
    async fn test_status_retry_extracts_retry_after_header() {
        use crate::config::{ExponentialBackoff, HttpClientConfig, RetryConfig};

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(Method::GET).path("/retry-after");
            then.status(429)
                .header("Retry-After", "60")
                .header("Content-Type", "application/json")
                .body(r#"{"error": "rate limited"}"#);
        });

        let config = HttpClientConfig {
            transport: crate::config::TransportSecurity::AllowInsecureHttp,
            retry: Some(RetryConfig {
                max_retries: 0, // No retries - we want to see the response immediately
                backoff: ExponentialBackoff::fast(),
                ..RetryConfig::default()
            }),
            rate_limit: None,
            ..Default::default()
        };

        let client = HttpClientBuilder::with_config(config).build().unwrap();
        let url = format!("{}/retry-after", server.base_url());

        let result = client.get(&url).send().await;

        // send() returns Ok(Response) - status codes don't become Err
        let response = result.expect("send() should return Ok(Response)");
        assert_eq!(response.status(), hyper::StatusCode::TOO_MANY_REQUESTS);

        // error_for_status() extracts Retry-After and Content-Type
        match response.error_for_status() {
            Err(HttpError::HttpStatus {
                status,
                retry_after,
                content_type,
                ..
            }) => {
                assert_eq!(status, hyper::StatusCode::TOO_MANY_REQUESTS);
                assert_eq!(
                    retry_after,
                    Some(std::time::Duration::from_secs(60)),
                    "Should extract Retry-After header"
                );
                assert_eq!(
                    content_type,
                    Some("application/json".to_owned()),
                    "Should extract Content-Type header"
                );
            }
            other => panic!("Expected HttpStatus error from error_for_status(), got: {other:?}"),
        }
    }

    // NOTE: test_status_retry_honors_retry_after_timing was removed because it relied
    // on seconds-scale elapsed time assertions which are flaky in CI environments.
    // The Retry-After header parsing and usage is tested at the unit level in:
    // - response::tests::test_parse_retry_after_*
    // - layers::tests::test_retry_layer_uses_retry_after_header (50ms, fast)
    // - test_status_retry_extracts_retry_after_header (verifies field extraction)

    /// Test: Retry delay ignores Retry-After when `ignore_retry_after=true`.
    ///
    /// Server always returns 429 with `Retry-After: 10`. Config has fast backoff.
    /// Elapsed time should be fast (< 1s), not ~20s (2 * 10s).
    #[tokio::test]
    async fn test_status_retry_ignores_retry_after_when_configured() {
        use crate::config::{ExponentialBackoff, HttpClientConfig, RetryConfig};

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(Method::GET).path("/ignore-retry-after");
            then.status(429)
                .header("Retry-After", "10") // 10 seconds
                .body("rate limited");
        });

        let config = HttpClientConfig {
            transport: crate::config::TransportSecurity::AllowInsecureHttp,
            retry: Some(RetryConfig {
                max_retries: 2,
                backoff: ExponentialBackoff::fast(), // 1ms initial
                ignore_retry_after: true,            // Ignore Retry-After header
                ..RetryConfig::default()
            }),
            rate_limit: None,
            ..Default::default()
        };

        let client = HttpClientBuilder::with_config(config).build().unwrap();
        let url = format!("{}/ignore-retry-after", server.base_url());

        let start = std::time::Instant::now();
        let _result = client.get(&url).send().await;
        let elapsed = start.elapsed();

        // With ignore_retry_after=true and fast backoff, should be very fast
        // NOT 2 * 10s = 20s from Retry-After
        assert!(
            elapsed < std::time::Duration::from_secs(2),
            "Should have used fast backoff, not 10s Retry-After; elapsed: {elapsed:?}"
        );

        // Verify we made 3 calls (1 initial + 2 retries)
        assert_eq!(mock.calls(), 3, "Expected 3 calls, got {}", mock.calls());
    }

    /// Test: Non-retryable status (404) is NOT converted to error by `StatusToErrorLayer`.
    ///
    /// 404 is not in retry triggers, so it passes through as Ok(Response).
    /// User can still use `.error_for_status()` or `.json()` to check.
    #[tokio::test]
    async fn test_non_retryable_status_passes_through() {
        use crate::config::{ExponentialBackoff, HttpClientConfig, RetryConfig};

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(Method::GET).path("/not-found");
            then.status(404)
                .header("content-type", "application/json")
                .body(r#"{"error": "not found"}"#);
        });

        let config = HttpClientConfig {
            transport: crate::config::TransportSecurity::AllowInsecureHttp,
            retry: Some(RetryConfig {
                max_retries: 3,
                backoff: ExponentialBackoff::fast(),
                ..RetryConfig::default()
            }),
            rate_limit: None,
            ..Default::default()
        };

        let client = HttpClientBuilder::with_config(config).build().unwrap();
        let url = format!("{}/not-found", server.base_url());

        // send() should succeed (404 is not a retryable error)
        let result = client.get(&url).send().await;

        // Only called once - no retry
        assert_eq!(
            mock.calls(),
            1,
            "404 should not trigger retry; expected 1 call, got {}",
            mock.calls()
        );

        // Response is Ok, but status is 404
        let response = result.expect("send() should succeed for 404");
        assert_eq!(response.status(), hyper::StatusCode::NOT_FOUND);

        // User can check status manually if needed via error_for_status
    }

    /// Test: Multiple retries exhausted returns Ok(Response) with final status.
    ///
    /// Server always returns 500. After `max_retries` (2) + initial = 3 attempts,
    /// returns Ok(Response) with 500 status. User can use `error_for_status()`.
    #[tokio::test]
    async fn test_status_retry_exhausted_returns_ok_response() {
        use crate::config::{ExponentialBackoff, HttpClientConfig, RetryConfig};

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(Method::GET).path("/always-500");
            then.status(500).body("always fails");
        });

        let config = HttpClientConfig {
            transport: crate::config::TransportSecurity::AllowInsecureHttp,
            retry: Some(RetryConfig {
                max_retries: 2, // 1 initial + 2 retries = 3 total
                backoff: ExponentialBackoff::fast(),
                ..RetryConfig::default()
            }),
            rate_limit: None,
            ..Default::default()
        };

        let client = HttpClientBuilder::with_config(config).build().unwrap();
        let url = format!("{}/always-500", server.base_url());

        let result = client.get(&url).send().await;

        // Should have tried 3 times (1 initial + 2 retries)
        assert_eq!(
            mock.calls(),
            3,
            "Expected 3 calls (1 initial + 2 retries), got {}",
            mock.calls()
        );

        // After retries exhaust: returns Ok(Response) with 500 status
        let response = result.expect("send() should return Ok(Response) after retries exhaust");
        assert_eq!(response.status(), hyper::StatusCode::INTERNAL_SERVER_ERROR);

        // User can convert to error via error_for_status()
        let err = response.error_for_status().unwrap_err();
        assert!(
            matches!(err, HttpError::HttpStatus { status, .. } if status == hyper::StatusCode::INTERNAL_SERVER_ERROR)
        );
    }

    /// Test: Without retry config, retryable statuses pass through as Ok(Response).
    ///
    /// When retry is disabled (None), `StatusToErrorLayer` is not added.
    /// 500 returns Ok(Response), not Err(HttpStatus).
    #[tokio::test]
    async fn test_no_retry_config_status_passes_through() {
        use crate::config::HttpClientConfig;

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(Method::GET).path("/no-retry");
            then.status(500).body("server error");
        });

        let config = HttpClientConfig {
            transport: crate::config::TransportSecurity::AllowInsecureHttp,
            retry: None, // No retry - StatusToErrorLayer not added
            rate_limit: None,
            ..Default::default()
        };

        let client = HttpClientBuilder::with_config(config).build().unwrap();
        let url = format!("{}/no-retry", server.base_url());

        let result = client.get(&url).send().await;

        // Only called once (no retry)
        assert_eq!(mock.calls(), 1);

        // Response is Ok (500 passes through without StatusToErrorLayer)
        let response = result.expect("send() should succeed when retry disabled");
        assert_eq!(response.status(), hyper::StatusCode::INTERNAL_SERVER_ERROR);

        // User can use error_for_status() to convert to error
        let err = response.error_for_status().unwrap_err();
        assert!(
            matches!(err, HttpError::HttpStatus { status, .. } if status == hyper::StatusCode::INTERNAL_SERVER_ERROR)
        );
    }

    // ==========================================================================
    // URL Scheme Validation Tests
    // ==========================================================================

    /// Test: http:// URL rejected when transport security is `TlsOnly` (default)
    #[tokio::test]
    async fn test_url_scheme_http_rejected_with_tls_only() {
        use crate::config::HttpClientConfig;

        // Default config has TlsOnly transport security
        let config = HttpClientConfig {
            retry: None,
            rate_limit: None,
            ..Default::default()
        };
        assert_eq!(config.transport, crate::config::TransportSecurity::TlsOnly);

        let client = HttpClientBuilder::with_config(config).build().unwrap();

        // Try to send a request to http:// URL
        let result = client.get("http://example.com/test").send().await;

        // Should fail with InvalidScheme error
        match result {
            Err(HttpError::InvalidScheme { scheme, reason }) => {
                assert_eq!(scheme, "http");
                assert!(
                    reason.contains("TlsOnly"),
                    "Error should mention TlsOnly: {reason}"
                );
            }
            Err(other) => panic!("Expected InvalidScheme error, got: {other:?}"),
            Ok(_) => panic!("Expected InvalidScheme error, but request succeeded"),
        }
    }

    /// Test: http:// URL allowed when transport security is `AllowInsecureHttp`
    #[tokio::test]
    async fn test_url_scheme_http_allowed_with_allow_insecure() {
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(Method::GET).path("/test");
            then.status(200).body("ok");
        });

        let client = HttpClientBuilder::new()
            .allow_insecure_http()
            .retry(None)
            .build()
            .unwrap();

        let url = format!("{}/test", server.base_url()); // http://127.0.0.1:xxxx
        let result = client.get(&url).send().await;

        assert!(result.is_ok(), "http:// should be allowed: {result:?}");
    }

    /// Test: https:// URL always allowed regardless of transport security
    #[tokio::test]
    async fn test_url_scheme_https_always_allowed() {
        // Note: We can't actually test HTTPS without a real server,
        // but we can verify the validation passes and fails later on connection
        let client = HttpClientBuilder::new().retry(None).build().unwrap();

        // The scheme validation should pass (not InvalidScheme)
        // but the actual connection will fail because example.com won't respond
        let result = client.get("https://localhost:0/test").send().await;

        // Should NOT be InvalidScheme - should be a transport/connection error
        if let Err(HttpError::InvalidScheme { .. }) = result {
            panic!("https:// should not trigger InvalidScheme error")
        }
        // Any other error (transport, timeout, etc.) or Ok is expected
    }

    /// Test: Invalid scheme (e.g., ftp://) rejected
    #[tokio::test]
    async fn test_url_scheme_invalid_rejected() {
        let client = HttpClientBuilder::new()
            .allow_insecure_http() // Even with insecure allowed
            .retry(None)
            .build()
            .unwrap();

        let result = client.get("ftp://files.example.com/file.txt").send().await;

        match result {
            Err(HttpError::InvalidScheme { scheme, reason }) => {
                assert_eq!(scheme, "ftp");
                assert!(
                    reason.contains("http://") || reason.contains("https://"),
                    "Error should mention supported schemes: {reason}"
                );
            }
            Err(other) => panic!("Expected InvalidScheme error, got: {other:?}"),
            Ok(_) => panic!("Expected InvalidScheme error, but request succeeded"),
        }
    }

    /// Test: Missing scheme rejected (now returns `InvalidUri` with proper parsing)
    #[tokio::test]
    async fn test_url_scheme_missing_rejected() {
        let client = HttpClientBuilder::new()
            .allow_insecure_http()
            .retry(None)
            .build()
            .unwrap();

        let result = client.get("example.com/test").send().await;

        match result {
            Err(HttpError::InvalidUri { url, reason, kind }) => {
                // With proper URI parsing, this is an invalid URI (no scheme)
                assert_eq!(url, "example.com/test");
                assert!(!reason.is_empty(), "Should have a reason for invalid URI");
                assert_eq!(kind, crate::error::InvalidUriKind::ParseError);
            }
            Err(other) => panic!("Expected InvalidUri error, got: {other:?}"),
            Ok(_) => panic!("Expected InvalidUri error, but request succeeded"),
        }
    }
}
