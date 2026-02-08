use std::collections::HashSet;
use std::time::Duration;

/// Default User-Agent string for HTTP requests
pub const DEFAULT_USER_AGENT: &str = concat!("modkit-http/", env!("CARGO_PKG_VERSION"));

/// Standard idempotency key header name (display form)
pub const IDEMPOTENCY_KEY_HEADER: &str = "Idempotency-Key";

/// Lowercase idempotency key header for `HeaderName` construction
const IDEMPOTENCY_KEY_HEADER_LOWER: &str = "idempotency-key";

/// Conditions that trigger a retry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RetryTrigger {
    /// Transport-level errors (connection refused, DNS failure, reset, etc.)
    TransportError,
    /// Request timeout
    Timeout,
    /// Specific HTTP status code
    Status(u16),
    /// Error that is never retryable (e.g., `DeadlineExceeded`, `ServiceClosed`)
    NonRetryable,
}

impl RetryTrigger {
    /// Create a trigger for HTTP 429 Too Many Requests
    pub const TOO_MANY_REQUESTS: Self = Self::Status(429);
    /// Create a trigger for HTTP 408 Request Timeout
    pub const REQUEST_TIMEOUT: Self = Self::Status(408);
    /// Create a trigger for HTTP 500 Internal Server Error
    pub const INTERNAL_SERVER_ERROR: Self = Self::Status(500);
    /// Create a trigger for HTTP 502 Bad Gateway
    pub const BAD_GATEWAY: Self = Self::Status(502);
    /// Create a trigger for HTTP 503 Service Unavailable
    pub const SERVICE_UNAVAILABLE: Self = Self::Status(503);
    /// Create a trigger for HTTP 504 Gateway Timeout
    pub const GATEWAY_TIMEOUT: Self = Self::Status(504);
}

/// Check if HTTP method is idempotent (safe to retry) per RFC 9110.
///
/// Idempotent methods: GET, HEAD, PUT, DELETE, OPTIONS, TRACE.
/// Non-idempotent methods: POST, PATCH.
#[must_use]
pub fn is_idempotent_method(method: &http::Method) -> bool {
    matches!(
        *method,
        http::Method::GET
            | http::Method::HEAD
            | http::Method::PUT
            | http::Method::DELETE
            | http::Method::OPTIONS
            | http::Method::TRACE
    )
}

