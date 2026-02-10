# Implementation Plan — Outbound OAuth2 Client Credentials

Reference: [0002-modkit-auth-client.md](./0002-modkit-auth-client.md)
Required reading: [ModKit unified system](/docs/modkit_unified_system/README.md)

Crate: `libs/modkit-auth`

## Existing code to reuse

Before starting, review the existing JWKS provider at
`libs/modkit-auth/src/providers/jwks.rs`. It is the closest analog to the OAuth2
token source and establishes patterns that must be followed:

| Concern | JWKS (existing) | OAuth2 (new) |
|---------|----------------|--------------|
| HTTP client | `HttpClient::builder().timeout().retry(None).build()` | `HttpClientConfig::token_endpoint()` |
| Lock-free cache | `Arc<ArcSwap<HashMap<String,DecodingKey>>>` | `ArcSwap<TokenInner>` |
| Background refresh | `run_jwks_refresh_task` + `CancellationToken` | `aliri_tokens` watcher |
| Backoff | Manual exponential in `calculate_backoff()` | Delegated to `aliri_tokens` |
| Error mapping | `map_http_error(HttpError) -> ClaimsError` | Extract shared `map_http_error` or create parallel `map_http_error(HttpError) -> TokenError` |
| Test scaffold | `httpmock` + `allow_insecure_http()` | Same pattern |

Key reuse points:

1. **`map_http_error`** — `jwks.rs:417-473` maps every `HttpError` variant.
   Extract it into a shared helper `src/providers/http_error_mapping.rs` or
   duplicate the pattern for `TokenError`. Extraction is preferred to keep the
   exhaustive match in one place (HttpError is `#[non_exhaustive]`).
2. **`ArcSwap` lock-free read pattern** — `jwks.rs` uses `keys.load()` for
   reads and `keys.store(Arc::new(...))` for writes. Token must follow the
   same pattern.
3. **`HttpClient` construction** — `jwks.rs:86-89` builds a client with no
   retries (JWKS handles its own). OAuth source uses
   `HttpClientConfig::token_endpoint()` instead (retries are appropriate for
   token acquisition).
4. **Test pattern** — `jwks.rs:482-499` (`test_provider_with_http`) shows
   how to build a test `HttpClient` with `allow_insecure_http()` for
   `httpmock`. Reuse the same approach.
5. **Dependencies already available** — `modkit-auth/Cargo.toml` already has
   `arc-swap`, `modkit-http`, `tokio-util`, `httpmock` (dev). Only
   `aliri_tokens` needs to be added.

---

## Phase 0 — Extract shared HTTP error mapping

**Goal:** Extract `map_http_error` from JWKS into a reusable utility so both JWKS and OAuth2 can use it without duplicating the exhaustive match.

### Prompt

```
Extract the HTTP error mapping function from the JWKS provider into a shared
utility in libs/modkit-auth.

Current state: `libs/modkit-auth/src/providers/jwks.rs` lines 417-473 contain
`fn map_http_error(e: modkit_http::HttpError) -> ClaimsError` that exhaustively
maps every HttpError variant. This pattern will be needed by the OAuth2 token
source as well.

1. Create `src/http_error.rs` with a generic mapping function:

   pub fn format_http_error(e: &modkit_http::HttpError, prefix: &str) -> String

   This function takes an HttpError reference and a context prefix (e.g. "JWKS",
   "OAuth2 token") and returns a human-readable error message string. It must
   handle every variant of HttpError (which is #[non_exhaustive]) with a
   catch-all `_ =>` arm at the end.

   Replicate the exact mapping logic from jwks.rs:417-473, but replace the
   hardcoded "JWKS" prefix with the `prefix` parameter.

2. Update `src/providers/jwks.rs`:
   - Replace the local `map_http_error` function with a call to the shared one:

     fn map_http_error(e: modkit_http::HttpError) -> ClaimsError {
         ClaimsError::JwksFetchFailed(crate::http_error::format_http_error(&e, "JWKS"))
     }

   - Delete the old exhaustive match from jwks.rs.

3. Wire `pub mod http_error` into src/lib.rs.

4. Run `cargo test -p modkit-auth` — all existing JWKS tests must pass unchanged.

The OAuth2 token source (Phase 2) will use the same shared function:
    TokenError::Http(crate::http_error::format_http_error(&e, "OAuth2 token"))
```

