# Outbound OAuth2 Client Credentials for ModKit modules using aliri_tokens

## Context

ModKit modules call internal vendor REST services secured with OAuth2 Client Credentials.
Each module gets:

- `client_id`, `client_secret`
- `token_endpoint` or `issuer_url`
- `scopes` list

The platform HTTP client is `modkit-http::HttpClient` (hyper + tower), which already provides retry with exponential backoff and jitter, OTel tracing, concurrency limiting, TLS-only transport, and a preconfigured `HttpClientConfig::token_endpoint()` profile.

## Requirements

- token acquisition + in-memory cache
- refresh before expiry
- jitter to avoid stampede
- safe concurrency under load
- automatic `Authorization: Bearer <token>` injection for outbound HTTP
- no `reqwest` dependency (direct or transitive) in `modkit-auth`

## Decision

Use `aliri_tokens` as the token lifecycle engine, without its `oauth2` feature (so it does not pull `reqwest`). Implement OAuth2 Client Credentials exchange as a custom token source that uses `modkit-http::HttpClient` with `HttpClientConfig::token_endpoint()`.

Outbound HTTP composition is hyper + tower. Authentication is implemented as a tower layer that composes with the existing `HttpClient` layer stack.

## Consequences

**Good:**

- refresh scheduling, jitter, concurrency control, and backoff are delegated to `aliri_tokens`
- token endpoint HTTP calls reuse `modkit-http::HttpClient` — retries, timeouts, rate limiting, OTel tracing, and TLS are handled by the existing tower stack; no duplicate implementation needed
- `HttpClientConfig::token_endpoint()` already configures conservative retry (transport errors, timeout, 429 only) appropriate for token acquisition
- outbound auth layer is a standard tower layer, composable with the existing `HttpClient` middleware
- no `reqwest` in `modkit-auth`

**Bad:**

- small amount of glue code is needed: `AsyncTokenSource` implementation wrapping `HttpClient`, optional OIDC discovery, tower auth layer
- `invalidate()` is not a native operation in `aliri_tokens` and must be implemented via watcher rotation

## Implementation Status

All components are implemented in `libs/modkit-auth/src/oauth2/` and `libs/modkit-http/src/builder.rs`.

### Modules and public API

| Module | Path | Public type | Description |
|--------|------|-------------|-------------|
| `config` | `oauth2/config.rs` | `OAuthClientConfig` | Configuration struct with `token_endpoint` / `issuer_url` (mutually exclusive), credentials, scopes, refresh policy, and optional `HttpClientConfig` override. `Debug` redacts `client_secret`. |
| `types` | `oauth2/types.rs` | `ClientAuthMethod`, `SecretString` | Auth method enum (`Basic` / `Form`). `SecretString` re-exported from `modkit-utils` (backed by `Zeroizing<String>`). |
| `error` | `oauth2/error.rs` | `TokenError` | `#[non_exhaustive]` error enum: `Http`, `InvalidResponse`, `UnsupportedTokenType`, `ConfigError`, `Unavailable`. All variants are secret-safe. |
| `token` | `oauth2/token.rs` | `Token` | Handle for obtaining bearer tokens. `Clone + Send + Sync`. Background refresh via `aliri_tokens::TokenWatcher`. Lock-free reads via `ArcSwap`. `get()` returns `SecretString`, `invalidate()` rotates the watcher without repeating OIDC discovery. |
| `layer` | `oauth2/layer.rs` | `BearerAuthLayer` | Tower `Layer` + `Service` that injects `Authorization: Bearer <token>` (or custom header) into outbound requests. |
| `builder_ext` | `oauth2/builder_ext.rs` | `HttpClientBuilderExt` | Extension trait on `modkit_http::HttpClientBuilder` providing `.with_bearer_auth(token)` and `.with_bearer_auth_header(token, header_name)`. |
| `discovery` | `oauth2/discovery.rs` | *(crate-internal)* | One-time OIDC discovery: fetches `{issuer_url}/.well-known/openid-configuration` and extracts `token_endpoint`. |
| `source` | `oauth2/source.rs` | *(crate-internal)* | `AsyncTokenSource` implementation that exchanges client credentials for an access token via `modkit-http::HttpClient`. |

All public types are re-exported from `modkit_auth::oauth2` and from the crate root (`modkit_auth::{...}`).

### `modkit-http` integration

`HttpClientBuilder::with_auth_layer(wrap)` accepts a generic `FnOnce(BoxCloneService) -> BoxCloneService` transform inserted between retry and timeout in the tower stack. This avoids a circular dependency (`modkit-auth` depends on `modkit-http`, not vice versa). The `HttpClientBuilderExt` extension trait in `modkit-auth` wraps `BearerAuthLayer` into this hook.

### Architecture notes

- **No circular dependency:** `modkit-http` has zero awareness of `modkit-auth`. The auth layer is injected via a generic hook (`with_auth_layer`) and an extension trait pattern.
- **OIDC discovery is one-time:** Runs in `Token::new()`. The resolved `token_endpoint` is captured in the `source_factory` closure, so `invalidate()` rebuilds the watcher without re-running discovery.
- **Secret safety:** `SecretString` uses `Zeroizing<String>` for secure memory cleanup. `Debug` and `Display` impls are redacted. Access tokens are only exposed transiently in `format!("Bearer {}", secret.expose())` within the auth layer.
- **Stack order:** `Buffer -> OTel -> LoadShed -> Retry -> **Auth** -> Timeout -> UserAgent -> Decompress -> Redirect -> hyper`