/// Exponential backoff configuration for retries
///
/// Computes delay as: `min(initial * multiplier^attempt, max)` with optional jitter.
#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    /// Initial backoff duration (default: 100ms)
    pub initial: Duration,

    /// Maximum backoff duration (default: 10s)
    pub max: Duration,

    /// Backoff multiplier for exponential growth (default: 2.0)
    pub multiplier: f64,

    /// Enable jitter to prevent thundering herd (default: true)
    ///
    /// When enabled, adds random delay of 0-25% to each backoff.
    pub jitter: bool,
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self {
            initial: Duration::from_millis(100),
            max: Duration::from_secs(10),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

impl ExponentialBackoff {
    /// Create backoff with custom initial and max durations
    #[must_use]
    pub fn new(initial: Duration, max: Duration) -> Self {
        Self {
            initial,
            max,
            ..Default::default()
        }
    }

    /// Create fast backoff for testing (1ms initial, 100ms max, no jitter)
    #[must_use]
    pub fn fast() -> Self {
        Self {
            initial: Duration::from_millis(1),
            max: Duration::from_millis(100),
            multiplier: 2.0,
            jitter: false,
        }
    }

    /// Create aggressive backoff (50ms initial, 30s max)
    #[must_use]
    pub fn aggressive() -> Self {
        Self {
            initial: Duration::from_millis(50),
            max: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Retry policy configuration with exponential backoff
///
/// Retry decisions are based on two sets of triggers:
/// - `always_retry`: Conditions that always trigger retry (regardless of HTTP method)
/// - `idempotent_retry`: Conditions that trigger retry only for idempotent methods (GET, HEAD, PUT, DELETE, OPTIONS, TRACE)
///   OR when the request has an idempotency key header
///
/// **Safety by default**: Non-idempotent methods (POST, PATCH) are only retried on
/// triggers in `always_retry` unless the request contains an idempotency key header.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries after the initial attempt (0 = no retries, default: 3)
    /// Total attempts = 1 (initial) + `max_retries`
    pub max_retries: usize,

    /// Backoff strategy configuration
    pub backoff: ExponentialBackoff,

    /// Triggers that always retry regardless of HTTP method
    /// Default: [Status(429)]
    ///
    /// **Note**: `TransportError` and `Timeout` are NOT in `always_retry` by default to avoid
    /// duplicating non-idempotent requests. They are in `idempotent_retry` instead.
    pub always_retry: HashSet<RetryTrigger>,

    /// Triggers that only retry for idempotent methods (GET, HEAD, OPTIONS, TRACE)
    /// OR when the request has an idempotency key header.
    /// Default: `[TransportError, Timeout, Status(408), Status(500), Status(502), Status(503), Status(504)]`
    pub idempotent_retry: HashSet<RetryTrigger>,

    /// If true, ignore the `Retry-After` HTTP header and always use backoff policy.
    /// If false (default), use `Retry-After` value when present for computing retry delay.
    pub ignore_retry_after: bool,

    /// Maximum bytes to drain from response body before retrying on HTTP status.
    /// Draining the body allows connection reuse. Default: 64 KiB.
    /// If the body exceeds this limit, draining stops and the connection may not be reused.
    ///
    /// **Note**: This limit applies to **decompressed** bytes. For compressed responses,
    /// the actual network traffic may be smaller than the configured limit.
    pub retry_response_drain_limit: usize,

    /// Whether to skip draining response body on retry.
    ///
    /// When `true`, the response body is not drained before retrying, meaning
    /// connections may not be reused after retryable errors. This saves CPU/memory
    /// by not decompressing error response bodies.
    ///
    /// # Performance Tradeoff
    ///
    /// Body draining operates on **decompressed** bytes (after `DecompressionLayer`).
    /// When servers return compressed error responses (e.g., gzip-compressed 503 HTML),
    /// draining requires CPU to decompress the body even though we discard the content.
    ///
    /// **Recommendation:**
    /// - Set to `true` for high-throughput services where connection reuse is less
    ///   important than CPU efficiency, or when error responses are typically compressed
    /// - Keep `false` (default) for low-to-medium throughput services where connection
    ///   reuse reduces latency and TCP connection overhead
    ///
    /// The `Content-Length` header is checked before draining; bodies larger than
    /// `retry_response_drain_limit` are skipped automatically regardless of this setting.
    ///
    /// Default: `false` (drain enabled for connection reuse)
    pub skip_drain_on_retry: bool,

    /// Header name that, when present on a request, enables retry for non-idempotent methods.
    /// Default: "Idempotency-Key"
    ///
    /// Set to `None` to disable idempotency-key based retry (only `always_retry` triggers
    /// will apply to non-idempotent methods).
    ///
    /// When a request includes this header, triggers in `idempotent_retry` will apply
    /// regardless of the HTTP method.
    ///
    /// Pre-parsed at config construction to avoid runtime parsing overhead.
    pub idempotency_key_header: Option<http::header::HeaderName>,
}

/// Default drain limit for response bodies before retry (64 KiB)
pub const DEFAULT_RETRY_RESPONSE_DRAIN_LIMIT: usize = 64 * 1024;

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff: ExponentialBackoff::default(),
            // Only 429 always retries - server explicitly requests retry
            always_retry: HashSet::from([RetryTrigger::TOO_MANY_REQUESTS]),
            // TransportError and Timeout moved here for safety - only retry idempotent methods
            // or when idempotency key header is present
            idempotent_retry: HashSet::from([
                RetryTrigger::TransportError,
                RetryTrigger::Timeout,
                RetryTrigger::REQUEST_TIMEOUT,
                RetryTrigger::INTERNAL_SERVER_ERROR,
                RetryTrigger::BAD_GATEWAY,
                RetryTrigger::SERVICE_UNAVAILABLE,
                RetryTrigger::GATEWAY_TIMEOUT,
            ]),
            ignore_retry_after: false,
            retry_response_drain_limit: DEFAULT_RETRY_RESPONSE_DRAIN_LIMIT,
            skip_drain_on_retry: false,
            idempotency_key_header: Some(http::header::HeaderName::from_static(
                IDEMPOTENCY_KEY_HEADER_LOWER,
            )),
        }
    }
}

