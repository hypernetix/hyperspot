use crate::{claims_error::ClaimsError, plugin_traits::KeyProvider};
use arc_swap::ArcSwap;
use async_trait::async_trait;
use jsonwebtoken::{DecodingKey, Header, Validation, decode, decode_header};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    #[serde(rename = "use")]
    #[allow(dead_code)]
    use_: Option<String>,
    n: String,
    e: String,
    #[allow(dead_code)]
    alg: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

/// JWKS-based key provider with lock-free reads
///
/// Uses `ArcSwap` for lock-free key lookups and background refresh with exponential backoff.
#[must_use]
pub struct JwksKeyProvider {
    /// JWKS endpoint URL
    jwks_uri: String,

    /// Keys stored in `ArcSwap` for lock-free reads
    keys: Arc<ArcSwap<HashMap<String, DecodingKey>>>,

    /// Last refresh time and error tracking for backoff
    refresh_state: Arc<RwLock<RefreshState>>,

    /// Shared HTTP client for JWKS fetches (pooled connections)
    /// `HttpClient` is `Clone + Send + Sync`, no external locking needed.
    client: modkit_http::HttpClient,

    /// Refresh interval (default: 5 minutes)
    refresh_interval: Duration,

    /// Maximum backoff duration (default: 1 hour)
    max_backoff: Duration,

    /// Cooldown for on-demand refresh (default: 60 seconds)
    on_demand_refresh_cooldown: Duration,
}

#[derive(Debug, Default)]
struct RefreshState {
    last_refresh: Option<Instant>,
    last_on_demand_refresh: Option<Instant>,
    consecutive_failures: u32,
    last_error: Option<String>,
    failed_kids: HashSet<String>,
}

impl JwksKeyProvider {
    /// Create a new JWKS key provider
    ///
    /// # Errors
    /// Returns error if HTTP client initialization fails (e.g., TLS setup)
    pub fn new(jwks_uri: impl Into<String>) -> Result<Self, modkit_http::HttpError> {
        Self::with_http_timeout(jwks_uri, Duration::from_secs(10))
    }