### Guard

```
Verify Phase 0 (shared HTTP error mapping) in libs/modkit-auth.

Check every item:

1. FILE EXISTS: src/http_error.rs with pub fn format_http_error.

2. EXHAUSTIVE MATCH: format_http_error handles these HttpError variants:
   HttpStatus, Json, Timeout, DeadlineExceeded, Transport, BodyTooLarge,
   Tls, RequestBuild, InvalidHeaderName, InvalidHeaderValue, FormEncode,
   Overloaded, ServiceClosed, InvalidUri, InvalidScheme, plus a `_ =>` arm.

3. PREFIX USED: Every match arm includes the prefix parameter in its output.
   e.g. "{prefix} HTTP {status}" not "JWKS HTTP {status}".

4. JWKS UNCHANGED: jwks.rs now calls format_http_error(&e, "JWKS").
   The local exhaustive match is removed.
   Run `cargo test -p modkit-auth` — all 11 existing JWKS tests pass.

5. NO SECRET LEAKS: format_http_error never includes request body, auth
   headers, or token values — only status codes, error messages, and URLs.

6. COMPILATION: `cargo check -p modkit-auth` succeeds with no warnings.
```

---

## Phase 1 — Foundation types

**Goal:** Define configuration, error, and response types. No async code yet.

### Prompt

```
Implement foundation types for outbound OAuth2 client credentials in
libs/modkit-auth.

Create a new module `src/oauth2/` with:

1. `src/oauth2/mod.rs` — re-exports.

2. `src/oauth2/config.rs` — `OAuthClientConfig`:
   - `token_endpoint: Option<Url>` (direct endpoint)
   - `issuer_url: Option<Url>` (for OIDC discovery, Phase 6)
   - Exactly one of `token_endpoint` or `issuer_url` must be set; validate in a
     `fn validate(&self) -> Result<(), TokenError>` method.
   - `client_id: String`
   - `client_secret: SecretString` (use the SecretString from this module)
   - `scopes: Vec<String>`
   - `auth_method: ClientAuthMethod` enum { Basic, Form } (default Basic)
   - `extra_headers: Vec<(String, String)>` (default empty)
   - `refresh_offset: Duration` (default 30 min)
   - `jitter_max: Duration` (default 5 min)
   - `min_refresh_period: Duration` (default 10 sec)
   - `default_ttl: Duration` (default 5 min — used when expires_in missing)
   - `http_config: Option<modkit_http::HttpClientConfig>` (override for token
     endpoint client; defaults to HttpClientConfig::token_endpoint())
   All durations use `std::time::Duration`. Derive `Clone`. Do NOT derive `Debug`
   for the whole struct — implement `Debug` manually, redacting `client_secret`.

3. `src/oauth2/error.rs` — `TokenError` enum (thiserror):
   - `Http(String)` — transport/status errors (store formatted message, not
     the raw HttpError, to avoid leaking internals; use the shared
     `format_http_error` from Phase 0 when constructing)
   - `InvalidResponse(String)` — malformed token response
   - `UnsupportedTokenType(String)` — token_type is not Bearer
   - `ConfigError(String)` — invalid config (e.g. both endpoints set)
   - `Unavailable(String)` — watcher not ready / shutdown
   Mark `#[non_exhaustive]`. Implement Display via thiserror. Ensure no variant
   can ever contain secret values.
   Follow the same pattern as `AuthError` in src/errors.rs.

4. `src/oauth2/types.rs`:
   - `SecretString` — newtype over `String` with `Debug` printing "[REDACTED]",
     `Display` printing "[REDACTED]", `Clone`, and `Drop` that zeroizes the
     inner buffer (use `unsafe { core::ptr::write_bytes }` on the Vec<u8>
     backing buffer, or the `zeroize` crate if available in workspace).
     Provide `fn expose(&self) -> &str` for controlled access.
   - `TokenResponse` (serde `Deserialize`):
     - `access_token: String`
     - `expires_in: Option<u64>`
     - `token_type: Option<String>`
     Deserialize only — never derive Serialize. Use `#[serde(default)]` for
     optional fields.
   - `ClientAuthMethod` enum { Basic, Form } with `Default` = Basic.

