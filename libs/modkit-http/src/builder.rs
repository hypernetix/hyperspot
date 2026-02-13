use crate::config::{
    HttpClientConfig, RedirectConfig, RetryConfig, TlsRootConfig, TransportSecurity,
};
use crate::error::HttpError;
use crate::layers::{OtelLayer, RetryLayer, SecureRedirectPolicy, UserAgentLayer};
use crate::response::ResponseBody;
use crate::tls;
use bytes::Bytes;
use http::Response;
use http_body_util::{BodyExt, Full};
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::{TokioExecutor, TokioTimer};
use std::time::Duration;
use tower::buffer::Buffer;
use tower::limit::ConcurrencyLimitLayer;
use tower::load_shed::LoadShedLayer;
use tower::timeout::TimeoutLayer;
use tower::util::BoxCloneService;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::decompression::DecompressionLayer;
use tower_http::follow_redirect::FollowRedirectLayer;

/// Type-erased inner service between layer composition steps in [`HttpClientBuilder::build`].
type InnerService =
    BoxCloneService<http::Request<Full<Bytes>>, http::Response<ResponseBody>, HttpError>;

/// Builder for constructing an [`HttpClient`] with a layered tower middleware stack.
pub struct HttpClientBuilder {
    config: HttpClientConfig,
    auth_layer: Option<Box<dyn FnOnce(InnerService) -> InnerService + Send>>,
}

