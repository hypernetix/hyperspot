use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use aliri_clock::DurationSecs;
use aliri_tokens::backoff::ErrorBackoffConfig;
use aliri_tokens::jitter::RandomEarlyJitter;
use aliri_tokens::{TokenStatus, TokenWatcher};
use arc_swap::ArcSwap;

use super::config::OAuthClientConfig;
use super::error::TokenError;
use super::source::OAuthTokenSource;
use modkit_utils::SecretString;

/// Internal state holding the live watcher.
///
/// Wrapped in `Arc<ArcSwap<_>>` so that [`Token::invalidate`] can atomically
/// swap in a replacement without blocking concurrent [`Token::get`] calls.
struct TokenInner {
    watcher: TokenWatcher,
}

/// Parameters needed to (re-)spawn a [`TokenWatcher`].
struct WatcherConfig {
    jitter_max: Duration,
    min_refresh_period: Duration,
}

/// Handle for obtaining `OAuth2` bearer tokens.
///
/// Internally drives an `aliri_tokens::TokenWatcher` for background refresh and
/// exposes lock-free reads via `ArcSwap` (same pattern as the JWKS key
/// provider).
///
/// `Token` is [`Clone`] + [`Send`] + [`Sync`] — share freely across tasks.
#[derive(Clone)]
pub struct Token {
    inner: Arc<ArcSwap<TokenInner>>,
    source_factory: Arc<dyn Fn() -> Result<OAuthTokenSource, TokenError> + Send + Sync>,
    watcher_config: Arc<WatcherConfig>,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Token").finish_non_exhaustive()
    }
}

impl Token {
    /// Create a new token handle and start background refresh.
    ///
    /// This performs an initial token fetch — if the token endpoint is
    /// unreachable or returns an error, `new` will fail immediately.
    ///
    /// # Errors
    ///
    /// Returns [`TokenError::ConfigError`] if the config is invalid.
    /// Returns [`TokenError::Http`] if the initial token fetch fails.
    pub async fn new(mut config: OAuthClientConfig) -> Result<Self, TokenError> {
        config.validate()?;

        // Resolve issuer_url → token_endpoint via OIDC discovery (one-time).
        if let Some(issuer_url) = config.issuer_url.take() {
            let http_config = config
                .http_config
                .clone()
                .unwrap_or_else(modkit_http::HttpClientConfig::token_endpoint);
            let client = modkit_http::HttpClientBuilder::with_config(http_config)
                .build()
                .map_err(|e| {
                    TokenError::Http(crate::http_error::format_http_error(&e, "OIDC discovery"))
                })?;
            let resolved = super::discovery::discover_token_endpoint(&client, &issuer_url).await?;
            config.token_endpoint = Some(resolved);
        }

        let watcher_config = Arc::new(WatcherConfig {
            jitter_max: config.jitter_max,
            min_refresh_period: config.min_refresh_period,
        });

        let source = OAuthTokenSource::new(&config)?;
        let watcher = spawn_watcher(source, &watcher_config).await?;

        let source_factory: Arc<dyn Fn() -> Result<OAuthTokenSource, TokenError> + Send + Sync> =
            Arc::new(move || OAuthTokenSource::new(&config));

        Ok(Self {
            inner: Arc::new(ArcSwap::from_pointee(TokenInner { watcher })),
            source_factory,
            watcher_config,
        })
    }

    /// Get the current bearer token.
    ///
    /// This is a lock-free read from the `ArcSwap`-cached watcher — it never
    /// blocks on a network call.  The underlying watcher refreshes the token in
    /// the background before it expires.
    ///
    /// The returned [`SecretString`] wraps the raw access-token value so it is
    /// not accidentally logged.
    ///
    /// # Errors
    ///
    /// Returns [`TokenError::Unavailable`] if the cached token has expired
    /// (the background watcher has not yet refreshed it).
    pub fn get(&self) -> Result<SecretString, TokenError> {
        let guard = self.inner.load();
        let borrowed = guard.watcher.token();
        if matches!(borrowed.token_status(), TokenStatus::Expired) {
            return Err(TokenError::Unavailable(
                "token expired, refresh pending".into(),
            ));
        }
        let raw = borrowed.access_token().as_str();
        Ok(SecretString::new(raw))
    }