Wire `mod oauth2` into `src/lib.rs` and re-export `OAuthClientConfig`,
`TokenError`, `SecretString`, `ClientAuthMethod` from the crate root
(Token will be re-exported in Phase 3).

Follow existing patterns in modkit-auth: thiserror for errors, no unwrap in
library code, #[non_exhaustive] on public enums.
```

### Guard

```
Verify Phase 1 (foundation types) of the OAuth2 client credentials
implementation in libs/modkit-auth/src/oauth2/.

Check every item:

1. FILES EXIST: mod.rs, config.rs, error.rs, types.rs under src/oauth2/.
   `mod oauth2` is declared in src/lib.rs.

2. SecretString:
   - Debug and Display both print "[REDACTED]", never the inner value.
   - `expose(&self) -> &str` is the only way to access the inner value.
   - Clone is implemented.
   - Drop zeroizes the buffer (write_bytes or zeroize).
   - Write a unit test: create a SecretString, format with Debug and Display,
     assert neither contains the original value. Assert expose() returns it.

3. OAuthClientConfig:
   - Has all fields from the design (token_endpoint, issuer_url, client_id,
     client_secret, scopes, auth_method, extra_headers, refresh_offset,
     jitter_max, min_refresh_period, default_ttl, http_config).
   - Does NOT derive Debug — has manual Debug impl that redacts client_secret.
   - validate() returns error if both token_endpoint and issuer_url are set,
     or if neither is set.
   - Write a unit test for validate() with both, neither, and exactly-one.

4. TokenError:
   - Is #[non_exhaustive].
   - Has variants: Http, InvalidResponse, UnsupportedTokenType, ConfigError,
     Unavailable.
   - Display output for Http variant does NOT leak secrets.
   - Write a test: format a TokenError::ConfigError, assert it renders.

5. TokenResponse:
   - Deserializes `{"access_token":"x","expires_in":3600,"token_type":"Bearer"}`.
   - Deserializes `{"access_token":"x"}` (optional fields missing).
   - Does NOT implement Serialize.
   - Write tests for both cases.

6. ClientAuthMethod: Default is Basic.

7. COMPILATION: `cargo check -p modkit-auth` succeeds with no warnings.

8. TESTS: `cargo test -p modkit-auth` — all new AND existing tests pass.

9. NO SECRETS IN DEBUG: grep all new files for `impl Debug` — every manual
   impl redacts client_secret / token values. No `#[derive(Debug)]` on types
   containing secrets.
```

---

## Phase 2 — Token source

**Goal:** Implement `AsyncTokenSource` for `aliri_tokens` using `modkit-http::HttpClient`.

### Prompt

```
Implement the OAuth2 token source in libs/modkit-auth/src/oauth2/source.rs.

This struct implements aliri_tokens::AsyncTokenSource and performs the actual
OAuth2 client credentials exchange using modkit-http::HttpClient.

REFERENCE: Study libs/modkit-auth/src/providers/jwks.rs for the established
patterns — especially how it constructs HttpClient (lines 86-89), how it calls
client.get().send().await (lines 131-139), and how it maps errors via the
shared format_http_error (Phase 0).

1. Add `aliri_tokens` to modkit-auth/Cargo.toml:
   aliri_tokens = { version = "0.3", default-features = false, features = ["rand"] }
   Verify it does NOT pull reqwest (no oauth2 feature).

2. Create `src/oauth2/source.rs` with struct `OAuthTokenSource`:
   - Fields:
     - `client: modkit_http::HttpClient` (built once from config)
     - `token_endpoint: Url`
     - `client_id: String`
     - `client_secret: SecretString`
     - `scopes: Option<String>` (pre-joined with spaces, None if empty)
     - `auth_method: ClientAuthMethod`
     - `extra_headers: Vec<(String, String)>`
     - `default_ttl: Duration`

   - Constructor `fn new(config: &OAuthClientConfig) -> Result<Self, TokenError>`:
     - Build HttpClient from config.http_config (if set) or
       HttpClientConfig::token_endpoint() (default).
       Map the HttpError from build via format_http_error (Phase 0):
       `TokenError::Http(format_http_error(&e, "OAuth2 token"))`
     - For now, require config.token_endpoint to be Some (OIDC discovery is
       Phase 6). Return TokenError::ConfigError if missing.
     - Pre-join scopes into a space-separated string.

   COMPARE with JWKS: JwksKeyProvider::new() builds the client at lines 86-89
   with custom timeout and no retries. Here we use token_endpoint() config
   which INCLUDES retries (appropriate for token POST, which is idempotent
   for client_credentials).

