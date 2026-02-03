# modkit-http

HTTP client library for ModKit, built on hyper and tower.

## What it does

- HTTPS client with TLS via rustls (WebPki roots by default, no OS dependency)
- Connection pooling via hyper
- Configurable request timeouts
- Automatic retries with exponential backoff (429, 5xx, transport errors, timeouts)
- User-Agent header injection
- Concurrency limiting with fail-fast load shedding
- Response body size limits (applied after decompression)
- **Transparent response decompression** (gzip, brotli, deflate)
- **Secure redirect following** with SSRF protection and credential leakage prevention

## What it does NOT do

- Cookie management
- Request body compression
- Caching
- WebSocket support
- Streaming uploads

## Usage

### Basic GET request

```rust
use cf_modkit_http::HttpClient;
use std::time::Duration;

let client = HttpClient::builder()
    .timeout(Duration::from_secs(10))
    .user_agent("my-app/1.0")
    .build()?;

// RequestBuilder API: chain methods then send
let data: MyResponse = client
    .get("https://api.example.com/data")
    .send()
    .await?
    .json()
    .await?;
```

### POST with form body

```rust
use cf_modkit_http::HttpClient;
use std::time::Duration;

let client = HttpClient::builder()
    .timeout(Duration::from_secs(30))
    .build()?;

let token: TokenResponse = client
    .post("https://auth.example.com/token")
    .header("authorization", "Basic dXNlcjpwYXNz")
    .form(&[
        ("grant_type", "client_credentials"),
        ("client_id", "my-client"),
    ])?
    .send()
    .await?
    .json()
    .await?;
```

## Transparent Decompression

The client automatically handles compressed responses:

- **Sends `Accept-Encoding: gzip, br, deflate`** on all requests
- **Decompresses response bodies** based on `Content-Encoding` header
- **Body size limit applies to decompressed bytes**, protecting against "zip bombs"

This is enabled by default with no configuration required. Example:

```rust
// Server sends gzip-compressed JSON with Content-Encoding: gzip
// Client automatically decompresses before parsing
let data: MyResponse = client
    .get("https://api.example.com/data")
    .send()
    .await?
    .json()
    .await?;
```

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `timeout` | 30s | Request timeout |
| `max_body_size` | 10 MB | Maximum response body size (after decompression) |
| `user_agent` | `modkit-http/1.0` | User-Agent header value |
| `retry` | Enabled (3 retries) | Retry on 429, 5xx (for idempotent methods), transport errors |
| `rate_limit` | 100 concurrent | Maximum concurrent requests |
| `redirect` | Same-origin, 10 max | Redirect policy with security controls (see below) |
| `pool_idle_timeout` | 90s | Idle connection timeout (None to disable) |
| `pool_max_idle_per_host` | 32 | Max idle connections per host |
| `transport` | `TlsOnly` | Transport security mode (`TlsOnly` or `AllowInsecureHttp` for testing) |
| `tls_roots` | WebPki | TLS root certificate source |

### Configuration presets

```rust
use cf_modkit_http::{HttpClientBuilder, HttpClientConfig};

// Minimal: no retry, no rate limit, 10s timeout
let client = HttpClientBuilder::with_config(HttpClientConfig::minimal()).build()?;

// Infrastructure: aggressive retry, 60s timeout, 50MB body limit
let client = HttpClientBuilder::with_config(HttpClientConfig::infra_default()).build()?;

// Token endpoint: conservative rate limit, no retry on POST
let client = HttpClientBuilder::with_config(HttpClientConfig::token_endpoint()).build()?;

// Testing: allows plain HTTP for mock servers
let client = HttpClientBuilder::with_config(HttpClientConfig::for_testing()).build()?;

// SSE streaming: 24h timeout, no retry, no rate limit
let client = HttpClientBuilder::with_config(HttpClientConfig::sse()).build()?;
```

## Redirect Security

By default, `modkit-http` follows redirects with security protections:

| Protection | Default | Description                                                 |
|------------|---------|-------------------------------------------------------------|
| Same-origin only | Yes | Follows redirects to the same host                          |
| Header stripping | Yes | Removes `Authorization`, `Cookie` on cross-origin redirects |
| HTTPS downgrade | Blocked | Blocks redirects from HTTPS to HTTP                         |
| Max redirects | 10 | Stops after 10 redirects                                    |

### Redirect policies

```rust
use cf_modkit_http::{HttpClient, RedirectConfig};

// Default: same-origin only (most secure)
let client = HttpClient::builder().build()?;

// Permissive: follow all redirects, but strip credentials on cross-origin
let client = HttpClient::builder()
    .redirect(RedirectConfig::permissive())
    .build()?;

// Disable redirects entirely
let client = HttpClient::builder()
    .redirect(RedirectConfig::disabled())
    .build()?;

// Custom: allow specific trusted hosts
use std::collections::HashSet;
let config = RedirectConfig {
    same_origin_only: true,
    allowed_redirect_hosts: HashSet::from(["cdn.example.com".to_string()]),
    ..Default::default()
};
let client = HttpClient::builder().redirect(config).build()?;
```

## Retry Behavior

The default retry policy:

| Trigger | Retried for | Notes |
|---------|-------------|-------|
| 429 Too Many Requests | All methods | Server explicitly requests retry |
| 408, 500, 502, 503, 504 | Idempotent methods (GET, HEAD, PUT, DELETE, OPTIONS, TRACE) | Or with `Idempotency-Key` header |
| Transport errors | Idempotent methods | Connection failures, resets |
| Timeouts | Idempotent methods | Per-attempt timeout exceeded |

Non-idempotent methods (POST, PATCH) are only retried on 429 by default to avoid duplicate side effects. To enable broader retry for these methods, include an `Idempotency-Key` header.

## Error handling

All operations return `HttpError`:

| Variant | Retryable | Description |
|---------|-----------|-------------|
| `TimeoutAttempt` | Yes | Single request attempt exceeded timeout |
| `DeadlineExceeded` | No | Total operation deadline exceeded (all retries) |
| `Transport` | Yes | Network/connection error |
| `HttpStatus` | No | Non-2xx HTTP response (from `error_for_status()` or `json()`) |
| `Json` | No | JSON parsing failed |
| `BodyTooLarge` | No | Response exceeded size limit |
| `Tls` | No | TLS certificate/setup error |
| `Overloaded` | No | Concurrency limit reached (fail-fast) |
| `ServiceClosed` | No | Internal service failure (buffer worker died) |

## License

Apache-2.0