impl RetryConfig {
    /// Create config with no retries
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            max_retries: 0,
            ..Default::default()
        }
    }

    /// Create config with aggressive retry policy (retries all 5xx for any method)
    ///
    /// **WARNING**: This policy retries non-idempotent methods on transport errors
    /// and timeouts, which may cause duplicate side effects. Use with caution.
    #[must_use]
    pub fn aggressive() -> Self {
        Self {
            max_retries: 5,
            backoff: ExponentialBackoff::aggressive(),
            always_retry: HashSet::from([
                RetryTrigger::TransportError,
                RetryTrigger::Timeout,
                RetryTrigger::TOO_MANY_REQUESTS,
                RetryTrigger::REQUEST_TIMEOUT,
                RetryTrigger::INTERNAL_SERVER_ERROR,
                RetryTrigger::BAD_GATEWAY,
                RetryTrigger::SERVICE_UNAVAILABLE,
                RetryTrigger::GATEWAY_TIMEOUT,
            ]),
            idempotent_retry: HashSet::new(),
            ignore_retry_after: false,
            retry_response_drain_limit: DEFAULT_RETRY_RESPONSE_DRAIN_LIMIT,
            skip_drain_on_retry: false,
            idempotency_key_header: Some(http::header::HeaderName::from_static(
                IDEMPOTENCY_KEY_HEADER_LOWER,
            )),
        }
    }

    /// Check if the given trigger should cause a retry for the given HTTP method
    ///
    /// # Arguments
    /// * `trigger` - The condition that triggered the retry consideration
    /// * `method` - The HTTP method of the request
    /// * `has_idempotency_key` - Whether the request has an idempotency key header
    ///
    /// # Retry Logic
    /// - Triggers in `always_retry` are always retried regardless of method
    /// - Triggers in `idempotent_retry` are retried if:
    ///   - The method is idempotent (GET, HEAD, PUT, DELETE, OPTIONS, TRACE), OR
    ///   - The request has an idempotency key header
    #[must_use]
    pub fn should_retry(
        &self,
        trigger: RetryTrigger,
        method: &http::Method,
        has_idempotency_key: bool,
    ) -> bool {
        if self.always_retry.contains(&trigger) {
            return true;
        }
        if self.idempotent_retry.contains(&trigger)
            && (is_idempotent_method(method) || has_idempotency_key)
        {
            return true;
        }
        false
    }
}

/// Rate limiting / concurrency limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum concurrent requests (default: 100)
    pub max_concurrent_requests: usize,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 100,
        }
    }
}

impl RateLimitConfig {
    /// Create config with unlimited concurrency
    #[must_use]
    pub fn unlimited() -> Self {
        Self {
            max_concurrent_requests: usize::MAX,
        }
    }

    /// Create config with very conservative limit
    #[must_use]
    pub fn conservative() -> Self {
        Self {
            max_concurrent_requests: 10,
        }
    }
}

/// Configuration for redirect behavior
///
/// Controls how the HTTP client handles 3xx redirect responses with security protections.
///
/// ## Security Features
///
/// - **Same-origin enforcement**: By default, only follows redirects to the same host
/// - **Header stripping**: Removes `Authorization`, `Cookie` on cross-origin redirects
/// - **Downgrade protection**: Blocks HTTPS → HTTP redirects
/// - **Host allow-list**: Configurable list of trusted redirect targets
///
/// ## Example
///
/// ```rust,ignore
/// use modkit_http::RedirectConfig;
/// use std::collections::HashSet;
///
/// // Permissive mode for general-purpose clients
/// let config = RedirectConfig::permissive();
///
/// // Custom allow-list for trusted hosts
/// let config = RedirectConfig {
///     same_origin_only: true,
///     allowed_redirect_hosts: HashSet::from(["cdn.example.com".to_string()]),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct RedirectConfig {
    /// Maximum number of redirects to follow (default: 10)
    ///
    /// Set to `0` to disable redirect following entirely.
    pub max_redirects: usize,

    /// Only allow same-origin redirects (default: true)
    ///
    /// When `true`, redirects to different hosts are blocked unless the target
    /// host is in `allowed_redirect_hosts`.
    ///
    /// **Security**: This is the safest default, preventing SSRF attacks where
    /// a malicious server redirects requests to internal services.
    pub same_origin_only: bool,

    /// Hosts that are allowed as redirect targets even when `same_origin_only` is true
    ///
    /// Use this to allow redirects to known, trusted hosts (e.g., CDN domains,
    /// authentication servers).
    ///
    /// **Note**: Entries should be hostnames only, without scheme or port.
    /// Example: `"cdn.example.com"`, not `"https://cdn.example.com"`.
    pub allowed_redirect_hosts: HashSet<String>,

    /// Strip sensitive headers on cross-origin redirects (default: true)
    ///
    /// When a redirect goes to a different origin (even if allowed), this removes:
    /// - `Authorization` header (prevents credential leakage)
    /// - `Cookie` header (prevents session hijacking)
    /// - `Proxy-Authorization` header
    ///
    /// **Security**: Always keep this enabled unless you have specific requirements.
    pub strip_sensitive_headers: bool,

    /// Allow HTTPS → HTTP downgrades (default: false)
    ///
    /// When `false`, redirects from HTTPS to HTTP are blocked.
    ///
    /// **Security**: Downgrades expose traffic to interception. Only enable
    /// for testing with local mock servers.
    pub allow_https_downgrade: bool,
}

