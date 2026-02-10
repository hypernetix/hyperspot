use std::fmt;
use std::time::Duration;
use url::Url;

use super::error::TokenError;
use super::types::{ClientAuthMethod, SecretString};

/// Configuration for an outbound `OAuth2` client credentials flow.
///
/// Exactly one of [`token_endpoint`](Self::token_endpoint) or
/// [`issuer_url`](Self::issuer_url) must be set.  Call
/// [`validate`](Self::validate) to enforce this constraint.
///
/// `Debug` is manually implemented to redact [`client_secret`](Self::client_secret).
pub struct OAuthClientConfig {
    // ---- endpoint resolution ------------------------------------------------
    /// Direct token endpoint URL (mutually exclusive with `issuer_url`).
    pub token_endpoint: Option<Url>,

    /// OIDC issuer URL for discovery (mutually exclusive with `token_endpoint`).
    /// The actual token endpoint is resolved via
    /// `{issuer_url}/.well-known/openid-configuration`.
    pub issuer_url: Option<Url>,

    // ---- credentials --------------------------------------------------------
    /// `OAuth2` client identifier.
    pub client_id: String,

    /// `OAuth2` client secret (redacted in `Debug` output).
    pub client_secret: SecretString,

    /// Requested scopes (normalized once, stable order).
    pub scopes: Vec<String>,

    /// How client credentials are transmitted to the token endpoint.
    pub auth_method: ClientAuthMethod,

    /// Extra headers attached to every token request (vendor quirks).
    pub extra_headers: Vec<(String, String)>,

    // ---- refresh policy -----------------------------------------------------
    /// How far before expiry the token should be refreshed (default: 30 min).
    pub refresh_offset: Duration,

    /// Maximum random jitter added to the refresh offset (default: 5 min).
    pub jitter_max: Duration,

    /// Minimum period between consecutive refresh attempts (default: 10 s).
    pub min_refresh_period: Duration,

    /// Fallback TTL when the token endpoint omits `expires_in` (default: 5 min).
    pub default_ttl: Duration,

    // ---- HTTP client --------------------------------------------------------
    /// Override for the internal HTTP client configuration.
    /// When `None`,
    /// [`HttpClientConfig::token_endpoint()`](modkit_http::HttpClientConfig::token_endpoint)
    /// is used.
    pub http_config: Option<modkit_http::HttpClientConfig>,
}

impl OAuthClientConfig {
    /// Validate that the configuration is self-consistent.
    ///
    /// # Errors
    ///
    /// Returns [`TokenError::ConfigError`] if:
    /// - both `token_endpoint` and `issuer_url` are set, or
    /// - neither is set.
    pub fn validate(&self) -> Result<(), TokenError> {
        if self.client_id.trim().is_empty() {
            return Err(TokenError::ConfigError(
                "client_id must not be empty".into(),
            ));
        }
        if self.client_secret.expose().is_empty() {
            return Err(TokenError::ConfigError(
                "client_secret must not be empty".into(),
            ));
        }
        match (&self.token_endpoint, &self.issuer_url) {
            (Some(_), Some(_)) => Err(TokenError::ConfigError(
                "token_endpoint and issuer_url are mutually exclusive".into(),
            )),
            (None, None) => Err(TokenError::ConfigError(
                "one of token_endpoint or issuer_url must be set".into(),
            )),
            _ => Ok(()),
        }
    }
}

impl Clone for OAuthClientConfig {
    fn clone(&self) -> Self {
        Self {
            token_endpoint: self.token_endpoint.clone(),
            issuer_url: self.issuer_url.clone(),
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            scopes: self.scopes.clone(),
            auth_method: self.auth_method,
            extra_headers: self.extra_headers.clone(),
            refresh_offset: self.refresh_offset,
            jitter_max: self.jitter_max,
            min_refresh_period: self.min_refresh_period,
            default_ttl: self.default_ttl,
            http_config: self.http_config.clone(),
        }
    }
}

/// `Debug` redacts `client_secret` to prevent accidental exposure in logs.
impl fmt::Debug for OAuthClientConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted_headers: Vec<_> = self
            .extra_headers
            .iter()
            .map(|(k, _)| (k.as_str(), "[REDACTED]"))
            .collect();
        f.debug_struct("OAuthClientConfig")
            .field("token_endpoint", &self.token_endpoint)
            .field("issuer_url", &self.issuer_url)
            .field("client_id", &self.client_id)
            .field("client_secret", &"[REDACTED]")
            .field("scopes", &self.scopes)
            .field("auth_method", &self.auth_method)
            .field("extra_headers", &redacted_headers)
            .field("refresh_offset", &self.refresh_offset)
            .field("jitter_max", &self.jitter_max)
            .field("min_refresh_period", &self.min_refresh_period)
            .field("default_ttl", &self.default_ttl)
            .field("http_config", &self.http_config)
            .finish()
    }
}