    /// Force-replace the internal watcher with a freshly-spawned one.
    ///
    /// Use this after receiving a 401 from a downstream service to immediately
    /// discard a potentially revoked token.
    ///
    /// If recreating the source or the initial token fetch fails, a warning is
    /// logged and the existing watcher is left in place.
    pub async fn invalidate(&self) {
        let source = match (self.source_factory)() {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("OAuth2 token invalidation: failed to create source: {e}");
                return;
            }
        };

        let watcher = match spawn_watcher(source, &self.watcher_config).await {
            Ok(w) => w,
            Err(e) => {
                tracing::warn!("OAuth2 token invalidation: initial fetch failed: {e}");
                return;
            }
        };

        self.inner.store(Arc::new(TokenInner { watcher }));
    }
}

/// Spawn a [`TokenWatcher`] from the given source and config.
async fn spawn_watcher(
    source: OAuthTokenSource,
    config: &WatcherConfig,
) -> Result<TokenWatcher, TokenError> {
    let jitter = RandomEarlyJitter::new(DurationSecs(config.jitter_max.as_secs()));
    let backoff =
        ErrorBackoffConfig::new(config.min_refresh_period, config.min_refresh_period * 30, 2);

    TokenWatcher::spawn_from_token_source(source, jitter, backoff).await
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use url::Url;

    /// Build a test config pointing at the given mock server.
    fn test_config(server: &MockServer) -> OAuthClientConfig {
        OAuthClientConfig {
            token_endpoint: Some(
                Url::parse(&format!("http://localhost:{}/token", server.port())).unwrap(),
            ),
            client_id: "test-client".into(),
            client_secret: SecretString::new("test-secret"),
            http_config: Some(modkit_http::HttpClientConfig::for_testing()),
            // Use short durations for tests.
            jitter_max: Duration::from_millis(0),
            min_refresh_period: Duration::from_millis(100),
            ..Default::default()
        }
    }

    fn token_json(token: &str, expires_in: u64) -> String {
        format!(r#"{{"access_token":"{token}","expires_in":{expires_in},"token_type":"Bearer"}}"#)
    }

    // -- trait assertions -----------------------------------------------------

    #[test]
    fn token_is_send_sync_clone() {
        fn assert_traits<T: Send + Sync + Clone>() {}
        assert_traits::<Token>();
    }

    // -- new ------------------------------------------------------------------

    #[tokio::test]
    async fn new_with_valid_config() {
        let server = MockServer::start();

        let _mock = server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(token_json("tok-new", 3600));
        });

        let token = Token::new(test_config(&server)).await;
        assert!(
            token.is_ok(),
            "Token::new() should succeed: {:?}",
            token.err()
        );
    }

    #[tokio::test]
    async fn new_validates_config() {
        let cfg = OAuthClientConfig {
            token_endpoint: Some(Url::parse("https://a.example.com/token").unwrap()),
            issuer_url: Some(Url::parse("https://b.example.com").unwrap()),
            client_id: "test-client".into(),
            client_secret: SecretString::new("test-secret"),
            ..Default::default()
        };
        let err = Token::new(cfg).await.unwrap_err();
        assert!(
            matches!(err, TokenError::ConfigError(ref msg) if msg.contains("mutually exclusive")),
            "expected ConfigError, got: {err}"
        );
    }

    // -- get ------------------------------------------------------------------

    #[tokio::test]
    async fn get_returns_secret_string() {
        let server = MockServer::start();

        let _mock = server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(token_json("tok-get-test", 3600));
        });

        let token = Token::new(test_config(&server)).await.unwrap();
        let secret = token.get().unwrap();

        assert_eq!(secret.expose(), "tok-get-test");
    }

    // -- invalidate -----------------------------------------------------------

    #[tokio::test]
    async fn invalidate_creates_new_watcher() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(token_json("tok-inv", 3600));
        });

        let token = Token::new(test_config(&server)).await.unwrap();
        assert_eq!(mock.calls(), 1, "initial fetch");

        token.invalidate().await;

        // invalidate spawns a new watcher which fetches a fresh token
        assert_eq!(mock.calls(), 2, "after invalidate");
    }

    // -- concurrency ----------------------------------------------------------

    #[tokio::test]
    async fn concurrent_get_no_deadlock() {
        let server = MockServer::start();

        let _mock = server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(token_json("tok-conc", 3600));
        });

        let token = Token::new(test_config(&server)).await.unwrap();

        let t1 = {
            let token = token.clone();
            tokio::spawn(async move { token.get() })
        };
        let t2 = {
            let token = token.clone();
            tokio::spawn(async move { token.get() })
        };

        let (r1, r2) = tokio::join!(t1, t2);
        assert!(r1.unwrap().is_ok());
        assert!(r2.unwrap().is_ok());
    }

    // -- OIDC discovery -------------------------------------------------------

    #[tokio::test]
    async fn new_with_issuer_url_discovery() {
        let server = MockServer::start();

        // Mock the OIDC discovery endpoint.
        let token_ep = format!("http://localhost:{}/oauth/token", server.port());
        let _discovery_mock = server.mock(|when, then| {
            when.method(GET).path("/.well-known/openid-configuration");
            then.status(200)
                .header("content-type", "application/json")
                .body(format!(r#"{{"token_endpoint":"{token_ep}"}}"#));
        });

        // Mock the resolved token endpoint.
        let _token_mock = server.mock(|when, then| {
            when.method(POST).path("/oauth/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(token_json("tok-discovered", 3600));
        });

        let cfg = OAuthClientConfig {
            issuer_url: Some(Url::parse(&format!("http://localhost:{}", server.port())).unwrap()),
            client_id: "test-client".into(),
            client_secret: SecretString::new("test-secret"),
            http_config: Some(modkit_http::HttpClientConfig::for_testing()),
            jitter_max: Duration::from_millis(0),
            min_refresh_period: Duration::from_millis(100),
            ..Default::default()
        };

        let token = Token::new(cfg).await.unwrap();
        let secret = token.get().unwrap();
        assert_eq!(secret.expose(), "tok-discovered");
    }

    #[tokio::test]
    async fn discovery_not_repeated_on_invalidate() {
        let server = MockServer::start();

        // Mock the OIDC discovery endpoint.
        let token_ep = format!("http://localhost:{}/oauth/token", server.port());
        let discovery_mock = server.mock(|when, then| {
            when.method(GET).path("/.well-known/openid-configuration");
            then.status(200)
                .header("content-type", "application/json")
                .body(format!(r#"{{"token_endpoint":"{token_ep}"}}"#));
        });

        // Mock the resolved token endpoint.
        let token_mock = server.mock(|when, then| {
            when.method(POST).path("/oauth/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(token_json("tok-disc-inv", 3600));
        });

        let cfg = OAuthClientConfig {
            issuer_url: Some(Url::parse(&format!("http://localhost:{}", server.port())).unwrap()),
            client_id: "test-client".into(),
            client_secret: SecretString::new("test-secret"),
            http_config: Some(modkit_http::HttpClientConfig::for_testing()),
            jitter_max: Duration::from_millis(0),
            min_refresh_period: Duration::from_millis(100),
            ..Default::default()
        };

        let token = Token::new(cfg).await.unwrap();
        assert_eq!(discovery_mock.calls(), 1, "discovery: initial");
        assert_eq!(token_mock.calls(), 1, "token: initial");

        // Invalidate should re-fetch the token but NOT re-run discovery.
        token.invalidate().await;

        assert_eq!(
            discovery_mock.calls(),
            1,
            "discovery must NOT be repeated on invalidate"
        );
        assert_eq!(token_mock.calls(), 2, "token: after invalidate");
    }

    // -- debug safety ---------------------------------------------------------

    #[tokio::test]
    async fn debug_does_not_reveal_tokens() {
        let server = MockServer::start();

        let _mock = server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(token_json("super-secret-tok", 3600));
        });

        let token = Token::new(test_config(&server)).await.unwrap();
        let dbg = format!("{token:?}");
        assert!(
            !dbg.contains("super-secret-tok"),
            "Debug must not reveal token value: {dbg}"
        );
    }
}