impl HttpClientBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: HttpClientConfig::default(),
            auth_layer: None,
        }
    }

    /// Create a builder with a specific configuration
    #[must_use]
    pub fn with_config(config: HttpClientConfig) -> Self {
        Self {
            config,
            auth_layer: None,
        }
    }

    /// Set the per-request timeout
    ///
    /// This timeout applies to each individual HTTP request/attempt.
    /// If retries are enabled, each retry attempt gets its own timeout.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.request_timeout = timeout;
        self
    }

    /// Set the total timeout spanning all retry attempts
    ///
    /// When set, the entire operation (including all retries and backoff delays)
    /// must complete within this duration. If the deadline is exceeded,
    /// the request fails with `HttpError::DeadlineExceeded(total_timeout)`.
    #[must_use]
    pub fn total_timeout(mut self, timeout: Duration) -> Self {
        self.config.total_timeout = Some(timeout);
        self
    }

    /// Set the user agent string
    #[must_use]
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = user_agent.into();
        self
    }

    /// Set the retry configuration
    #[must_use]
    pub fn retry(mut self, retry: Option<RetryConfig>) -> Self {
        self.config.retry = retry;
        self
    }

    /// Set the maximum response body size
    #[must_use]
    pub fn max_body_size(mut self, size: usize) -> Self {
        self.config.max_body_size = size;
        self
    }

    /// Set transport security mode
    ///
    /// Use `TransportSecurity::AllowInsecureHttp` only for testing with mock servers.
    #[must_use]
    pub fn transport(mut self, transport: TransportSecurity) -> Self {
        self.config.transport = transport;
        self
    }

    /// Allow insecure HTTP connections (for testing only)
    ///
    /// Equivalent to `.transport(TransportSecurity::AllowInsecureHttp)`.
    ///
    /// **WARNING**: This should only be used for local testing with mock servers.
    /// Never use in production as it exposes traffic to interception.
    ///
    /// # Compile-time Safety
    ///
    /// This method is only available in debug builds or when the `allow-insecure-http`
    /// feature is explicitly enabled. This prevents accidental use in production.
    ///
    /// To use in release builds (e.g., for integration tests), add:
    /// ```toml
    /// [features]
    /// allow-insecure-http = []
    /// ```
    #[must_use]
    #[cfg(any(debug_assertions, feature = "allow-insecure-http"))]
    pub fn allow_insecure_http(mut self) -> Self {
        tracing::warn!(
            target: "modkit_http::security",
            "allow_insecure_http() called - HTTP traffic will NOT be encrypted"
        );
        self.config.transport = TransportSecurity::AllowInsecureHttp;
        self
    }

    /// Enable OpenTelemetry tracing layer
    ///
    /// When enabled, creates spans for outbound requests with HTTP metadata
    /// and injects W3C trace context headers (when `otel` feature is enabled).
    #[must_use]
    pub fn with_otel(mut self) -> Self {
        self.config.otel = true;
        self
    }

    /// Insert an optional auth layer between retry and timeout in the stack.
    ///
    /// Stack position: `… → Retry → **this layer** → Timeout → …`
    ///
    /// The layer sits inside the retry loop so each attempt re-executes it
    /// (e.g. re-reads a refreshed bearer token). Only one auth layer can be
    /// set; a second call replaces the first.
    #[must_use]
    pub fn with_auth_layer(
        mut self,
        wrap: impl FnOnce(InnerService) -> InnerService + Send + 'static,
    ) -> Self {
        self.auth_layer = Some(Box::new(wrap));
        self
    }

    /// Set the buffer capacity for concurrent request handling
    ///
    /// The HTTP client uses an internal buffer to allow concurrent requests
    /// without external locking. This sets the maximum number of requests
    /// that can be queued.
    ///
    /// **Note**: A capacity of 0 is invalid and will be clamped to 1.
    /// Tower's Buffer panics with capacity=0, so we enforce minimum of 1.
    #[must_use]
    pub fn buffer_capacity(mut self, capacity: usize) -> Self {
        // Clamp to at least 1 - tower::Buffer panics with capacity=0
        self.config.buffer_capacity = capacity.max(1);
        self
    }

    /// Set the maximum number of redirects to follow
    ///
    /// Set to `0` to disable redirect following (3xx responses pass through as-is).
    /// Default: 10
    #[must_use]
    pub fn max_redirects(mut self, max_redirects: usize) -> Self {
        self.config.redirect.max_redirects = max_redirects;
        self
    }

    /// Disable redirect following
    ///
    /// Equivalent to `.max_redirects(0)`. When disabled, 3xx responses are
    /// returned to the caller without following the `Location` header.
    #[must_use]
    pub fn no_redirects(mut self) -> Self {
        self.config.redirect = RedirectConfig::disabled();
        self
    }

    /// Set the redirect policy configuration
    ///
    /// Use this to configure redirect security settings:
    /// - `same_origin_only`: Only follow redirects to the same host
    /// - `strip_sensitive_headers`: Remove `Authorization`/`Cookie` on cross-origin
    /// - `allow_https_downgrade`: Allow HTTPS → HTTP redirects (not recommended)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = HttpClient::builder()
    ///     .redirect(RedirectConfig::permissive()) // Allow all redirects with header stripping
    ///     .build()?;
    /// ```
    #[must_use]
    pub fn redirect(mut self, config: RedirectConfig) -> Self {
        self.config.redirect = config;
        self
    }

    /// Set the idle connection timeout for the connection pool
    ///
    /// Connections that remain idle for longer than this duration will be
    /// closed and removed from the pool. Default: 90 seconds.
    ///
    /// Set to `None` to disable idle timeout (connections kept indefinitely).
    #[must_use]
    pub fn pool_idle_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.config.pool_idle_timeout = timeout;
        self
    }

    /// Set the maximum number of idle connections per host
    ///
    /// Limits how many idle connections are kept in the pool for each host.
    /// Default: 32.
    ///
    /// - Setting to `0` disables connection reuse entirely
    /// - Setting too high may waste resources on rarely-used connections
    #[must_use]
    pub fn pool_max_idle_per_host(mut self, max: usize) -> Self {
        self.config.pool_max_idle_per_host = max;
        self
    }

    /// Build the HTTP client with all configured layers
    ///
    /// # Errors
    /// Returns an error if TLS initialization fails or configuration is invalid
    pub fn build(self) -> Result<crate::HttpClient, HttpError> {
        // Warn if insecure HTTP is enabled (should only be used for testing)
        if self.config.transport == TransportSecurity::AllowInsecureHttp {
            tracing::warn!(
                "insecure HTTP enabled (TransportSecurity::AllowInsecureHttp); \
                 use only for testing with mock servers"
            );
        }

        let timeout = self.config.request_timeout;
        let total_timeout = self.config.total_timeout;

        // Build the HTTPS connector (may fail for Native roots if no valid certs)
        let https = build_https_connector(self.config.tls_roots, self.config.transport)?;

        // Create the base hyper client with HTTP/2 support and connection pool settings
        let mut client_builder = Client::builder(TokioExecutor::new());

        // Configure connection pool
        // CRITICAL: pool_timer is required for pool_idle_timeout to work!
        client_builder
            .pool_timer(TokioTimer::new())
            .pool_max_idle_per_host(self.config.pool_max_idle_per_host)
            .http2_only(false); // Allow both HTTP/1 and HTTP/2 via ALPN

        // Set idle timeout (None = no timeout, connections kept indefinitely)
        if let Some(idle_timeout) = self.config.pool_idle_timeout {
            client_builder.pool_idle_timeout(idle_timeout);
        }

        let hyper_client = client_builder.build::<_, Full<Bytes>>(https);

        // Parse user agent header (may fail)
        let ua_layer = UserAgentLayer::try_new(&self.config.user_agent)?;

        // =======================================================================
        // Tower Layer Stack (outer to inner)
        // =======================================================================
        //
        // Request flow (outer → inner):
        //   Buffer → OtelLayer → LoadShed/Concurrency → RetryLayer →
        //   [AuthLayer?] → ErrorMapping → Timeout → UserAgent →
        //   Decompression → FollowRedirect → hyper_client
        //
        // AuthLayer (if set via with_auth_layer) sits inside the retry
        // loop so each retry re-acquires credentials (e.g. refreshed
        // bearer token).
        //
        // Response flow (inner → outer):
        //   hyper_client → FollowRedirect → Decompression → UserAgent →
        //   Timeout → ErrorMapping → [AuthLayer?] → RetryLayer →
        //   LoadShed/Concurrency → OtelLayer → Buffer
        //
        // Key semantics (reqwest-like):
        //   - send() returns Ok(Response) for ALL HTTP statuses (including 4xx/5xx)
        //   - send() returns Err only for transport/timeout/TLS errors
        //   - Non-2xx converted to error ONLY via error_for_status()
        //   - RetryLayer handles both Err (transport) and Ok(Response) (status)
        //     retries internally, draining body before retry for connection reuse
        //   - FollowRedirect handles 3xx responses internally with security protections:
        //     * Same-origin enforcement (default) - blocks SSRF attacks
        //     * Sensitive header stripping on cross-origin redirects
        //     * HTTPS downgrade protection
        //
        // =======================================================================
        //
        let redirect_policy = SecureRedirectPolicy::new(self.config.redirect.clone());

        // Build the service stack with secure redirect following
        let service = ServiceBuilder::new()
            .layer(TimeoutLayer::new(timeout))
            .layer(ua_layer)
            .layer(DecompressionLayer::new())
            .layer(FollowRedirectLayer::with_policy(redirect_policy))
            .service(hyper_client);

        // Map the decompression body to our boxed ResponseBody type.
        // This converts Response<DecompressionBody<Incoming>> to Response<ResponseBody>.
        //
        // The decompression body's error type is tower_http::BoxError, which we convert
        // to our boxed error type for consistency with the ResponseBody definition.
        let service = service.map_response(map_decompression_response);

        // Map errors to HttpError with proper timeout duration
        let service = service.map_err(move |e: tower::BoxError| map_tower_error(e, timeout));

        // Box the service for type erasure
        let mut boxed_service = service.boxed_clone();

        // Apply auth layer (between timeout and retry).
        // Inside retry so each retry attempt re-acquires the token.
        if let Some(wrap) = self.auth_layer {
            boxed_service = wrap(boxed_service);
        }

        // Conditionally wrap with RetryLayer
        //
        // RetryLayer handles retries for both:
        // - Err(HttpError::Transport/Timeout) - transport-level failures
        // - Ok(Response) with retryable status codes (429, 5xx for GET, etc.)
        //
        // When retrying on status codes, RetryLayer drains the response body
        // (up to configured limit) to allow connection reuse.
        //
        // If total_timeout is set, the entire operation (including all retries)
        // must complete within that duration.
        if let Some(ref retry_config) = self.config.retry {
            let retry_layer = RetryLayer::with_total_timeout(retry_config.clone(), total_timeout);
            let retry_service = ServiceBuilder::new()
                .layer(retry_layer)
                .service(boxed_service);
            boxed_service = retry_service.boxed_clone();
        }

        // Conditionally wrap with concurrency limit + load shedding
        // LoadShedLayer returns error immediately when ConcurrencyLimitLayer is saturated
        // instead of waiting indefinitely (Poll::Pending)
        if let Some(rate_limit) = self.config.rate_limit
            && rate_limit.max_concurrent_requests < usize::MAX
        {
            let limited_service = ServiceBuilder::new()
                .layer(LoadShedLayer::new())
                .layer(ConcurrencyLimitLayer::new(
                    rate_limit.max_concurrent_requests,
                ))
                .service(boxed_service);
            // Map load shed errors to HttpError::Overloaded
            let limited_service = limited_service.map_err(map_load_shed_error);
            boxed_service = limited_service.boxed_clone();
        }

        // Conditionally wrap with OTEL tracing layer (outermost layer before buffer)
        // Applied last so it sees the final request after UserAgent and other modifications.
        // Creates spans, records status, and injects trace context headers.
        if self.config.otel {
            let otel_service = ServiceBuilder::new()
                .layer(OtelLayer::new())
                .service(boxed_service);
            boxed_service = otel_service.boxed_clone();
        }

        // Wrap in Buffer as the final step for true concurrent access
        // Buffer spawns a background task that processes requests from a channel,
        // providing Clone + Send + Sync without any mutex serialization.
        let buffer_capacity = self.config.buffer_capacity.max(1);
        let buffered_service: crate::client::BufferedService =
            Buffer::new(boxed_service, buffer_capacity);

        Ok(crate::HttpClient {
            service: buffered_service,
            max_body_size: self.config.max_body_size,
            transport_security: self.config.transport,
        })
    }
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Map tower errors to `HttpError` with actual timeout duration
///
/// Attempts to extract existing `HttpError` from the boxed error before
/// wrapping as `Transport`. This preserves typed errors like `Overloaded`
/// and `ServiceClosed` that may have been boxed by tower middleware.
fn map_tower_error(err: tower::BoxError, timeout: Duration) -> HttpError {
    if err.is::<tower::timeout::error::Elapsed>() {
        return HttpError::Timeout(timeout);
    }

    // Try to extract existing HttpError before wrapping as Transport
    match err.downcast::<HttpError>() {
        Ok(http_err) => *http_err,
        Err(other) => HttpError::Transport(other),
    }
}