impl Default for OAuthClientConfig {
    fn default() -> Self {
        Self {
            token_endpoint: None,
            issuer_url: None,
            client_id: String::new(),
            client_secret: SecretString::new(String::new()),
            scopes: Vec::new(),
            auth_method: ClientAuthMethod::default(),
            extra_headers: Vec::new(),
            refresh_offset: Duration::from_secs(30 * 60),
            jitter_max: Duration::from_secs(5 * 60),
            min_refresh_period: Duration::from_secs(10),
            default_ttl: Duration::from_secs(5 * 60),
            http_config: None,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    fn test_url(s: &str) -> Url {
        Url::parse(s).unwrap()
    }

    // ---- validate -----------------------------------------------------------

    /// Returns a minimal valid config (credentials + one endpoint).
    fn valid_base() -> OAuthClientConfig {
        OAuthClientConfig {
            client_id: "my-client".into(),
            client_secret: SecretString::new("my-secret"),
            ..Default::default()
        }
    }

    #[test]
    fn validate_ok_with_token_endpoint_only() {
        let cfg = OAuthClientConfig {
            token_endpoint: Some(test_url("https://auth.example.com/token")),
            ..valid_base()
        };
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn validate_ok_with_issuer_url_only() {
        let cfg = OAuthClientConfig {
            issuer_url: Some(test_url("https://auth.example.com")),
            ..valid_base()
        };
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn validate_err_when_both_set() {
        let cfg = OAuthClientConfig {
            token_endpoint: Some(test_url("https://a.example.com/token")),
            issuer_url: Some(test_url("https://b.example.com")),
            ..valid_base()
        };
        let err = cfg.validate().unwrap_err();
        assert!(
            err.to_string().contains("mutually exclusive"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn validate_err_when_neither_set() {
        let cfg = valid_base();
        let err = cfg.validate().unwrap_err();
        assert!(
            err.to_string().contains("must be set"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn validate_err_when_client_id_empty() {
        let cfg = OAuthClientConfig {
            token_endpoint: Some(test_url("https://auth.example.com/token")),
            client_id: String::new(),
            client_secret: SecretString::new("my-secret"),
            ..Default::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(
            err.to_string().contains("client_id"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn validate_err_when_client_id_whitespace() {
        let cfg = OAuthClientConfig {
            token_endpoint: Some(test_url("https://auth.example.com/token")),
            client_id: "   ".into(),
            client_secret: SecretString::new("my-secret"),
            ..Default::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(
            err.to_string().contains("client_id"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn validate_err_when_client_secret_empty() {
        let cfg = OAuthClientConfig {
            token_endpoint: Some(test_url("https://auth.example.com/token")),
            client_id: "my-client".into(),
            client_secret: SecretString::new(""),
            ..Default::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(
            err.to_string().contains("client_secret"),
            "unexpected error: {err}"
        );
    }

    // ---- Debug redaction ----------------------------------------------------

    #[test]
    fn debug_redacts_client_secret() {
        let cfg = OAuthClientConfig {
            token_endpoint: Some(test_url("https://auth.example.com/token")),
            client_id: "my-client".into(),
            client_secret: SecretString::new("super-secret"),
            ..Default::default()
        };
        let dbg = format!("{cfg:?}");
        assert!(dbg.contains("[REDACTED]"), "Debug must contain [REDACTED]");
        assert!(
            !dbg.contains("super-secret"),
            "Debug must not contain the raw secret"
        );
        assert!(dbg.contains("my-client"), "Debug should contain client_id");
    }

    #[test]
    fn debug_redacts_extra_header_values() {
        let cfg = OAuthClientConfig {
            token_endpoint: Some(test_url("https://auth.example.com/token")),
            client_id: "my-client".into(),
            client_secret: SecretString::new("s"),
            extra_headers: vec![("x-api-key".into(), "secret-api-key-value".into())],
            ..Default::default()
        };
        let dbg = format!("{cfg:?}");
        assert!(
            dbg.contains("x-api-key"),
            "Debug should contain header name"
        );
        assert!(
            !dbg.contains("secret-api-key-value"),
            "Debug must not contain header value"
        );
    }

    // ---- Default ------------------------------------------------------------

    #[test]
    fn default_durations() {
        let cfg = OAuthClientConfig::default();
        assert_eq!(cfg.refresh_offset, Duration::from_secs(30 * 60));
        assert_eq!(cfg.jitter_max, Duration::from_secs(5 * 60));
        assert_eq!(cfg.min_refresh_period, Duration::from_secs(10));
        assert_eq!(cfg.default_ttl, Duration::from_secs(5 * 60));
        assert_eq!(cfg.auth_method, ClientAuthMethod::Basic);
    }
}