3. Implement `aliri_tokens::AsyncTokenSource` for `OAuthTokenSource`:
   - `async fn request_token(&self) -> Result<TokenWithLifetime, BoxError>`:
     a. Build form fields: grant_type=client_credentials, optional scope.
     b. Build request via `self.client.post(url).form(&fields)?`
        COMPARE with JWKS: jwks.rs uses `self.client.get(&self.jwks_uri).send()`
        at line 131. Here we use POST with form body.
     c. If auth_method is Basic: add Authorization header with
        base64(client_id:client_secret) via .header().
        If Form: add client_id and client_secret to form fields instead.
     d. Add extra_headers via .header().
     e. Call .send().await, then error_for_status().
     f. Parse response as TokenResponse via .json().
        COMPARE with JWKS: jwks.rs at line 139 does the same `.json().await`.
     g. Validate: if token_type is Some and != "Bearer" (case-insensitive),
        return Err(UnsupportedTokenType).
     h. Compute lifetime: Duration::from_secs(expires_in) if present,
        else self.default_ttl.
     i. Return TokenWithLifetime { token: access_token, lifetime }.

   - Map HttpError using format_http_error(&e, "OAuth2 token") (Phase 0).
     Never include client_secret in error messages.

4. Wire source.rs into src/oauth2/mod.rs. OAuthTokenSource is pub(crate) —
   it is an internal implementation detail used by Token (Phase 3).

DO NOT add:
- Manual retry logic (HttpClient retries via RetryLayer).
- Manual tracing spans (HttpClient traces via OtelLayer).
- Manual timeout logic (HttpClient has request_timeout).
- Manual TLS configuration (HttpClient enforces TLS-only).
```

### Guard

```
Verify Phase 2 (token source) of the OAuth2 client credentials implementation.

Check every item:

1. DEPENDENCY: `aliri_tokens` in modkit-auth/Cargo.toml with
   `default-features = false, features = ["rand"]`.
   Run: `cargo tree -p cf-modkit-auth | grep reqwest` — must return NOTHING.

2. OAuthTokenSource: all fields private. Visibility is pub(crate).

3. Constructor:
   - Builds HttpClient from HttpClientConfig::token_endpoint() by default.
   - Accepts http_config override from OAuthClientConfig.
   - Maps HttpError from client build via format_http_error.
   - Returns TokenError::ConfigError if token_endpoint is None.

4. AsyncTokenSource::request_token():
   - Form body includes grant_type=client_credentials.
   - Scope included only when non-empty.
   - Basic auth: header is "Basic {base64(id:secret)}".
   - Form auth: client_id + client_secret in form body, no Authorization header.
   - Extra headers applied.
   - error_for_status() called before json().
   - token_type validation case-insensitive.
   - Missing expires_in falls back to default_ttl.
   - Returned lifetime is Duration, not zero.

5. SECRET SAFETY:
   - grep source.rs for "client_secret" — only in struct field and
     form body / Authorization header construction. Never in error messages,
     tracing, or Debug output.
   - access_token is never logged.

6. ERROR MAPPING uses shared format_http_error from Phase 0
   (not a local exhaustive match).

7. NO DUPLICATE CONCERNS:
   - No manual retry logic.
   - No manual tracing spans.
   - No manual timeout logic.
   - No manual TLS configuration.

8. COMPILATION: `cargo check -p modkit-auth` succeeds.

9. TESTS (use httpmock + allow_insecure_http, same pattern as
   jwks.rs test_provider_with_http at line 482):
   - Mock server returns valid token response → assert access_token + lifetime.
   - Mock returns response without expires_in → assert default_ttl used.
   - Mock returns token_type: "mac" → assert error.
   - Empty scopes → assert scope param absent from request body.
   - Basic auth → assert Authorization header present in mock request.
   - Form auth → assert client_id in form body, no Authorization header.