impl Default for RedirectConfig {
    fn default() -> Self {
        Self {
            max_redirects: 10,
            same_origin_only: true,
            allowed_redirect_hosts: HashSet::new(),
            strip_sensitive_headers: true,
            allow_https_downgrade: false,
        }
    }
}

impl RedirectConfig {
    /// Create a permissive configuration that allows all redirects with header stripping
    ///
    /// This is suitable for general-purpose HTTP clients that need to follow
    /// redirects to any host, but still want protection against credential leakage.
    ///
    /// **Note**: This configuration still blocks HTTPS → HTTP downgrades.
    #[must_use]
    pub fn permissive() -> Self {
        Self {
            max_redirects: 10,
            same_origin_only: false,
            allowed_redirect_hosts: HashSet::new(),
            strip_sensitive_headers: true,
            allow_https_downgrade: false,
        }
    }

    /// Create a configuration that disables redirect following
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            max_redirects: 0,
            ..Default::default()
        }
    }

    /// Create a configuration for testing (allows HTTP, permissive)
    ///
    /// **WARNING**: Only use for local testing with mock servers.
    #[must_use]
    pub fn for_testing() -> Self {
        Self {
            max_redirects: 10,
            same_origin_only: false,
            allowed_redirect_hosts: HashSet::new(),
            strip_sensitive_headers: true, // Still strip headers even in tests
            allow_https_downgrade: true,   // Allow for HTTP mock servers
        }
    }
}

/// TLS root certificate configuration
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum TlsRootConfig {
    /// Use Mozilla's root certificates (webpki-roots, no OS dependency)
    #[default]
    WebPki,
    /// Use OS native root certificate store
    Native,
}

/// Transport security configuration
///
/// Controls whether the client enforces TLS or allows insecure HTTP.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransportSecurity {
    /// Require TLS for all connections (HTTPS only) - default and recommended
    #[default]
    TlsOnly,
    /// Allow insecure HTTP connections (for testing with mock servers only)
    ///
    /// **WARNING**: This should only be used for local testing with mock servers.
    /// Never use in production as it exposes traffic to interception.
    AllowInsecureHttp,
}

/// Overall HTTP client configuration
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// Per-request timeout (default: 30 seconds)
    ///
    /// This timeout applies to each individual HTTP request/attempt.
    /// If retries are enabled, each retry attempt gets its own timeout.
    pub request_timeout: Duration,

    /// Total timeout spanning all retry attempts (default: None)
    ///
    /// When set, the entire operation (including all retries and backoff delays)
    /// must complete within this duration. If the deadline is exceeded,
    /// the request fails with `HttpError::DeadlineExceeded(total_timeout)`.
    ///
    /// When `None`, there is no total deadline - each attempt can take up to
    /// `request_timeout`, and retries can continue indefinitely within their limits.
    pub total_timeout: Option<Duration>,

    /// Maximum response body size in bytes (default: 10 MB)
    pub max_body_size: usize,

    /// User-Agent header value (default: "modkit-http/1.0")
    pub user_agent: String,

    /// Retry policy configuration
    pub retry: Option<RetryConfig>,

    /// Rate limiting / concurrency configuration
    pub rate_limit: Option<RateLimitConfig>,

    /// Transport security mode (default: `TlsOnly`)
    ///
    /// Use `AllowInsecureHttp` only for testing with local mock servers.
    pub transport: TransportSecurity,

    /// TLS root certificate strategy (default: `WebPki`)
    pub tls_roots: TlsRootConfig,

    /// Enable OpenTelemetry tracing layer (default: false)
    /// Creates spans for outbound requests and injects trace context headers.
    pub otel: bool,

    /// Buffer capacity for concurrent request handling (default: 1024)
    ///
    /// The HTTP client uses an internal buffer to allow multiple concurrent
    /// requests without external locking. This sets the maximum number of
    /// requests that can be queued waiting for processing.
    pub buffer_capacity: usize,

    /// Redirect policy configuration (default: same-origin only with header stripping)
    ///
    /// Controls how 3xx redirect responses are handled with security protections:
    /// - Same-origin enforcement (SSRF protection)
    /// - Sensitive header stripping on cross-origin redirects
    /// - HTTPS downgrade protection
    ///
    /// Use `RedirectConfig::permissive()` for general-purpose HTTP client behavior
    /// that allows cross-origin redirects with header stripping.
    ///
    /// Use `RedirectConfig::disabled()` to turn off redirect following entirely.
    pub redirect: RedirectConfig,

    /// Timeout for idle connections in the pool (default: 90 seconds)
    ///
    /// Connections that remain idle (unused) for longer than this duration
    /// will be closed and removed from the pool. This prevents resource leaks
    /// and ensures connections don't become stale.
    ///
    /// Set to `None` to use hyper-util's default idle timeout.
    pub pool_idle_timeout: Option<Duration>,

    /// Maximum number of idle connections per host (default: 32)
    ///
    /// Limits how many idle connections are kept in the pool for each host.
    /// Setting this to `0` disables connection reuse entirely.
    /// Setting this too high may waste resources on rarely-used connections.
    ///
    /// **Note**: This only limits *idle* connections. Active connections are
    /// not limited by this setting.
    pub pool_max_idle_per_host: usize,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
            total_timeout: None,
            max_body_size: 10 * 1024 * 1024, // 10 MB
            user_agent: DEFAULT_USER_AGENT.to_owned(),
            retry: Some(RetryConfig::default()),
            rate_limit: Some(RateLimitConfig::default()),
            transport: TransportSecurity::TlsOnly,
            tls_roots: TlsRootConfig::default(),
            otel: false,
            buffer_capacity: 1024,
            redirect: RedirectConfig::default(),
            pool_idle_timeout: Some(Duration::from_secs(90)),
            pool_max_idle_per_host: 32,
        }
    }
}