/// Map load shed errors to `HttpError::Overloaded`
fn map_load_shed_error(err: tower::BoxError) -> HttpError {
    if err.is::<tower::load_shed::error::Overloaded>() {
        HttpError::Overloaded
    } else {
        // Pass through other HttpError types (from inner service)
        match err.downcast::<HttpError>() {
            Ok(http_err) => *http_err,
            Err(err) => HttpError::Transport(err),
        }
    }
}

/// Map the decompression response to our boxed response body type.
///
/// This converts `Response<DecompressionBody<Incoming>>` to `Response<ResponseBody>`
/// by boxing the body with appropriate error type mapping.
fn map_decompression_response<B>(response: Response<B>) -> Response<ResponseBody>
where
    B: hyper::body::Body<Data = Bytes> + Send + Sync + 'static,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    let (parts, body) = response.into_parts();
    // Convert the decompression body errors to our boxed error type.
    // tower-http's DecompressionBody uses tower_http::BoxError which is
    // compatible with our Box<dyn Error + Send + Sync> via Into.
    let boxed_body: ResponseBody = body.map_err(Into::into).boxed();
    Response::from_parts(parts, boxed_body)
}

/// Build the HTTPS connector with the specified TLS root configuration.
///
/// For `TlsRootConfig::Native`, uses cached native root certificates to avoid
/// repeated OS certificate store lookups on each `build()` call.
///
/// HTTP/2 is enabled via `enable_all_versions()` which configures ALPN to
/// advertise both h2 and http/1.1. Protocol selection happens during TLS
/// handshake based on server support.
///
/// # Errors
///
/// Returns `HttpError::Tls` if `TlsRootConfig::Native` is requested but no
/// valid root certificates are available from the OS certificate store.
fn build_https_connector(
    tls_roots: TlsRootConfig,
    transport: TransportSecurity,
) -> Result<HttpsConnector<HttpConnector>, HttpError> {
    let allow_http = transport == TransportSecurity::AllowInsecureHttp;

    match tls_roots {
        TlsRootConfig::WebPki => {
            let provider = tls::get_crypto_provider();
            let builder = hyper_rustls::HttpsConnectorBuilder::new()
                .with_provider_and_webpki_roots(provider)
                // Preserve source error for debugging —
                // rustls::Error implements Error + Send + Sync
                .map_err(|e| HttpError::Tls(Box::new(e)))?;
            let connector = if allow_http {
                builder.https_or_http().enable_all_versions().build()
            } else {
                builder.https_only().enable_all_versions().build()
            };
            Ok(connector)
        }
        TlsRootConfig::Native => {
            let client_config = tls::native_roots_client_config()
                // Native returns String error; convert to boxed error for consistency
                .map_err(|e| HttpError::Tls(e.into()))?;
            let builder = hyper_rustls::HttpsConnectorBuilder::new().with_tls_config(client_config);
            let connector = if allow_http {
                builder.https_or_http().enable_all_versions().build()
            } else {
                builder.https_only().enable_all_versions().build()
            };
            Ok(connector)
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::config::DEFAULT_USER_AGENT;

    #[test]
    fn test_builder_default() {
        let builder = HttpClientBuilder::new();
        assert_eq!(builder.config.request_timeout, Duration::from_secs(30));
        assert_eq!(builder.config.user_agent, DEFAULT_USER_AGENT);
        assert!(builder.config.retry.is_some());
        assert_eq!(builder.config.buffer_capacity, 1024);
    }

    #[test]
    fn test_builder_with_config() {
        let config = HttpClientConfig::minimal();
        let builder = HttpClientBuilder::with_config(config);
        assert_eq!(builder.config.request_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_builder_timeout() {
        let builder = HttpClientBuilder::new().timeout(Duration::from_secs(60));
        assert_eq!(builder.config.request_timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_builder_user_agent() {
        let builder = HttpClientBuilder::new().user_agent("custom/1.0");
        assert_eq!(builder.config.user_agent, "custom/1.0");
    }

    #[test]
    fn test_builder_retry() {
        let builder = HttpClientBuilder::new().retry(None);
        assert!(builder.config.retry.is_none());
    }

    #[test]
    fn test_builder_max_body_size() {
        let builder = HttpClientBuilder::new().max_body_size(1024);
        assert_eq!(builder.config.max_body_size, 1024);
    }

    #[test]
    fn test_builder_transport_security() {
        let builder = HttpClientBuilder::new().transport(TransportSecurity::AllowInsecureHttp);
        assert_eq!(
            builder.config.transport,
            TransportSecurity::AllowInsecureHttp
        );

        let builder = HttpClientBuilder::new().allow_insecure_http();
        assert_eq!(
            builder.config.transport,
            TransportSecurity::AllowInsecureHttp
        );

        let builder = HttpClientBuilder::new();
        assert_eq!(builder.config.transport, TransportSecurity::TlsOnly);
    }

    #[test]
    fn test_builder_otel() {
        let builder = HttpClientBuilder::new().with_otel();
        assert!(builder.config.otel);
    }

    #[test]
    fn test_builder_buffer_capacity() {
        let builder = HttpClientBuilder::new().buffer_capacity(512);
        assert_eq!(builder.config.buffer_capacity, 512);
    }

    /// Test that `buffer_capacity=0` is clamped to 1 to prevent panic.
    ///
    /// Tower's Buffer panics with capacity=0, so we enforce minimum of 1.
    #[test]
    fn test_builder_buffer_capacity_zero_clamped() {
        let builder = HttpClientBuilder::new().buffer_capacity(0);
        assert_eq!(
            builder.config.buffer_capacity, 1,
            "buffer_capacity=0 should be clamped to 1"
        );
    }

    /// Test that `buffer_capacity=0` via config is clamped during `build()`.
    #[tokio::test]
    async fn test_builder_buffer_capacity_zero_in_config_clamped() {
        let config = HttpClientConfig {
            buffer_capacity: 0, // Invalid - should be clamped in build()
            ..Default::default()
        };
        let result = HttpClientBuilder::with_config(config).build();
        // Should succeed (clamped to 1), not panic
        assert!(
            result.is_ok(),
            "build() should succeed with capacity clamped to 1"
        );
    }

    #[tokio::test]
    async fn test_builder_build_with_otel() {
        let client = HttpClientBuilder::new().with_otel().build();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_builder_with_auth_layer() {
        let client = HttpClientBuilder::new()
            .allow_insecure_http()
            .with_auth_layer(|svc| svc) // identity transform
            .build();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_builder_build() {
        let client = HttpClientBuilder::new().build();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_builder_build_with_insecure_http() {
        let client = HttpClientBuilder::new().allow_insecure_http().build();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_builder_build_with_sse_config() {
        use crate::config::HttpClientConfig;
        let config = HttpClientConfig::sse();
        let client = HttpClientBuilder::with_config(config).build();
        assert!(client.is_ok(), "SSE config should build successfully");
    }

    #[tokio::test]
    async fn test_builder_build_invalid_user_agent() {
        let client = HttpClientBuilder::new()
            .user_agent("invalid\x00agent")
            .build();
        assert!(client.is_err());
    }

    #[tokio::test]
    async fn test_builder_default_uses_webpki_roots() {
        let builder = HttpClientBuilder::new();
        assert_eq!(builder.config.tls_roots, TlsRootConfig::WebPki);
        // Build should succeed without OS native roots
        let client = builder.build();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_builder_native_roots() {
        let config = HttpClientConfig {
            tls_roots: TlsRootConfig::Native,
            ..Default::default()
        };
        let result = HttpClientBuilder::with_config(config).build();

        // Native roots may succeed or fail depending on OS certificate availability.
        // On systems with certs: Ok(_)
        // On minimal containers without certs: Err(HttpError::Tls(_))
        match &result {
            Ok(_) => {
                // Success on systems with native certs
            }
            Err(HttpError::Tls(err)) => {
                // Expected failure on systems without native certs
                let msg = err.to_string();
                assert!(
                    msg.contains("native root") || msg.contains("certificate"),
                    "TLS error should mention certificates: {msg}"
                );
            }
            Err(other) => {
                panic!("Unexpected error type: {other:?}");
            }
        }
    }

    #[tokio::test]
    async fn test_builder_webpki_roots_https_only() {
        let config = HttpClientConfig {
            tls_roots: TlsRootConfig::WebPki,
            transport: TransportSecurity::TlsOnly,
            ..Default::default()
        };
        let client = HttpClientBuilder::with_config(config).build();
        assert!(client.is_ok());
    }

    /// Verify HTTP/2 is enabled for all TLS root configurations.
    ///
    /// HTTP/2 support is configured via `enable_all_versions()` on the connector,
    /// which sets up ALPN to negotiate h2 or http/1.1 during TLS handshake.
    /// The hyper client uses `http2_only(false)` to allow both protocols.
    #[tokio::test]
    async fn test_http2_enabled_for_all_configurations() {
        // Test WebPki with AllowInsecureHttp
        let client = HttpClientBuilder::new().allow_insecure_http().build();
        assert!(
            client.is_ok(),
            "WebPki + AllowInsecureHttp should build with HTTP/2 enabled"
        );

        // Test WebPki with TlsOnly (HTTPS only)
        let client = HttpClientBuilder::new()
            .transport(TransportSecurity::TlsOnly)
            .build();
        assert!(
            client.is_ok(),
            "WebPki + TlsOnly should build with HTTP/2 enabled"
        );

        // Test Native roots with AllowInsecureHttp
        let config = HttpClientConfig {
            tls_roots: TlsRootConfig::Native,
            transport: TransportSecurity::AllowInsecureHttp,
            ..Default::default()
        };
        let client = HttpClientBuilder::with_config(config).build();
        assert!(
            client.is_ok(),
            "Native + AllowInsecureHttp should build with HTTP/2 enabled"
        );

        // Test Native roots with TlsOnly (HTTPS only)
        let config = HttpClientConfig {
            tls_roots: TlsRootConfig::Native,
            transport: TransportSecurity::TlsOnly,
            ..Default::default()
        };
        let client = HttpClientBuilder::with_config(config).build();
        assert!(
            client.is_ok(),
            "Native + TlsOnly should build with HTTP/2 enabled"
        );
    }

    /// Test that concurrency limit uses fail-fast behavior (C2).
    ///
    /// `LoadShedLayer` + `ConcurrencyLimitLayer` combination returns Overloaded error
    /// immediately when capacity is exhausted, instead of blocking indefinitely.
    #[tokio::test]
    async fn test_load_shedding_returns_overloaded_error() {
        use bytes::Bytes;
        use http::{Request, Response};
        use http_body_util::Full;
        use std::future::Future;
        use std::pin::Pin;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::task::{Context, Poll};
        use tower::Service;
        use tower::ServiceExt;

        // A service that holds a slot forever once called
        #[derive(Clone)]
        struct SlotHoldingService {
            active: Arc<AtomicUsize>,
        }

        impl Service<Request<Full<Bytes>>> for SlotHoldingService {
            type Response = Response<Full<Bytes>>;
            type Error = HttpError;
            type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

            fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                Poll::Ready(Ok(()))
            }

            fn call(&mut self, _: Request<Full<Bytes>>) -> Self::Future {
                self.active.fetch_add(1, Ordering::SeqCst);
                // Never complete - holds the slot
                Box::pin(std::future::pending())
            }
        }

        let active = Arc::new(AtomicUsize::new(0));

        // Build a service with load shedding and concurrency limit of 1
        let service = tower::ServiceBuilder::new()
            .layer(LoadShedLayer::new())
            .layer(ConcurrencyLimitLayer::new(1))
            .service(SlotHoldingService {
                active: active.clone(),
            });

        let service = service.map_err(map_load_shed_error);

        // First request: will occupy the single slot
        let req1 = Request::builder()
            .uri("http://test")
            .body(Full::new(Bytes::new()))
            .unwrap();
        let mut svc1 = service.clone();

        let svc1_ready = svc1.ready().await.unwrap();
        let _pending_fut = svc1_ready.call(req1);

        // Wait for the slot to be occupied
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(
            active.load(Ordering::SeqCst),
            1,
            "First request should be active"
        );

        // Second request: LoadShedLayer should reject because ConcurrencyLimit is at capacity
        let req2 = Request::builder()
            .uri("http://test")
            .body(Full::new(Bytes::new()))
            .unwrap();

        let mut svc2 = service.clone();

        // LoadShedLayer checks poll_ready and returns Overloaded if inner service is not ready
        let result = tokio::time::timeout(Duration::from_millis(100), async {
            // poll_ready should return quickly with error (not block)
            match svc2.ready().await {
                Ok(ready_svc) => ready_svc.call(req2).await,
                Err(e) => Err(e),
            }
        })
        .await;

        // Should complete within timeout (not hang) and return Overloaded
        assert!(result.is_ok(), "Request should not hang");
        let err = result.unwrap().unwrap_err();
        assert!(
            matches!(err, HttpError::Overloaded),
            "Expected Overloaded error, got: {err:?}"
        );
    }

    /// Test that `AllowInsecureHttp` emits a warning during `build()`
    #[tokio::test]
    async fn test_insecure_http_warning_emitted() {
        use std::sync::{Arc, Mutex};
        use tracing_subscriber::layer::SubscriberExt;

        // Custom layer to capture warning events
        #[derive(Clone, Default)]
        struct WarningCapture {
            warnings: Arc<Mutex<Vec<String>>>,
        }

        impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for WarningCapture {
            fn on_event(
                &self,
                event: &tracing::Event<'_>,
                _ctx: tracing_subscriber::layer::Context<'_, S>,
            ) {
                if *event.metadata().level() == tracing::Level::WARN {
                    let mut visitor = MessageVisitor(String::new());
                    event.record(&mut visitor);
                    self.warnings.lock().unwrap().push(visitor.0);
                }
            }
        }

        struct MessageVisitor(String);
        impl tracing::field::Visit for MessageVisitor {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.0 = format!("{value:?}");
                }
            }
        }

        let capture = WarningCapture::default();
        let warnings = capture.warnings.clone();

        let subscriber = tracing_subscriber::registry().with(capture);

        // Build with `AllowInsecureHttp` under the capturing subscriber
        tracing::subscriber::with_default(subscriber, || {
            _ = HttpClientBuilder::new().allow_insecure_http().build();
        });

        let captured = warnings.lock().unwrap();
        // Two warnings: one from allow_insecure_http() and one from build()
        assert!(
            !captured.is_empty(),
            "expected at least one warning, got: {:?}",
            *captured
        );
        assert!(
            captured
                .iter()
                .any(|w| w.contains("insecure HTTP") || w.contains("HTTP traffic")),
            "warning should mention insecure HTTP: {:?}",
            *captured
        );
    }

    /// Test that `TlsOnly` does NOT emit an insecure HTTP warning
    #[tokio::test]
    async fn test_tls_only_no_warning() {
        use std::sync::{Arc, Mutex};
        use tracing_subscriber::layer::SubscriberExt;

        #[derive(Clone, Default)]
        struct WarningCapture {
            warnings: Arc<Mutex<Vec<String>>>,
        }

        impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for WarningCapture {
            fn on_event(
                &self,
                event: &tracing::Event<'_>,
                _ctx: tracing_subscriber::layer::Context<'_, S>,
            ) {
                if *event.metadata().level() == tracing::Level::WARN {
                    let mut visitor = MessageVisitor(String::new());
                    event.record(&mut visitor);
                    if visitor.0.contains("insecure HTTP") {
                        self.warnings.lock().unwrap().push(visitor.0);
                    }
                }
            }
        }

        struct MessageVisitor(String);
        impl tracing::field::Visit for MessageVisitor {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.0 = format!("{value:?}");
                }
            }
        }

        let capture = WarningCapture::default();
        let warnings = capture.warnings.clone();

        let subscriber = tracing_subscriber::registry().with(capture);

        // Build with `TlsOnly`
        tracing::subscriber::with_default(subscriber, || {
            _ = HttpClientBuilder::new()
                .transport(TransportSecurity::TlsOnly)
                .build();
        });

        let captured = warnings.lock().unwrap();
        assert!(
            captured.is_empty(),
            "no insecure HTTP warning expected, got: {:?}",
            *captured
        );
    }

    // ==========================================================================
    // map_tower_error Tests
    // ==========================================================================

    /// Test that `map_tower_error` preserves `HttpError::Overloaded` when wrapped in `BoxError`
    #[test]
    fn test_map_tower_error_preserves_overloaded() {
        let http_err = HttpError::Overloaded;
        let boxed: tower::BoxError = Box::new(http_err);
        let result = map_tower_error(boxed, Duration::from_secs(30));

        assert!(
            matches!(result, HttpError::Overloaded),
            "Should preserve HttpError::Overloaded, got: {result:?}"
        );
    }

    /// Test that `map_tower_error` preserves `HttpError::ServiceClosed` when wrapped in `BoxError`
    #[test]
    fn test_map_tower_error_preserves_service_closed() {
        let http_err = HttpError::ServiceClosed;
        let boxed: tower::BoxError = Box::new(http_err);
        let result = map_tower_error(boxed, Duration::from_secs(30));

        assert!(
            matches!(result, HttpError::ServiceClosed),
            "Should preserve HttpError::ServiceClosed, got: {result:?}"
        );
    }

    /// Test that `map_tower_error` preserves `HttpError::Timeout` with original duration
    #[test]
    fn test_map_tower_error_preserves_timeout_attempt() {
        let original_duration = Duration::from_secs(5);
        let http_err = HttpError::Timeout(original_duration);
        let boxed: tower::BoxError = Box::new(http_err);
        // Pass a different timeout to verify original is preserved
        let result = map_tower_error(boxed, Duration::from_secs(30));

        match result {
            HttpError::Timeout(d) => {
                assert_eq!(
                    d, original_duration,
                    "Should preserve original timeout duration"
                );
            }
            other => panic!("Should preserve HttpError::Timeout, got: {other:?}"),
        }
    }

    /// Test that `map_tower_error` wraps unknown errors as Transport
    #[test]
    fn test_map_tower_error_wraps_unknown_as_transport() {
        let other_err: tower::BoxError = Box::new(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "connection refused",
        ));
        let result = map_tower_error(other_err, Duration::from_secs(30));

        assert!(
            matches!(result, HttpError::Transport(_)),
            "Should wrap unknown errors as Transport, got: {result:?}"
        );
    }
}