```

---

## Phase 3 — Token handle (watcher + invalidate)

**Goal:** Implement the public `Token` struct that wraps `aliri_tokens` watcher with `ArcSwap`-based invalidation.

### Prompt

```
Implement the public Token handle in libs/modkit-auth/src/oauth2/token.rs.

This is the public API that modules use to get bearer tokens. It wraps an
aliri_tokens watcher and supports invalidation via ArcSwap rotation.

REFERENCE: Study the ArcSwap usage in libs/modkit-auth/src/providers/jwks.rs:
- jwks.rs:41 — `keys: Arc<ArcSwap<HashMap<String, DecodingKey>>>`
- jwks.rs:190 — `self.keys.store(Arc::new(new_keys))` for atomic swap
- jwks.rs:299 — `let keys = self.keys.load()` for lock-free read
The Token handle follows the same pattern but swaps the entire watcher.

1. Create `src/oauth2/token.rs` with:

   struct TokenInner {
       watcher: aliri_tokens::TokenWatcher<String>,
   }

   #[derive(Clone)]
   pub struct Token {
       inner: Arc<ArcSwap<TokenInner>>,
       source_factory: Arc<dyn Fn() -> Result<OAuthTokenSource, TokenError> + Send + Sync>,
       watcher_config: Arc<WatcherConfig>,
   }

   struct WatcherConfig {
       refresh_offset: Duration,
       jitter_max: Duration,
       min_refresh_period: Duration,
   }

2. Constructor `Token::new(config: OAuthClientConfig) -> Result<Self, TokenError>`:
   - Call config.validate().
   - Create OAuthTokenSource::new(&config).
   - Create aliri_tokens watcher from the source with the refresh parameters.
   - Store a source_factory closure that captures the config and can
     recreate the source for invalidation.
   - Wrap the watcher in TokenInner, then Arc, then ArcSwap.

3. `pub async fn get(&self) -> Result<SecretString, TokenError>`:
   - `let guard = self.inner.load();` — lock-free read (compare jwks.rs:299).
   - Call `guard.watcher.token().await` to get the cached/refreshed token.
   - Wrap the raw String in SecretString immediately.
   - Map aliri_tokens errors to TokenError::Unavailable.

4. `pub async fn invalidate(&self)`:
   - Call source_factory() to create a new OAuthTokenSource.
   - Create a new watcher with same WatcherConfig.
   - `self.inner.store(Arc::new(TokenInner { watcher }))` — atomic swap
     (compare jwks.rs:190).
   - Old watcher is dropped when last reference is released.
   - If source_factory fails: log warning via `tracing::warn!`, do NOT swap,
     do NOT panic.

5. Re-export Token from src/oauth2/mod.rs and from the crate root.

Use arc-swap from workspace (already in Cargo.toml).
Token must be Clone + Send + Sync.
```

### Guard

```
Verify Phase 3 (Token handle) of the OAuth2 client credentials implementation.

Check every item:

1. Token is Clone + Send + Sync:
   - Write a compile-time assertion:
     fn _assert_token_traits() {
         fn assert_send_sync_clone<T: Send + Sync + Clone>() {}
         assert_send_sync_clone::<Token>();
     }

2. Token::new():
   - Calls config.validate().
   - Creates OAuthTokenSource.
   - Creates aliri_tokens watcher with refresh_offset, jitter_max,
     min_refresh_period from config.
   - Returns Result, not panics.

3. Token::get():
   - Returns SecretString (not raw String).
   - Uses self.inner.load() (lock-free read, same as jwks.rs:299).
   - Maps watcher errors to TokenError::Unavailable.

4. Token::invalidate():
   - Creates a NEW watcher (does not reuse the old one).
   - Uses self.inner.store() for atomic swap (same as jwks.rs:190).
   - If source_factory fails: logs warning, does NOT swap, does NOT panic.
   - Old watcher is eventually dropped (no leak).

5. SECRET SAFETY:
   - Token::get() wraps the raw token string in SecretString immediately.
   - The raw String is never stored in Token or TokenInner beyond the watcher.
   - No Debug impl on Token or TokenInner prints token values.

6. ArcSwap PATTERN matches JWKS:
   - Uses ArcSwap<TokenInner> (not RwLock).
   - get() uses .load() (lock-free read).
   - invalidate() uses .store() (atomic swap).

