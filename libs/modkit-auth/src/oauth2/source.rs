use std::time::Duration;

use aliri_clock::DurationSecs;
use aliri_tokens::sources::AsyncTokenSource;
use aliri_tokens::{AccessToken, IdToken, TokenLifetimeConfig, TokenWithLifetime};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use http::header::AUTHORIZATION;
use url::Url;
use zeroize::Zeroizing;

use super::config::OAuthClientConfig;
use super::error::TokenError;
use super::types::ClientAuthMethod;
use modkit_utils::SecretString;

/// Token source that exchanges client credentials for an access token using
/// `modkit-http::HttpClient`.
///
/// It implements [`aliri_tokens::AsyncTokenSource`] so that
/// `aliri_tokens` can drive refresh scheduling, jitter, and backoff.
pub struct OAuthTokenSource {
    client: modkit_http::HttpClient,
    token_endpoint: Url,
    client_id: String,
    client_secret: SecretString,
    /// Pre-joined scopes (space-separated), or `None` when the scope list is
    /// empty.
    scopes: Option<String>,
    auth_method: ClientAuthMethod,
    extra_headers: Vec<(String, String)>,
    default_ttl: Duration,
    refresh_offset: Duration,
    min_refresh_period: Duration,
}

impl OAuthTokenSource {
    /// Build a new token source from the given configuration.
    ///
    /// # Errors
    ///
    /// Returns [`TokenError::ConfigError`] if `token_endpoint` is `None`.
    ///
    /// Returns [`TokenError::Http`] if the underlying `HttpClient` fails to
    /// build.
    pub fn new(config: &OAuthClientConfig) -> Result<Self, TokenError> {
        let token_endpoint = config
            .token_endpoint
            .clone()
            .ok_or_else(|| TokenError::ConfigError("token_endpoint is required".into()))?;

        let http_config = config
            .http_config
            .clone()
            .unwrap_or_else(modkit_http::HttpClientConfig::token_endpoint);

        let client = modkit_http::HttpClientBuilder::with_config(http_config)
            .build()
            .map_err(|e| {
                TokenError::Http(crate::http_error::format_http_error(&e, "OAuth2 token"))
            })?;

        let scopes = if config.scopes.is_empty() {
            None
        } else {
            Some(config.scopes.join(" "))
        };

        Ok(Self {
            client,
            token_endpoint,
            client_id: config.client_id.clone(),
            client_secret: config.client_secret.clone(),
            scopes,
            auth_method: config.auth_method,
            extra_headers: config.extra_headers.clone(),
            default_ttl: config.default_ttl,
            refresh_offset: config.refresh_offset,
            min_refresh_period: config.min_refresh_period,
        })
    }
}

#[async_trait]
impl AsyncTokenSource for OAuthTokenSource {
    type Error = TokenError;

    async fn request_token(&mut self) -> Result<TokenWithLifetime, Self::Error> {
        // -- build form fields ---------------------------------------------------
        let mut fields: Vec<(&str, &str)> = vec![("grant_type", "client_credentials")];

        if let Some(ref scope) = self.scopes {
            fields.push(("scope", scope));
        }

        // For Form auth, credentials go into the form body.
        // Wrap the temporary copy in `Zeroizing` so it is scrubbed on drop.
        let secret_expose;
        if self.auth_method == ClientAuthMethod::Form {
            secret_expose = Zeroizing::new(self.client_secret.expose().to_owned());
            fields.push(("client_id", &self.client_id));
            fields.push(("client_secret", &secret_expose));
        }

        // -- build request -------------------------------------------------------
        let mut builder = self.client.post(self.token_endpoint.as_str());

        // For Basic auth, credentials go into the Authorization header.
        // Wrap intermediates in `Zeroizing` so the plaintext is scrubbed on drop.
        if self.auth_method == ClientAuthMethod::Basic {
            let credentials = Zeroizing::new(format!(
                "{}:{}",
                self.client_id,
                self.client_secret.expose()
            ));
            let encoded = Zeroizing::new(general_purpose::STANDARD.encode(credentials.as_bytes()));
            let header_value = Zeroizing::new(format!("Basic {}", &*encoded));
            builder = builder.header(AUTHORIZATION.as_str(), &header_value);
        }

        // Apply vendor-specific extra headers.
        for (name, value) in &self.extra_headers {
            builder = builder.header(name, value);
        }

        let response = builder
            .form(fields.as_slice())
            .map_err(|e| {
                TokenError::Http(crate::http_error::format_http_error(&e, "OAuth2 token"))
            })?
            .send()
            .await
            .map_err(|e| {
                TokenError::Http(crate::http_error::format_http_error(&e, "OAuth2 token"))
            })?;

        // -- check status, then parse response ------------------------------------
        let token_resp: super::types::TokenResponse = response
            .error_for_status()
            .map_err(|e| {
                TokenError::Http(crate::http_error::format_http_error(&e, "OAuth2 token"))
            })?
            .json()
            .await
            .map_err(|e| {
                TokenError::Http(crate::http_error::format_http_error(&e, "OAuth2 token"))
            })?;

        // -- validate token_type -------------------------------------------------
        if let Some(ref tt) = token_resp.token_type
            && !tt.eq_ignore_ascii_case("bearer")
        {
            return Err(TokenError::UnsupportedTokenType(tt.clone()));
        }

        // -- compute lifetime ----------------------------------------------------
        let lifetime_secs = token_resp.expires_in.unwrap_or(self.default_ttl.as_secs());

        // Compute per-token refresh parameters so that the stale time
        // never exceeds the expiry time, even for short-lived tokens.
        let (freshness, min_stale) = refresh_params(
            lifetime_secs,
            &self.refresh_offset,
            &self.min_refresh_period,
        );
        let lifetime_config = TokenLifetimeConfig::new(freshness, min_stale);

        let access_token = AccessToken::new(token_resp.access_token);
        let token = lifetime_config.create_token(
            &access_token,
            None::<&IdToken>,
            DurationSecs(lifetime_secs),
        );

        Ok(token)
    }
}