    /// Create a new JWKS key provider with custom HTTP timeout
    ///
    /// # Errors
    /// Returns error if HTTP client initialization fails (e.g., TLS setup)
    pub fn with_http_timeout(
        jwks_uri: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self, modkit_http::HttpError> {
        let client = modkit_http::HttpClient::builder()
            .timeout(timeout)
            .retry(None) // JWKS provider handles its own retry logic
            .build()?;

        Ok(Self {
            jwks_uri: jwks_uri.into(),
            keys: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            refresh_state: Arc::new(RwLock::new(RefreshState::default())),
            client,
            refresh_interval: Duration::from_secs(300), // 5 minutes
            max_backoff: Duration::from_secs(3600),     // 1 hour
            on_demand_refresh_cooldown: Duration::from_secs(60), // 1 minute
        })
    }

    /// Create a new JWKS key provider (alias for new, kept for compatibility)
    ///
    /// # Errors
    /// Returns error if HTTP client initialization fails (e.g., TLS setup)
    pub fn try_new(jwks_uri: impl Into<String>) -> Result<Self, modkit_http::HttpError> {
        Self::new(jwks_uri)
    }

    /// Create with custom refresh interval
    pub fn with_refresh_interval(mut self, interval: Duration) -> Self {
        self.refresh_interval = interval;
        self
    }

    /// Create with custom max backoff
    pub fn with_max_backoff(mut self, max_backoff: Duration) -> Self {
        self.max_backoff = max_backoff;
        self
    }

    /// Create with custom on-demand refresh cooldown
    pub fn with_on_demand_refresh_cooldown(mut self, cooldown: Duration) -> Self {
        self.on_demand_refresh_cooldown = cooldown;
        self
    }

    /// Fetch JWKS from the endpoint
    async fn fetch_jwks(&self) -> Result<HashMap<String, DecodingKey>, ClaimsError> {
        // HttpClient is Clone + Send + Sync, no locking needed
        let jwks: JwksResponse = self
            .client
            .get(&self.jwks_uri)
            .send()
            .await
            .map_err(|e| map_http_error(&e))?
            .json()
            .await
            .map_err(|e| map_http_error(&e))?;

        let mut keys = HashMap::new();
        for jwk in jwks.keys {
            if jwk.kty == "RSA" {
                let key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
                    .map_err(|e| ClaimsError::JwksFetchFailed(format!("Invalid RSA key: {e}")))?;
                keys.insert(jwk.kid, key);
            }
        }

        if keys.is_empty() {
            return Err(ClaimsError::JwksFetchFailed(
                "No valid RSA keys found in JWKS".into(),
            ));
        }

        Ok(keys)
    }

    /// Calculate backoff duration based on consecutive failures
    fn calculate_backoff(&self, failures: u32) -> Duration {
        let base = Duration::from_secs(60); // 1 minute base
        let exponential = base * 2u32.pow(failures.min(10)); // Cap at 2^10
        exponential.min(self.max_backoff)
    }

    /// Check if refresh is needed based on interval and backoff
    async fn should_refresh(&self) -> bool {
        let state = self.refresh_state.read().await;

        match state.last_refresh {
            None => true, // Never refreshed
            Some(last) => {
                let elapsed = last.elapsed();
                if state.consecutive_failures == 0 {
                    // Normal refresh interval
                    elapsed >= self.refresh_interval
                } else {
                    // Exponential backoff
                    elapsed >= self.calculate_backoff(state.consecutive_failures)
                }
            }
        }
    }

    /// Perform key refresh with error tracking
    async fn perform_refresh(&self) -> Result<(), ClaimsError> {
        match self.fetch_jwks().await {
            Ok(new_keys) => {
                // Update keys atomically
                self.keys.store(Arc::new(new_keys));

                // Update refresh state
                let mut state = self.refresh_state.write().await;
                state.last_refresh = Some(Instant::now());
                state.consecutive_failures = 0;
                state.last_error = None;

                Ok(())
            }
            Err(e) => {
                // Update failure state
                let mut state = self.refresh_state.write().await;
                state.last_refresh = Some(Instant::now());
                state.consecutive_failures += 1;
                state.last_error = Some(e.to_string());

                Err(e)
            }
        }
    }

    /// Check if a key exists in the cache
    fn key_exists(&self, kid: &str) -> bool {
        let keys = self.keys.load();
        keys.contains_key(kid)
    }

    /// Check if we're in cooldown period and handle throttling logic
    async fn check_refresh_throttle(&self, kid: &str) -> Result<(), ClaimsError> {
        let state = self.refresh_state.read().await;
        if let Some(last_on_demand) = state.last_on_demand_refresh {
            let elapsed = last_on_demand.elapsed();
            if elapsed < self.on_demand_refresh_cooldown {
                let remaining = self.on_demand_refresh_cooldown.saturating_sub(elapsed);
                tracing::debug!(
                    kid = kid,
                    remaining_secs = remaining.as_secs(),
                    "On-demand JWKS refresh throttled (cooldown active)"
                );

                // Check if this kid has failed before
                if state.failed_kids.contains(kid) {
                    tracing::warn!(
                        kid = kid,
                        "Unknown kid repeatedly requested despite recent refresh attempts"
                    );
                }

                return Err(ClaimsError::UnknownKeyId(kid.to_owned()));
            }
        }
        Ok(())
    }

    /// Update state after successful refresh and check if kid is now available
    async fn handle_refresh_success(&self, kid: &str) -> Result<(), ClaimsError> {
        let mut state = self.refresh_state.write().await;
        state.last_on_demand_refresh = Some(Instant::now());

        // Check if the kid now exists
        if self.key_exists(kid) {
            // Kid found - remove from failed list if present
            state.failed_kids.remove(kid);
        } else {
            // Kid still not found after refresh - track it
            state.failed_kids.insert(kid.to_owned());
            tracing::warn!(
                kid = kid,
                "Kid still not found after on-demand JWKS refresh"
            );
        }

        Ok(())
    }

    /// Update state after failed refresh
    async fn handle_refresh_failure(&self, kid: &str, error: ClaimsError) -> ClaimsError {
        let mut state = self.refresh_state.write().await;
        state.last_on_demand_refresh = Some(Instant::now());
        state.failed_kids.insert(kid.to_owned());
        error
    }

    /// Try to refresh keys if unknown kid is encountered
    /// Implements throttling to prevent excessive refreshes
    async fn on_demand_refresh(&self, kid: &str) -> Result<(), ClaimsError> {
        // Check if key exists
        if self.key_exists(kid) {
            return Ok(());
        }

        // Check if we're in cooldown period
        self.check_refresh_throttle(kid).await?;

        // Attempt refresh and track the kid if it fails
        tracing::info!(
            kid = kid,
            "Performing on-demand JWKS refresh for unknown kid"
        );

        match self.perform_refresh().await {
            Ok(()) => self.handle_refresh_success(kid).await,
            Err(e) => Err(self.handle_refresh_failure(kid, e).await),
        }
    }

    /// Get a key by kid (lock-free read)
    fn get_key(&self, kid: &str) -> Option<DecodingKey> {
        let keys = self.keys.load();
        keys.get(kid).cloned()
    }

    /// Validate JWT and decode into header + raw claims
    fn validate_token(
        token: &str,
        key: &DecodingKey,
        header: &Header,
    ) -> Result<Value, ClaimsError> {
        let mut validation = Validation::new(header.alg);

        // Disable all built-in validations - we'll do them separately
        validation.validate_exp = false;
        validation.validate_nbf = false;
        validation.validate_aud = false;

        // Don't require any standard claims
        let empty_claims: &[&str] = &[];
        validation.set_required_spec_claims(empty_claims);

        let token_data = decode::<Value>(token, key, &validation)
            .map_err(|e| ClaimsError::DecodeFailed(format!("JWT validation failed: {e}")))?;

        Ok(token_data.claims)
    }
}

#[async_trait]
impl KeyProvider for JwksKeyProvider {
    fn name(&self) -> &'static str {
        "jwks"
    }