impl HttpClientConfig {
    /// Create minimal configuration (no retry, no rate limit, small timeout)
    #[must_use]
    pub fn minimal() -> Self {
        Self {
            request_timeout: Duration::from_secs(10),
            total_timeout: None,
            max_body_size: 1024 * 1024, // 1 MB
            user_agent: DEFAULT_USER_AGENT.to_owned(),
            retry: None,
            rate_limit: None,
            transport: TransportSecurity::TlsOnly,
            tls_roots: TlsRootConfig::default(),
            otel: false,
            buffer_capacity: 256,
            redirect: RedirectConfig::default(),
            pool_idle_timeout: Some(Duration::from_secs(30)),
            pool_max_idle_per_host: 8,
        }
    }

    /// Create configuration for infrastructure services (aggressive retry, large timeout)
    #[must_use]
    pub fn infra_default() -> Self {
        Self {
            request_timeout: Duration::from_secs(60),
            total_timeout: None,
            max_body_size: 50 * 1024 * 1024, // 50 MB
            user_agent: DEFAULT_USER_AGENT.to_owned(),
            retry: Some(RetryConfig::aggressive()),
            rate_limit: Some(RateLimitConfig::default()),
            transport: TransportSecurity::TlsOnly,
            tls_roots: TlsRootConfig::default(),
            otel: false,
            buffer_capacity: 1024,
            redirect: RedirectConfig::default(),
            pool_idle_timeout: Some(Duration::from_secs(120)),
            pool_max_idle_per_host: 64,
        }
    }