7. COMPILATION: `cargo check -p modkit-auth` succeeds.

8. TESTS (same httpmock pattern as JWKS tests):
   - Token::new() with valid config succeeds.
   - Token::new() with invalid config (both endpoints) returns Err.
   - Token::get() returns SecretString whose expose() matches mock token.
   - Token::invalidate() causes next get() to re-fetch (verify mock hit
     count increments after invalidate).
   - Two concurrent get() calls do not deadlock.
```

---

## Phase 4 — Tower auth layer

**Goal:** Implement `BearerAuthLayer` / `BearerAuthService` that injects `Authorization: Bearer <token>` on every request.

### Prompt

```
Implement the outbound bearer auth tower layer in
libs/modkit-auth/src/oauth2/layer.rs.

REFERENCE: Follow the exact tower Layer + Service pattern used by modkit-http:
- libs/modkit-http/src/layers/otel.rs — OtelLayer + OtelService (simplest async layer)
- libs/modkit-http/src/layers/user_agent.rs — UserAgentLayer + UserAgentService
  (synchronous header injection, closest conceptual match, but our layer needs
  async because token.get() is async)

NOTE: The existing bearer token EXTRACTION (inbound) is in
libs/modkit-auth/src/axum_ext.rs:221-227 (extract_bearer_token). Our layer does
the INVERSE — it INJECTS a bearer token into outbound requests.

1. `BearerAuthLayer`:
   - Fields:
     - `token: Token`
     - `header_name: http::header::HeaderName` (default: AUTHORIZATION)
   - Constructors:
     - `fn new(token: Token) -> Self` (uses AUTHORIZATION)
     - `fn with_header_name(token: Token, header_name: HeaderName) -> Self`
   - Implement `tower::Layer<S>` producing `BearerAuthService<S>`.
   - Derive Clone.

2. `BearerAuthService<S>`:
   - Fields: inner: S, token: Token, header_name: HeaderName
   - Derive Clone.
   - Implement `tower::Service<http::Request<B>>` where
     S: Service<Request<B>, Response = Response<ResBody>, Error = HttpError>:
     - poll_ready: delegate to inner.
     - call: return a boxed future (Pin<Box<dyn Future + Send>>) that:
       a. Calls self.token.get().await.
       b. On success: set header `{header_name}: Bearer {token.expose()}`
          on the request. The expose() value must NOT be bound to a long-lived
          variable.
       c. On error: map TokenError to HttpError. Since HttpError is
          #[non_exhaustive] and lives in modkit-http, map via
          `HttpError::Transport(Box::new(token_error))` — this preserves the
          error chain without modifying modkit-http's error enum.
       d. Forward request to inner service.

   Compare with OtelService (otel.rs) which also uses a boxed async future
   in call() to handle async tracing operations.

3. Re-export BearerAuthLayer from src/oauth2/mod.rs and the crate root.

IMPORTANT: token.expose() is used only transiently for header value
construction. Do not store it in any struct field or log it.
```

### Guard

```
Verify Phase 4 (tower auth layer) of the OAuth2 client credentials implementation.

Check every item:

1. LAYER PATTERN matches modkit-http conventions:
   - BearerAuthLayer implements tower::Layer<S>.
   - BearerAuthService<S> implements tower::Service<Request<B>>.
   - Both derive Clone.

2. HEADER INJECTION:
   - Default header is AUTHORIZATION.
   - Value format is "Bearer {token}" (with space, no extra whitespace).
   - with_header_name allows custom header (e.g. "X-Api-Key").
   - Header set BEFORE calling inner service.

3. ASYNC HANDLING:
   - call() returns a boxed future that is Send + 'static.
   - token.get().await called inside the future (not synchronously in call()).
   - poll_ready only checks inner readiness.

4. ERROR HANDLING:
   - TokenError mapped to HttpError::Transport(Box::new(token_error)).
   - If token.get() fails, request is NOT forwarded to inner service.
   - Error does not contain the token value.

5. SECRET SAFETY:
   - token.expose() called only in header value construction.
   - grep layer.rs for "tracing" / "debug!" / "info!" / "warn!" — if present,
     must NOT include the token value.