    async fn validate_and_decode(&self, token: &str) -> Result<(Header, Value), ClaimsError> {
        // Strip "Bearer " prefix if present
        let token = token.trim_start_matches("Bearer ").trim();

        // Decode header to get kid and algorithm
        let header = decode_header(token)
            .map_err(|e| ClaimsError::DecodeFailed(format!("Invalid JWT header: {e}")))?;

        let kid = header
            .kid
            .as_ref()
            .ok_or_else(|| ClaimsError::DecodeFailed("Missing kid in JWT header".into()))?;

        // Try to get key from cache
        let key = if let Some(k) = self.get_key(kid) {
            k
        } else {
            // Key not in cache, try on-demand refresh
            self.on_demand_refresh(kid).await?;

            // Try again after refresh
            self.get_key(kid)
                .ok_or_else(|| ClaimsError::UnknownKeyId(kid.clone()))?
        };

        // Validate signature and decode claims
        let claims = Self::validate_token(token, &key, &header)?;

        Ok((header, claims))
    }

    async fn refresh_keys(&self) -> Result<(), ClaimsError> {
        if self.should_refresh().await {
            self.perform_refresh().await
        } else {
            Ok(())
        }
    }
}

/// Background task to periodically refresh JWKS
///
/// This task will run until the `cancellation_token` is cancelled, enabling
/// graceful shutdown per `ModKit` patterns. Without cancellation support, this
/// task would run indefinitely and potentially cause process hang on shutdown.
///
/// # Example
///
/// ```ignore
/// use tokio_util::sync::CancellationToken;
/// use std::sync::Arc;
///
/// let provider = Arc::new(JwksKeyProvider::new("https://issuer/.well-known/jwks.json")?);
/// let cancel_token = CancellationToken::new();
///
/// // Spawn the refresh task
/// let task_handle = tokio::spawn(run_jwks_refresh_task(provider.clone(), cancel_token.clone()));
///
/// // On shutdown:
/// cancel_token.cancel();
/// task_handle.await?;
/// ```
pub async fn run_jwks_refresh_task(
    provider: Arc<JwksKeyProvider>,
    cancellation_token: CancellationToken,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(60)); // Check every minute

    loop {
        tokio::select! {
            () = cancellation_token.cancelled() => {
                tracing::info!("JWKS refresh task shutting down");
                break;
            }
            _ = interval.tick() => {
                if let Err(e) = provider.refresh_keys().await {
                    tracing::warn!("JWKS refresh failed: {}", e);
                }
            }
        }
    }
}