    /// Create configuration for `OAuth2` token endpoints (conservative retry)
    ///
    /// Token endpoints use POST but are effectively idempotent for retry purposes:
    /// - Getting a token twice is safe (you'd just use the second one)
    /// - Transport errors before response mean no token was issued
    ///
    /// This config retries on transport errors, timeout, and 429 for all methods.
    #[must_use]
    pub fn token_endpoint() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
            total_timeout: None,
            max_body_size: 1024 * 1024, // 1 MB
            user_agent: DEFAULT_USER_AGENT.to_owned(),
            retry: Some(RetryConfig {
                max_retries: 3,
                // For token endpoints: retry transport errors, timeout, and 429
                // Note: Token requests (POST) are effectively idempotent - getting
                // a token twice is safe, so we put these in always_retry
                always_retry: HashSet::from([
                    RetryTrigger::TransportError,
                    RetryTrigger::Timeout,
                    RetryTrigger::TOO_MANY_REQUESTS,
                ]),
                idempotent_retry: HashSet::new(), // No additional retries for 5xx
                ignore_retry_after: false,
                retry_response_drain_limit: DEFAULT_RETRY_RESPONSE_DRAIN_LIMIT,
                idempotency_key_header: None, // Not needed - always_retry handles all cases
                ..RetryConfig::default()
            }),
            rate_limit: Some(RateLimitConfig::conservative()),
            transport: TransportSecurity::TlsOnly,
            tls_roots: TlsRootConfig::default(),
            otel: false,
            buffer_capacity: 256,
            redirect: RedirectConfig::default(),
            pool_idle_timeout: Some(Duration::from_secs(60)),
            pool_max_idle_per_host: 4,
        }
    }

    /// Create configuration for testing with mock servers (allows insecure HTTP)
    ///
    /// **WARNING**: This configuration allows plain HTTP connections.
    /// Use only for local testing with mock servers, never in production.
    #[must_use]
    pub fn for_testing() -> Self {
        Self {
            request_timeout: Duration::from_secs(10),
            total_timeout: None,
            max_body_size: 1024 * 1024, // 1 MB
            user_agent: DEFAULT_USER_AGENT.to_owned(),
            retry: None,
            rate_limit: None,
            transport: TransportSecurity::AllowInsecureHttp,
            tls_roots: TlsRootConfig::default(),
            otel: false,
            buffer_capacity: 256,
            redirect: RedirectConfig::for_testing(),
            pool_idle_timeout: Some(Duration::from_secs(10)),
            pool_max_idle_per_host: 4,
        }
    }

    /// Create configuration optimized for Server-Sent Events (SSE) streaming.
    ///
    /// SSE connections are long-lived HTTP requests where the server holds the
    /// connection open and pushes events. This preset disables retry and rate
    /// limiting, and sets a permissive request timeout.
    ///
    /// # Timeout behavior
    ///
    /// `request_timeout` is set to 24 hours rather than truly unlimited,
    /// because `TimeoutLayer` requires a finite `Duration`. Override if needed:
    ///
    /// ```rust,ignore
    /// let mut config = HttpClientConfig::sse();
    /// config.request_timeout = Duration::from_secs(3600); // 1 hour
    /// let client = HttpClientBuilder::with_config(config).build()?;
    /// ```
    ///
    /// # Streaming
    ///
    /// Use [`HttpResponse::into_body()`] for streaming — it bypasses the
    /// `max_body_size` limit. SSE reconnection with `Last-Event-ID` is the
    /// caller's responsibility.
    ///
    /// ```rust,ignore
    /// let client = HttpClientBuilder::with_config(HttpClientConfig::sse()).build()?;
    ///
    /// let response = client
    ///     .get("https://api.example.com/events")
    ///     .header("accept", "text/event-stream")
    ///     .send()
    ///     .await?;
    ///
    /// let mut body = response.into_body();
    /// while let Some(frame) = body.frame().await {
    ///     let frame = frame?;
    ///     if let Some(chunk) = frame.data_ref() {
    ///         // parse SSE event data
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn sse() -> Self {
        Self {
            request_timeout: Duration::from_secs(86_400), // 24 hours
            total_timeout: None,
            max_body_size: 10 * 1024 * 1024, // 10 MB (only for bytes()/json(), not into_body())
            user_agent: DEFAULT_USER_AGENT.to_owned(),
            retry: None, // SSE reconnection is protocol-level (Last-Event-ID)
            rate_limit: None,
            transport: TransportSecurity::TlsOnly,
            tls_roots: TlsRootConfig::default(),
            otel: false,
            buffer_capacity: 64,
            redirect: RedirectConfig::default(),
            pool_idle_timeout: None, // use hyper-util default
            pool_max_idle_per_host: 1,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_retry_trigger_constants() {
        assert_eq!(RetryTrigger::TOO_MANY_REQUESTS, RetryTrigger::Status(429));
        assert_eq!(RetryTrigger::REQUEST_TIMEOUT, RetryTrigger::Status(408));
        assert_eq!(
            RetryTrigger::INTERNAL_SERVER_ERROR,
            RetryTrigger::Status(500)
        );
        assert_eq!(RetryTrigger::BAD_GATEWAY, RetryTrigger::Status(502));
        assert_eq!(RetryTrigger::SERVICE_UNAVAILABLE, RetryTrigger::Status(503));
        assert_eq!(RetryTrigger::GATEWAY_TIMEOUT, RetryTrigger::Status(504));
    }

    #[test]
    fn test_is_idempotent_method() {
        // Idempotent per RFC 9110
        assert!(is_idempotent_method(&http::Method::GET));
        assert!(is_idempotent_method(&http::Method::HEAD));
        assert!(is_idempotent_method(&http::Method::PUT));
        assert!(is_idempotent_method(&http::Method::DELETE));
        assert!(is_idempotent_method(&http::Method::OPTIONS));
        assert!(is_idempotent_method(&http::Method::TRACE));
        // Non-idempotent
        assert!(!is_idempotent_method(&http::Method::POST));
        assert!(!is_idempotent_method(&http::Method::PATCH));
    }

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.backoff.initial, Duration::from_millis(100));
        assert_eq!(config.backoff.max, Duration::from_secs(10));
        assert!((config.backoff.multiplier - 2.0).abs() < f64::EPSILON);
        assert!(config.backoff.jitter);

        // Check always_retry defaults - only 429 is always retried
        assert!(
            config
                .always_retry
                .contains(&RetryTrigger::TOO_MANY_REQUESTS)
        );
        assert_eq!(config.always_retry.len(), 1);

        // Check idempotent_retry defaults - includes TransportError and Timeout for safety
        assert!(
            config
                .idempotent_retry
                .contains(&RetryTrigger::TransportError)
        );
        assert!(config.idempotent_retry.contains(&RetryTrigger::Timeout));
        assert!(
            config
                .idempotent_retry
                .contains(&RetryTrigger::REQUEST_TIMEOUT)
        );
        assert!(
            config
                .idempotent_retry
                .contains(&RetryTrigger::INTERNAL_SERVER_ERROR)
        );
        assert!(config.idempotent_retry.contains(&RetryTrigger::BAD_GATEWAY));
        assert!(
            config
                .idempotent_retry
                .contains(&RetryTrigger::SERVICE_UNAVAILABLE)
        );
        assert!(
            config
                .idempotent_retry
                .contains(&RetryTrigger::GATEWAY_TIMEOUT)
        );
        assert_eq!(config.idempotent_retry.len(), 7);

        // Default respects Retry-After header
        assert!(!config.ignore_retry_after);

        // Default drain limit
        assert_eq!(
            config.retry_response_drain_limit,
            DEFAULT_RETRY_RESPONSE_DRAIN_LIMIT
        );

        // Default idempotency key header
        assert_eq!(
            config.idempotency_key_header,
            Some(http::header::HeaderName::from_static(
                IDEMPOTENCY_KEY_HEADER_LOWER
            ))
        );
    }

    #[test]
    fn test_retry_config_disabled() {
        let config = RetryConfig::disabled();
        assert_eq!(config.max_retries, 0);
    }

    #[test]
    fn test_retry_config_aggressive() {
        let config = RetryConfig::aggressive();
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.backoff.initial, Duration::from_millis(50));
        assert_eq!(config.backoff.max, Duration::from_secs(30));
        // Aggressive moves all 5xx to always_retry
        assert!(
            config
                .always_retry
                .contains(&RetryTrigger::INTERNAL_SERVER_ERROR)
        );
        assert!(config.idempotent_retry.is_empty());
    }

    #[test]
    fn test_should_retry_always() {
        let config = RetryConfig::default();

        // 429 always retries regardless of method or idempotency key
        assert!(config.should_retry(RetryTrigger::TOO_MANY_REQUESTS, &http::Method::GET, false));
        assert!(config.should_retry(RetryTrigger::TOO_MANY_REQUESTS, &http::Method::POST, false));
        assert!(config.should_retry(RetryTrigger::TOO_MANY_REQUESTS, &http::Method::POST, true));
    }

    #[test]
    fn test_should_retry_idempotent_only() {
        let config = RetryConfig::default();

        // TransportError retries for idempotent methods only (by default)
        assert!(config.should_retry(RetryTrigger::TransportError, &http::Method::GET, false));
        assert!(!config.should_retry(RetryTrigger::TransportError, &http::Method::POST, false));

        // 500 only retries for idempotent methods
        assert!(config.should_retry(
            RetryTrigger::INTERNAL_SERVER_ERROR,
            &http::Method::GET,
            false
        ));
        assert!(!config.should_retry(
            RetryTrigger::INTERNAL_SERVER_ERROR,
            &http::Method::POST,
            false
        ));

        // 503 only retries for idempotent methods
        assert!(config.should_retry(
            RetryTrigger::SERVICE_UNAVAILABLE,
            &http::Method::HEAD,
            false
        ));
        assert!(!config.should_retry(
            RetryTrigger::SERVICE_UNAVAILABLE,
            &http::Method::POST,
            false
        ));

        // Timeout only retries for idempotent methods
        assert!(config.should_retry(RetryTrigger::Timeout, &http::Method::GET, false));
        assert!(!config.should_retry(RetryTrigger::Timeout, &http::Method::POST, false));
    }

    #[test]
    fn test_should_retry_with_idempotency_key() {
        let config = RetryConfig::default();

        // TransportError retries for non-idempotent methods when idempotency key is present
        assert!(config.should_retry(RetryTrigger::TransportError, &http::Method::POST, true));
        assert!(config.should_retry(RetryTrigger::TransportError, &http::Method::PUT, true));
        assert!(config.should_retry(RetryTrigger::TransportError, &http::Method::DELETE, true));
        assert!(config.should_retry(RetryTrigger::TransportError, &http::Method::PATCH, true));

        // Timeout retries for non-idempotent methods when idempotency key is present
        assert!(config.should_retry(RetryTrigger::Timeout, &http::Method::POST, true));

        // 500 retries for non-idempotent methods when idempotency key is present
        assert!(config.should_retry(
            RetryTrigger::INTERNAL_SERVER_ERROR,
            &http::Method::POST,
            true
        ));
    }

    #[test]
    fn test_should_retry_not_configured() {
        let config = RetryConfig::default();

        // 400 Bad Request is not in any retry set
        assert!(!config.should_retry(RetryTrigger::Status(400), &http::Method::GET, false));
        assert!(!config.should_retry(RetryTrigger::Status(400), &http::Method::POST, false));
        assert!(!config.should_retry(RetryTrigger::Status(400), &http::Method::POST, true)); // Even with idempotency key

        // 404 Not Found is not in any retry set
        assert!(!config.should_retry(RetryTrigger::Status(404), &http::Method::GET, false));
    }

    #[test]
    fn test_rate_limit_config_defaults() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_concurrent_requests, 100);
    }

    #[test]
    fn test_rate_limit_config_unlimited() {
        let config = RateLimitConfig::unlimited();
        assert_eq!(config.max_concurrent_requests, usize::MAX);
    }

    #[test]
    fn test_rate_limit_config_conservative() {
        let config = RateLimitConfig::conservative();
        assert_eq!(config.max_concurrent_requests, 10);
    }

    #[test]
    fn test_http_client_config_defaults() {
        let config = HttpClientConfig::default();
        assert_eq!(config.request_timeout, Duration::from_secs(30));
        assert_eq!(config.max_body_size, 10 * 1024 * 1024);
        assert_eq!(config.user_agent, DEFAULT_USER_AGENT);
        assert!(config.retry.is_some());
        assert!(config.rate_limit.is_some());
        assert_eq!(config.transport, TransportSecurity::TlsOnly);
        assert!(!config.otel);
        assert_eq!(config.buffer_capacity, 1024);
    }

    #[test]
    fn test_http_client_config_minimal() {
        let config = HttpClientConfig::minimal();
        assert_eq!(config.request_timeout, Duration::from_secs(10));
        assert_eq!(config.max_body_size, 1024 * 1024);
        assert!(config.retry.is_none());
        assert!(config.rate_limit.is_none());
    }

    #[test]
    fn test_http_client_config_infra_default() {
        let config = HttpClientConfig::infra_default();
        assert_eq!(config.request_timeout, Duration::from_secs(60));
        assert_eq!(config.max_body_size, 50 * 1024 * 1024);
        assert!(config.retry.is_some());
        assert_eq!(config.retry.unwrap().max_retries, 5);
    }

    #[test]
    fn test_http_client_config_token_endpoint() {
        let config = HttpClientConfig::token_endpoint();
        assert_eq!(config.request_timeout, Duration::from_secs(30));

        let retry = config.retry.unwrap();
        // Token endpoint: no idempotent-only retries (conservative for auth)
        assert!(retry.idempotent_retry.is_empty());
        // But still retry transport errors and 429
        assert!(retry.always_retry.contains(&RetryTrigger::TransportError));
        assert!(
            retry
                .always_retry
                .contains(&RetryTrigger::TOO_MANY_REQUESTS)
        );

        let rate_limit = config.rate_limit.unwrap();
        assert_eq!(rate_limit.max_concurrent_requests, 10); // Conservative
    }

    #[test]
    fn test_http_client_config_for_testing() {
        let config = HttpClientConfig::for_testing();
        assert_eq!(config.transport, TransportSecurity::AllowInsecureHttp);
        assert!(config.retry.is_none());
    }

    #[test]
    fn test_http_client_config_sse() {
        let config = HttpClientConfig::sse();
        assert_eq!(config.request_timeout, Duration::from_secs(86_400));
        assert!(config.total_timeout.is_none());
        assert!(config.retry.is_none());
        assert!(config.rate_limit.is_none());
        assert!(!config.otel);
        assert_eq!(config.buffer_capacity, 64);
        assert!(config.pool_idle_timeout.is_none());
        assert_eq!(config.pool_max_idle_per_host, 1);
    }
}