6. INVERSE OF EXTRACTION: compare with axum_ext.rs:221 extract_bearer_token
   which strips "Bearer " prefix for inbound. Our layer adds "Bearer " prefix
   for outbound. Format is consistent.

7. COMPILATION: `cargo check -p modkit-auth` succeeds.

8. TESTS:
   - Mock inner service captures request headers.
   - Wrap with BearerAuthLayer, send request → assert Authorization header
     is "Bearer {expected}".
   - Custom header name → assert custom header set.
   - Token error → assert request not forwarded, error returned.
```

---

## Phase 5 — HttpClientBuilder integration

**Goal:** Add `.with_bearer_auth(token)` to `HttpClientBuilder` so the auth layer is part of the composed tower stack.

### Prompt

```
Add bearer auth support to modkit-http's HttpClientBuilder.

REFERENCE: Read libs/modkit-http/src/builder.rs to understand the current tower
stack composition. The stack order is:

  Buffer → OTel → LoadShed/Concurrency → Retry → Timeout → UserAgent → Decompress → Redirect → hyper

The auth layer must be inserted INSIDE retry but OUTSIDE timeout:

  Buffer → OTel → LoadShed/Concurrency → Retry → **BearerAuth** → Timeout → UserAgent → ...

This ensures: (a) retries re-acquire the token on each attempt, (b) token
acquisition time counts toward the per-request timeout.

1. Add modkit-auth as an OPTIONAL dependency of modkit-http behind a feature flag.

   In libs/modkit-http/Cargo.toml:
   [features]
   oauth = ["dep:modkit-auth"]

   [dependencies]
   modkit-auth = { workspace = true, optional = true }

2. In builder.rs, add field (feature-gated):
   #[cfg(feature = "oauth")]
   bearer_auth: Option<modkit_auth::oauth2::BearerAuthLayer>,

   Initialize to None in HttpClientBuilder::new().

3. Add builder methods (feature-gated):
   #[cfg(feature = "oauth")]
   pub fn with_bearer_auth(mut self, token: modkit_auth::Token) -> Self
   #[cfg(feature = "oauth")]
   pub fn with_bearer_auth_header(mut self, token: modkit_auth::Token, header_name: HeaderName) -> Self