/// Map `HttpError` variants to appropriate `ClaimsError` messages
fn map_http_error(e: &modkit_http::HttpError) -> ClaimsError {
    ClaimsError::JwksFetchFailed(crate::http_error::format_http_error(e, "JWKS"))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    /// Create a test provider with insecure HTTP allowed (for httpmock) and no retries
    fn test_provider_with_http(uri: &str) -> JwksKeyProvider {
        let client = modkit_http::HttpClient::builder()
            .timeout(Duration::from_secs(5))
            .retry(None)
            .allow_insecure_http()
            .build()
            .expect("failed to create test HTTP client");

        JwksKeyProvider {
            jwks_uri: uri.to_owned(),
            keys: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            refresh_state: Arc::new(RwLock::new(RefreshState::default())),
            client,
            refresh_interval: Duration::from_secs(300),
            max_backoff: Duration::from_secs(3600),
            on_demand_refresh_cooldown: Duration::from_secs(60),
        }
    }

    /// Create a basic test provider (HTTPS only, for non-network tests)
    fn test_provider(uri: &str) -> JwksKeyProvider {
        JwksKeyProvider::new(uri).expect("failed to create test provider")
    }

    /// Valid JWKS JSON response with a single RSA key
    fn valid_jwks_json() -> &'static str {
        r#"{
            "keys": [{
                "kty": "RSA",
                "kid": "test-key-1",
                "use": "sig",
                "n": "0vx7agoebGcQSuuPiLJXZptN9nndrQmbXEps2aiAFbWhM78LhWx4cbbfAAtVT86zwu1RK7aPFFxuhDR1L6tSoc_BJECPebWKRXjBZCiFV4n3oknjhMstn64tZ_2W-5JsGY4Hc5n9yBXArwl93lqt7_RN5w6Cf0h4QyQ5v-65YGjQR0_FDW2QvzqY368QQMicAtaSqzs8KJZgnYb9c7d0zgdAZHzu6qMQvRL5hajrn1n91CbOpbISD08qNLyrdkt-bFTWhAI4vMQFh6WeZu0fM4lFd2NcRwr3XPksINHaQ-G_xBniIqbw0Ls1jF44-csFCur-kEgU8awapJzKnqDKgw",
                "e": "AQAB",
                "alg": "RS256"
            }]
        }"#
    }

    #[tokio::test]
    async fn test_calculate_backoff() {
        let provider = test_provider("https://example.com/jwks");

        assert_eq!(provider.calculate_backoff(0), Duration::from_secs(60));
        assert_eq!(provider.calculate_backoff(1), Duration::from_secs(120));
        assert_eq!(provider.calculate_backoff(2), Duration::from_secs(240));
        assert_eq!(provider.calculate_backoff(3), Duration::from_secs(480));

        // Should cap at max_backoff
        assert_eq!(provider.calculate_backoff(100), provider.max_backoff);
    }

    #[tokio::test]
    async fn test_should_refresh_on_first_call() {
        let provider = test_provider("https://example.com/jwks");
        assert!(provider.should_refresh().await);
    }

    #[tokio::test]
    async fn test_key_storage() {
        let provider = test_provider("https://example.com/jwks");

        // Initially empty
        assert!(provider.get_key("test-kid").is_none());

        // Store a dummy key
        let mut keys = HashMap::new();
        keys.insert("test-kid".to_owned(), DecodingKey::from_secret(b"secret"));
        provider.keys.store(Arc::new(keys));

        // Should be retrievable
        assert!(provider.get_key("test-kid").is_some());
    }

    #[tokio::test]
    async fn test_on_demand_refresh_returns_ok_when_key_exists() {
        let provider = test_provider("https://example.com/jwks");

        // Pre-populate with a key
        let mut keys = HashMap::new();
        keys.insert(
            "existing-kid".to_owned(),
            DecodingKey::from_secret(b"secret"),
        );
        provider.keys.store(Arc::new(keys));

        // Should return Ok immediately without any refresh
        let result = provider.on_demand_refresh("existing-kid").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_try_new_returns_result() {
        // Valid URL should work
        let result = JwksKeyProvider::try_new("https://example.com/jwks");
        assert!(result.is_ok());
    }

    // ==================== httpmock-based tests ====================

    #[tokio::test]
    async fn test_fetch_jwks_success_with_valid_json() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET).path("/jwks");
            then.status(200)
                .header("content-type", "application/json")
                .body(valid_jwks_json());
        });

        let jwks_url = server.url("/jwks");
        let provider = test_provider_with_http(&jwks_url);

        let result = provider.perform_refresh().await;
        assert!(result.is_ok(), "Expected success, got: {result:?}");

        // Verify key was stored
        assert!(
            provider.get_key("test-key-1").is_some(),
            "Expected key 'test-key-1' to be stored"
        );

        mock.assert();
    }

    #[tokio::test]
    async fn test_fetch_jwks_http_404_error_mapping() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET).path("/jwks");
            then.status(404).body("Not Found");
        });

        let jwks_url = server.url("/jwks");
        let provider = test_provider_with_http(&jwks_url);

        let result = provider.perform_refresh().await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("JWKS HTTP 404"),
            "Expected error to contain 'JWKS HTTP 404', got: {err_msg}"
        );
        // Must NOT say "parse"
        assert!(
            !err_msg.to_lowercase().contains("parse"),
            "HTTP status error should not mention 'parse', got: {err_msg}"
        );

        mock.assert();
    }

    #[tokio::test]
    async fn test_fetch_jwks_http_500_error_mapping() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET).path("/jwks");
            then.status(500).body("Internal Server Error");
        });

        let jwks_url = server.url("/jwks");
        let provider = test_provider_with_http(&jwks_url);

        let result = provider.perform_refresh().await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("JWKS HTTP 500"),
            "Expected error to contain 'JWKS HTTP 500', got: {err_msg}"
        );

        mock.assert();
    }

    #[tokio::test]
    async fn test_fetch_jwks_invalid_json_error_mapping() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET).path("/jwks");
            then.status(200)
                .header("content-type", "application/json")
                .body("this is not valid json");
        });

        let jwks_url = server.url("/jwks");
        let provider = test_provider_with_http(&jwks_url);

        let result = provider.perform_refresh().await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("JWKS JSON parse failed"),
            "Expected error to contain 'JWKS JSON parse failed', got: {err_msg}"
        );

        mock.assert();
    }

    #[tokio::test]
    async fn test_fetch_jwks_empty_keys_error() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET).path("/jwks");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"keys": []}"#);
        });

        let jwks_url = server.url("/jwks");
        let provider = test_provider_with_http(&jwks_url);

        let result = provider.perform_refresh().await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("No valid RSA keys"),
            "Expected error about no RSA keys, got: {err_msg}"
        );

        mock.assert();
    }

    #[tokio::test]
    async fn test_on_demand_refresh_respects_cooldown() {
        let server = MockServer::start();

        // First request will return 404
        let mock = server.mock(|when, then| {
            when.method(GET).path("/jwks");
            then.status(404).body("Not Found");
        });

        let jwks_url = server.url("/jwks");
        let provider = test_provider_with_http(&jwks_url)
            .with_on_demand_refresh_cooldown(Duration::from_secs(60));

        // First attempt - should try to refresh and fail
        let result1 = provider.on_demand_refresh("test-kid").await;
        assert!(result1.is_err());

        // Immediate second attempt - should be throttled (no network call)
        let result2 = provider.on_demand_refresh("test-kid").await;
        assert!(result2.is_err());

        // Should return UnknownKeyId due to cooldown
        match result2.unwrap_err() {
            ClaimsError::UnknownKeyId(_) => {}
            other => panic!("Expected UnknownKeyId during cooldown, got: {other:?}"),
        }

        // Only one request should have been made (first attempt)
        mock.assert_calls(1);
    }

    #[tokio::test]
    async fn test_on_demand_refresh_tracks_failed_kids() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(GET).path("/jwks");
            then.status(404).body("Not Found");
        });

        let jwks_url = server.url("/jwks");
        let provider = test_provider_with_http(&jwks_url)
            .with_on_demand_refresh_cooldown(Duration::from_millis(100));

        // Attempt refresh - will fail and track the kid
        let result = provider.on_demand_refresh("failed-kid").await;
        assert!(result.is_err());

        // Check that failed_kids contains the kid
        let state = provider.refresh_state.read().await;
        assert!(state.failed_kids.contains("failed-kid"));
    }

    #[tokio::test]
    async fn test_perform_refresh_updates_state_on_failure() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(GET).path("/jwks");
            then.status(500).body("Server Error");
        });

        let jwks_url = server.url("/jwks");
        let provider = test_provider_with_http(&jwks_url);

        // Mark as previously failed
        {
            let mut state = provider.refresh_state.write().await;
            state.consecutive_failures = 3;
            state.last_error = Some("Previous error".to_owned());
        }

        // This will fail
        _ = provider.perform_refresh().await;

        // Check that consecutive_failures increased
        let state = provider.refresh_state.read().await;
        assert_eq!(state.consecutive_failures, 4);
        assert!(state.last_error.is_some());
    }

    #[tokio::test]
    async fn test_perform_refresh_resets_state_on_success() {
        let server = MockServer::start();

        server.mock(|when, then| {
            when.method(GET).path("/jwks");
            then.status(200)
                .header("content-type", "application/json")
                .body(valid_jwks_json());
        });

        let jwks_url = server.url("/jwks");
        let provider = test_provider_with_http(&jwks_url);

        // Mark as previously failed
        {
            let mut state = provider.refresh_state.write().await;
            state.consecutive_failures = 5;
            state.last_error = Some("Previous error".to_owned());
        }

        // This should succeed
        let result = provider.perform_refresh().await;
        assert!(result.is_ok());

        // Check that state was reset
        let state = provider.refresh_state.read().await;
        assert_eq!(state.consecutive_failures, 0);
        assert!(state.last_error.is_none());
    }

    #[tokio::test]
    async fn test_validate_and_decode_with_missing_kid() {
        let server = MockServer::start();

        // Return valid JWKS but without the requested kid
        server.mock(|when, then| {
            when.method(GET).path("/jwks");
            then.status(200)
                .header("content-type", "application/json")
                .body(valid_jwks_json());
        });

        let jwks_url = server.url("/jwks");
        let provider = test_provider_with_http(&jwks_url)
            .with_on_demand_refresh_cooldown(Duration::from_millis(100));

        // Create a minimal JWT with a kid that doesn't exist in JWKS
        // Header: {"alg":"RS256","kid":"nonexistent-kid"}
        let token = "eyJhbGciOiJSUzI1NiIsImtpZCI6Im5vbmV4aXN0ZW50LWtpZCJ9.\
                     eyJzdWIiOiIxMjM0NTY3ODkwIn0.invalid";

        // Should attempt on-demand refresh but kid still won't exist
        let result = provider.validate_and_decode(token).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ClaimsError::UnknownKeyId(kid) => {
                assert_eq!(kid, "nonexistent-kid");
            }
            other => panic!("Expected UnknownKeyId, got: {other:?}"),
        }
    }
}
