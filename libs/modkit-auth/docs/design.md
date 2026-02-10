## Detailed design

### Dependency policy

`modkit-auth` depends on `aliri_tokens` without default features and without `oauth2`:

```toml
aliri_tokens = { version = "0.3", default-features = false, features = ["rand"] }
```

`modkit-auth` depends on `modkit-http` for token endpoint HTTP calls. This is acceptable because `modkit-http` is hyper-based and does not pull `reqwest`.

### Public API

Modules use a long-lived token capability:

```rust
#[derive(Clone)]
pub struct Token {
    inner: std::sync::Arc<TokenInner>,
}

impl Token {
    pub fn get(&self) -> Result<SecretString, TokenError>;
    pub async fn invalidate(&self);
}
```

Rules:

- modules never see client credentials
- modules never store the token string beyond request construction
- token is returned only as a transient `SecretString`

### Token configuration

**Required:**

- `token_endpoint` or `issuer_url`
- `client_id`, `client_secret`
- `scopes` (normalized once, stable order)

**Optional:**

- extra headers for vendor quirks
- client auth method: `Basic` (default) or `form`
- refresh policy:
    - refresh offset (default 30 minutes)
    - jitter max (default 5 minutes)
    - minimum refresh period (default 10 seconds)
- safe default TTL if `expires_in` is missing
- `HttpClientConfig` override for the token endpoint client (defaults to `HttpClientConfig::token_endpoint()`)

### Token source (modkit-http based)

Implement a custom `AsyncTokenSource` for `aliri_tokens`.

The token source holds an `HttpClient` instance built with `HttpClientConfig::token_endpoint()`. This client provides:

- retry with exponential backoff and jitter (transport errors, timeout, 429)
- concurrency limiting (10 concurrent requests)
- 30s request timeout
- 1 MB response body limit
- OTel tracing (when `otel` feature is enabled)
- TLS-only transport (no accidental plaintext token requests)

**Responsibilities:**

- resolve token endpoint:
    - use direct `token_endpoint`, or
    - OIDC discovery from `issuer_url` using `/.well-known/openid-configuration`
- do client credentials grant:
    - `grant_type=client_credentials`
    - optional `scope` (space-separated)
    - `Basic` auth or form-based auth
    - uses `HttpClient::post(url).form(&fields)` for the token request
- attach extra headers (if configured) via `RequestBuilder::header()`
- parse response via `HttpResponse::json::<TokenResponse>()`:
    - `access_token` (required)
    - `expires_in` (optional)
    - `token_type` (optional)

**Behavior rules:**

- missing `expires_in` uses a safe default TTL (prevents hot loops)
- if `token_type` exists and is not `Bearer`, fail token acquisition
- HTTP error handling: `HttpClient` already retries on transport errors, timeouts, and 429; the token source maps non-2xx responses to `TokenError` using `HttpResponse::error_for_status()`
- `Retry-After` header parsing uses `modkit_http::response::parse_retry_after()` when available in error context
- sanitize errors: never log or include secrets (`client_secret`, token value) in error text

### Token watcher

Use `aliri_tokens` watcher created from the token source.

Watcher responsibilities:

- in-memory caching
- refresh scheduling
- jitter application
- error backoff
- concurrency control (avoid duplicate refresh under load)
- optionally serve current token while refresh is in progress

### Invalidate semantics

`aliri_tokens` watcher has no explicit "drop cached token now" API.

Implementation:

- store the active watcher behind `ArcSwap` (preferred) or `RwLock<Arc<_>>`
- `Token::invalidate()` creates a new watcher (same config) and swaps it in
- old watcher is dropped when no longer referenced
- next `get()` triggers a new fetch via the token source

### Outbound auth layer (tower)

Implement a tower layer that injects the `Authorization` header.

**Behavior:**

- on each request:
    - call `Token::get()`
    - set `Authorization: Bearer <token>`
    - forward request
- configurable header name if a vendor requires a non-standard header

**Integration with `HttpClient`:** The auth layer is composed into the `HttpClient` tower stack at build time using a decoupled architecture:

- `modkit-http` provides a generic `HttpClientBuilder::with_auth_layer()` hook that accepts any tower layer. This keeps modkit-http free of OAuth-specific knowledge and avoids circular dependencies.
- `modkit-auth` provides an extension trait `HttpClientBuilderExt` with `.with_bearer_auth(token)` and `.with_bearer_auth_header(token, header_name)` methods. These wrap `BearerAuthLayer` and call `with_auth_layer()` internally.

The auth layer is inserted between retry and timeout in the tower stack, so each retry re-acquires the token.

### Integration rules

- Token endpoint calls use a dedicated `HttpClient` built with `HttpClientConfig::token_endpoint()`. This client is internal to the token source and is not shared with module business logic.
- Module outbound requests use `HttpClient` with the auth tower layer composed in. Tracing (OTel), retries, concurrency limits, and other middleware are already part of the `HttpClient` stack.
- No migration or coexistence concerns — `TracedClient` has been removed; all outbound HTTP uses `modkit-http::HttpClient`.