4. In build(), insert BearerAuthLayer conditionally.
   Use tower::util::option_layer or manual Either wrapping.
   The layer goes between RetryLayer and TimeoutLayer in the ServiceBuilder chain.

   When oauth feature is disabled: no changes to build() at all (use #[cfg]).
   When oauth feature is enabled but bearer_auth is None: layer is a passthrough.

IMPORTANT: When the `oauth` feature is not enabled, there must be ZERO impact
on compilation, binary size, or API surface. All oauth code behind
#[cfg(feature = "oauth")].
```

### Guard

```
Verify Phase 5 (HttpClientBuilder integration) in libs/modkit-http.

Check every item:

1. FEATURE FLAG:
   - modkit-http/Cargo.toml has `oauth = ["dep:modkit-auth"]` feature.
   - modkit-auth listed as optional dependency.
   - `cargo check -p cf-modkit-http` (no features) compiles cleanly.
   - `cargo check -p cf-modkit-http --features oauth` compiles cleanly.

2. BUILDER API (feature-gated):
   - with_bearer_auth() and with_bearer_auth_header() exist only with
     #[cfg(feature = "oauth")].
   - Both return Self (fluent API, #[must_use]).

3. LAYER POSITION in tower stack:
   - Read build() — confirm BearerAuthLayer is composed AFTER RetryLayer
     wraps the inner service (inside retry) and BEFORE TimeoutLayer
     (outside timeout).
   - Stack order:
     Buffer → OTel → LoadShed → Retry → BearerAuth → Timeout → UserAgent → ...

4. CONDITIONAL COMPILATION:
   - Without oauth feature: no reference to modkit_auth in compiled code.
   - With oauth feature but no with_bearer_auth() call: no auth layer in stack.

5. COMPILATION:
   - `cargo check -p cf-modkit-http` — OK.
   - `cargo check -p cf-modkit-http --features oauth` — OK.
   - `cargo check --workspace` — OK.

6. TESTS (behind #[cfg(feature = "oauth")]):
   - HttpClient with with_bearer_auth(mock_token) → mock server receives
     Authorization: Bearer {token}.
   - HttpClient WITHOUT with_bearer_auth() → no Authorization header.
```

---

## Phase 6 — OIDC discovery

**Goal:** Support `issuer_url` as an alternative to `token_endpoint` — resolve the token endpoint via `/.well-known/openid-configuration`.

### Prompt

```
Add OIDC discovery support to OAuthTokenSource in
libs/modkit-auth/src/oauth2/source.rs.

REFERENCE: The JWKS provider (libs/modkit-auth/src/providers/jwks.rs) fetches
a remote JSON document (JWKS) via HttpClient and parses it. OIDC discovery is
the same pattern: fetch a JSON document and extract one field.

Compare:
- JWKS: GET {jwks_uri} → parse JwksResponse { keys: Vec<Jwk> }
- OIDC: GET {issuer_url}/.well-known/openid-configuration → parse { token_endpoint }

1. Create `src/oauth2/discovery.rs`:
   - `OidcDiscoveryDoc` (serde Deserialize):
     - `token_endpoint: String` (required)
     - Other fields ignored (no deny_unknown_fields).
   - `pub async fn discover_token_endpoint(
         client: &HttpClient,
         issuer_url: &Url,
     ) -> Result<Url, TokenError>`:
     - Build discovery URL: strip trailing slash from issuer_url, then append
       `/.well-known/openid-configuration`.
     - GET via client.get(url).send().await.
     - Call error_for_status(), then .json::<OidcDiscoveryDoc>().
     - Parse token_endpoint string as Url.
     - On error: map via format_http_error (Phase 0) for HTTP errors,
       TokenError::InvalidResponse for parse/missing field errors.

   Follow the same HTTP call pattern as jwks.rs fetch_jwks() (lines 129-139).

2. Update `OAuthTokenSource`:
   - Add a constructor variant or make new() async:
     `pub(crate) async fn new(config: &OAuthClientConfig) -> Result<Self, TokenError>`
   - If config.token_endpoint is Some: use directly (no change).
   - If config.issuer_url is Some: call discover_token_endpoint() to resolve.
   - Store resolved URL. Discovery is ONE-TIME at construction.

3. Update `Token::new()`:
   - Becomes async: `pub async fn new(config: OAuthClientConfig) -> Result<Self, TokenError>`
   - Discovery runs once in new(). The resolved token_endpoint is captured in
     the source_factory closure so invalidation does NOT repeat discovery.

4. Wire discovery.rs into src/oauth2/mod.rs.
```

### Guard

```
Verify Phase 6 (OIDC discovery) of the OAuth2 client credentials implementation.

Check every item:

1. DISCOVERY URL CONSTRUCTION:
   - "https://auth.example.com" → ".../openid-configuration" (no double slash).
   - "https://auth.example.com/" → ".../openid-configuration" (trailing slash stripped).
   - Unit test for both cases.

2. DISCOVERY HTTP PATTERN matches JWKS:
   - Uses client.get(url).send().await (compare jwks.rs:131-136).
   - Calls error_for_status() before json() (compare jwks.rs:137-139).
   - Maps errors via format_http_error (Phase 0).

3. ONE-TIME DISCOVERY:
   - Discovery in Token::new() or OAuthTokenSource::new().
   - Subsequent token requests use cached endpoint.
   - Token::invalidate() does NOT repeat discovery.
   - Test: mock discovery + mock token endpoint. Call get() multiple times,
     invalidate() once. Discovery mock hit count = 1.

4. CONFIG VALIDATION:
   - token_endpoint only: no discovery.
   - issuer_url only: discovery runs.
   - Both set: error.
   - Neither set: error.

5. Token::new() IS ASYNC. All callers updated.

6. SECRET SAFETY:
   - Discovery URL does not contain secrets.
   - Error messages from discovery do not contain secrets.

7. COMPILATION: `cargo check -p modkit-auth` succeeds.

8. TESTS (httpmock + allow_insecure_http, same pattern as JWKS tests):
   - Integration: mock OIDC discovery + mock token endpoint → Token::get()
     returns expected token.
   - Unit: discover_token_endpoint with valid response.
   - Unit: discover_token_endpoint with missing token_endpoint field → error.
   - Unit: trailing slash handling.
```