/// Compute refresh parameters for [`TokenLifetimeConfig`].
///
/// Returns `(freshness_period, min_staleness_period)` such that
/// `max(lifetime × freshness_period, min_staleness_period) <= lifetime`,
/// guaranteeing the stale time never exceeds the expiry time.
///
/// `min_refresh_period` is used as a **lower bound on the staleness
/// window** (the minimum time the token spends in the "stale" state
/// before expiry), not as a refresh deadline. It is capped to
/// `desired_delay` so it can never push the stale time past expiry.
/// In `aliri_tokens` terms it maps to `min_staleness_period`.
///
/// - Normal case (`offset < lifetime`): stale `offset` seconds before
///   expiry.
/// - Short-lived token (`offset >= lifetime`): stale at 50% of lifetime.
/// - Zero lifetime: immediately stale.
#[allow(clippy::integer_division, clippy::cast_precision_loss)]
fn refresh_params(
    lifetime_secs: u64,
    refresh_offset: &Duration,
    min_refresh_period: &Duration,
) -> (f64, DurationSecs) {
    if lifetime_secs == 0 {
        return (0.0, DurationSecs(0));
    }

    let offset = refresh_offset.as_secs();
    let desired_delay = if offset < lifetime_secs {
        lifetime_secs - offset
    } else {
        // Fallback: stale at 50% of lifetime (truncation is fine).
        lifetime_secs / 2
    };

    // Precision loss is negligible — token lifetimes are practical
    // values well within f64 mantissa range.
    let freshness = (desired_delay as f64) / (lifetime_secs as f64);
    let min_stale = min_refresh_period.as_secs().min(desired_delay);

    (freshness, DurationSecs(min_stale))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    /// Build a minimal valid config pointing at the mock server.
    fn test_config(server: &MockServer) -> OAuthClientConfig {
        OAuthClientConfig {
            token_endpoint: Some(
                Url::parse(&format!("http://localhost:{}/token", server.port())).unwrap(),
            ),
            client_id: "test-client".into(),
            client_secret: SecretString::new("test-secret"),
            http_config: Some(modkit_http::HttpClientConfig::for_testing()),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn request_token_basic_auth_success() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/token")
                .header_exists("authorization")
                .body_includes("grant_type=client_credentials");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"access_token":"tok-123","expires_in":3600,"token_type":"Bearer"}"#);
        });

        let mut source = OAuthTokenSource::new(&test_config(&server)).unwrap();
        let token = source.request_token().await.unwrap();

        assert_eq!(token.access_token().as_str(), "tok-123");
        assert_eq!(token.lifetime(), DurationSecs(3600));
        mock.assert();
    }

    #[tokio::test]
    async fn missing_expires_in_uses_default_ttl() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"access_token":"tok-456"}"#);
        });

        let mut source = OAuthTokenSource::new(&test_config(&server)).unwrap();
        let token = source.request_token().await.unwrap();

        // default_ttl from OAuthClientConfig::default() is 5 min = 300s
        assert_eq!(token.lifetime(), DurationSecs(300));
        mock.assert();
    }

    #[tokio::test]
    async fn expires_in_zero_honoured() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"access_token":"tok-zero","expires_in":0}"#);
        });

        let mut source = OAuthTokenSource::new(&test_config(&server)).unwrap();
        let token = source.request_token().await.unwrap();

        // Server-provided expires_in is honoured as-is; aliri handles refresh scheduling.
        assert_eq!(token.lifetime(), DurationSecs(0));
        mock.assert();
    }

    #[tokio::test]
    async fn unsupported_token_type_returns_error() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"access_token":"tok","token_type":"mac"}"#);
        });

        let mut source = OAuthTokenSource::new(&test_config(&server)).unwrap();
        let err = source.request_token().await.unwrap_err();

        assert!(
            matches!(err, TokenError::UnsupportedTokenType(ref t) if t == "mac"),
            "expected UnsupportedTokenType(\"mac\"), got: {err}"
        );
        mock.assert();
    }

    #[tokio::test]
    async fn empty_scopes_omits_scope_param() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/token")
                .body_includes("grant_type=client_credentials")
                .body_excludes("scope");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"access_token":"tok"}"#);
        });

        let mut source = OAuthTokenSource::new(&test_config(&server)).unwrap();
        source.request_token().await.unwrap();
        mock.assert();
    }

    #[tokio::test]
    async fn scopes_are_space_joined() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/token")
                .body_includes("scope=read+write");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"access_token":"tok"}"#);
        });

        let mut cfg = test_config(&server);
        cfg.scopes = vec!["read".into(), "write".into()];
        let mut source = OAuthTokenSource::new(&cfg).unwrap();
        source.request_token().await.unwrap();
        mock.assert();
    }

    #[tokio::test]
    async fn basic_auth_header_present() {
        let server = MockServer::start();

        let expected = format!(
            "Basic {}",
            general_purpose::STANDARD.encode("test-client:test-secret")
        );

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/token")
                .header("authorization", &expected);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"access_token":"tok"}"#);
        });

        let mut source = OAuthTokenSource::new(&test_config(&server)).unwrap();
        source.request_token().await.unwrap();
        mock.assert();
    }

    #[tokio::test]
    async fn form_auth_sends_credentials_in_body() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/token")
                .body_includes("client_id=test-client")
                .body_includes("client_secret=test-secret")
                .body_includes("grant_type=client_credentials");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"access_token":"tok"}"#);
        });

        let mut cfg = test_config(&server);
        cfg.auth_method = ClientAuthMethod::Form;
        let mut source = OAuthTokenSource::new(&cfg).unwrap();
        source.request_token().await.unwrap();
        mock.assert();
    }

    #[tokio::test]
    async fn form_auth_does_not_send_basic_header() {
        let server = MockServer::start();

        // Mock that REQUIRES a Basic auth header — should NOT be hit.
        let basic_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/token")
                .header_exists("authorization");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"access_token":"tok"}"#);
        });

        // Catch-all mock for the POST.
        let fallback_mock = server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"access_token":"tok"}"#);
        });

        let mut cfg = test_config(&server);
        cfg.auth_method = ClientAuthMethod::Form;
        let mut source = OAuthTokenSource::new(&cfg).unwrap();
        source.request_token().await.unwrap();

        assert_eq!(
            basic_mock.calls(),
            0,
            "Form auth must not send Authorization header"
        );
        fallback_mock.assert();
    }

    #[tokio::test]
    async fn extra_headers_are_applied() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/token")
                .header("x-vendor-key", "vendor-value");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"access_token":"tok"}"#);
        });

        let mut cfg = test_config(&server);
        cfg.extra_headers = vec![("x-vendor-key".into(), "vendor-value".into())];
        let mut source = OAuthTokenSource::new(&cfg).unwrap();
        source.request_token().await.unwrap();
        mock.assert();
    }

    #[tokio::test]
    async fn http_error_mapped_via_format_http_error() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(401)
                .header("content-type", "application/json")
                .body(r#"{"error":"invalid_client"}"#);
        });

        let mut source = OAuthTokenSource::new(&test_config(&server)).unwrap();
        let err = source.request_token().await.unwrap_err();

        assert!(
            matches!(
                err,
                TokenError::Http(ref msg)
                    if msg.contains("OAuth2 token")
                        && msg.contains("401")
            ),
            "expected Http error with OAuth2 token prefix and 401, got: {err}"
        );
        mock.assert();
    }

    #[test]
    fn config_error_when_token_endpoint_missing() {
        let cfg = OAuthClientConfig::default();
        let result = OAuthTokenSource::new(&cfg);
        let Err(err) = result else {
            panic!("expected ConfigError, got Ok");
        };
        assert!(
            matches!(err, TokenError::ConfigError(ref msg) if msg.contains("token_endpoint")),
            "expected ConfigError about token_endpoint, got: {err}"
        );
    }

    #[tokio::test]
    async fn bearer_case_insensitive() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"access_token":"tok","token_type":"bEaReR"}"#);
        });

        let mut source = OAuthTokenSource::new(&test_config(&server)).unwrap();
        let token = source.request_token().await.unwrap();

        assert_eq!(token.access_token().as_str(), "tok");
        mock.assert();
    }

    // -- refresh_params -------------------------------------------------------

    /// Helper: verify that the aliri formula
    /// `max(lifetime * freshness, min_stale) <= lifetime`
    /// holds for the given params.
    #[allow(clippy::cast_precision_loss)]
    fn assert_stale_before_expiry(lifetime: u64, freshness: f64, min_stale: DurationSecs) {
        let delay_a = (lifetime as f64) * freshness;
        let delay_b = min_stale.0 as f64;
        let delay = delay_a.max(delay_b);
        assert!(
            delay <= lifetime as f64,
            "stale ({delay}) must not exceed lifetime ({lifetime})"
        );
    }

    #[test]
    fn refresh_normal_token() {
        // 1-hour token, 30-min offset → stale at 50%
        let (r, ms) = refresh_params(
            3600,
            &Duration::from_secs(30 * 60),
            &Duration::from_secs(10),
        );
        assert!((r - 0.5).abs() < f64::EPSILON);
        assert_eq!(ms, DurationSecs(10));
        assert_stale_before_expiry(3600, r, ms);
    }

    #[test]
    fn refresh_short_lived_token() {
        // 20-min token, 30-min offset → fallback 0.5
        let (r, ms) = refresh_params(
            1200,
            &Duration::from_secs(30 * 60),
            &Duration::from_secs(10),
        );
        assert!((r - 0.5).abs() < f64::EPSILON);
        assert_eq!(ms, DurationSecs(10));
        assert_stale_before_expiry(1200, r, ms);
    }

    #[test]
    fn refresh_equal_lifetime_and_offset() {
        // 30-min token, 30-min offset → fallback 0.5
        let (r, ms) = refresh_params(
            1800,
            &Duration::from_secs(30 * 60),
            &Duration::from_secs(10),
        );
        assert!((r - 0.5).abs() < f64::EPSILON);
        assert_stale_before_expiry(1800, r, ms);
    }

    #[test]
    fn refresh_zero_lifetime() {
        // Both values must be zero so stale == expiry.
        let (r, ms) = refresh_params(0, &Duration::from_secs(30 * 60), &Duration::from_secs(10));
        assert!((r - 0.0).abs() < f64::EPSILON);
        assert_eq!(ms, DurationSecs(0));
    }

    #[test]
    fn refresh_small_offset() {
        // 5-min token, 1-min offset → stale at 80%
        let (r, ms) = refresh_params(300, &Duration::from_secs(60), &Duration::from_secs(10));
        assert!((r - 0.8).abs() < f64::EPSILON);
        assert_eq!(ms, DurationSecs(10));
        assert_stale_before_expiry(300, r, ms);
    }

    #[test]
    fn refresh_zero_offset() {
        // No offset → stale at 100% (only stale when expired)
        let (r, ms) = refresh_params(3600, &Duration::from_secs(0), &Duration::from_secs(10));
        assert!((r - 1.0).abs() < f64::EPSILON);
        assert_eq!(ms, DurationSecs(10));
        assert_stale_before_expiry(3600, r, ms);
    }

    #[test]
    fn refresh_min_period_exceeds_lifetime() {
        // min_refresh_period (600s) > lifetime (300s) — must be capped
        let (r, ms) = refresh_params(300, &Duration::from_secs(60), &Duration::from_secs(600));
        // desired_delay = 300 - 60 = 240
        assert!((r - 0.8).abs() < f64::EPSILON);
        // min_stale capped to desired_delay, not 600
        assert_eq!(ms, DurationSecs(240));
        assert_stale_before_expiry(300, r, ms);
    }

    #[test]
    fn refresh_zero_lifetime_nonzero_min_period() {
        // expires_in=0 with min_refresh_period=10 — both must be zero
        let (r, ms) = refresh_params(0, &Duration::from_secs(30 * 60), &Duration::from_secs(10));
        assert!((r - 0.0).abs() < f64::EPSILON);
        assert_eq!(ms, DurationSecs(0));
    }
}
