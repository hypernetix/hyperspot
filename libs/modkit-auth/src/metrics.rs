/// Metrics tracking for auth events
///
/// This module provides a trait-based approach to metrics that can be
/// implemented with various backends (Prometheus, StatsD, etc.)
/// Auth event types for metrics tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthEvent {
    /// JWT validation succeeded
    JwtValid,

    /// JWT validation failed
    JwtInvalid,

    /// JWKS refresh succeeded
    JwksRefreshSuccess,

    /// JWKS refresh failed
    JwksRefreshFailure,

    /// Opaque token validation succeeded
    OpaqueTokenValid,

    /// Opaque token validation failed
    OpaqueTokenInvalid,
}

impl AuthEvent {
    /// Get the metric name for this event
    pub fn metric_name(&self) -> &'static str {
        match self {
            AuthEvent::JwtValid => "auth.jwt.valid",
            AuthEvent::JwtInvalid => "auth.jwt.invalid",
            AuthEvent::JwksRefreshSuccess => "auth.jwks.refresh.ok",
            AuthEvent::JwksRefreshFailure => "auth.jwks.refresh.fail",
            AuthEvent::OpaqueTokenValid => "auth.opaque.valid",
            AuthEvent::OpaqueTokenInvalid => "auth.opaque.invalid",
        }
    }
}

/// Labels for auth metrics
#[derive(Default, Debug, Clone)]
pub struct AuthMetricLabels {
    /// Provider name (e.g., "keycloak", "oidc_default")
    pub provider: Option<String>,

    /// Issuer URL
    pub issuer: Option<String>,

    /// Key ID (for JWKS)
    pub kid: Option<String>,

    /// Error type (for failures)
    pub error_type: Option<String>,
}

impl AuthMetricLabels {
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = Some(issuer.into());
        self
    }

    pub fn with_kid(mut self, kid: impl Into<String>) -> Self {
        self.kid = Some(kid.into());
        self
    }

    pub fn with_error_type(mut self, error_type: impl Into<String>) -> Self {
        self.error_type = Some(error_type.into());
        self
    }
}

/// Trait for metrics backends
pub trait AuthMetrics: Send + Sync {
    /// Record an auth event
    fn record_event(&self, event: AuthEvent, labels: &AuthMetricLabels);

    /// Record validation duration
    fn record_duration(&self, duration_ms: u64, labels: &AuthMetricLabels);
}

/// No-op metrics implementation (default)
#[derive(Debug, Clone, Copy)]
pub struct NoOpMetrics;

impl AuthMetrics for NoOpMetrics {
    fn record_event(&self, _event: AuthEvent, _labels: &AuthMetricLabels) {
        // No-op
    }

    fn record_duration(&self, _duration_ms: u64, _labels: &AuthMetricLabels) {
        // No-op
    }
}

/// Logging-based metrics implementation (for debugging)
#[derive(Debug, Clone, Copy)]
pub struct LoggingMetrics;

impl AuthMetrics for LoggingMetrics {
    fn record_event(&self, event: AuthEvent, labels: &AuthMetricLabels) {
        tracing::debug!(
            metric = event.metric_name(),
            provider = ?labels.provider,
            issuer = ?labels.issuer,
            kid = ?labels.kid,
            error_type = ?labels.error_type,
            "Auth event recorded"
        );
    }

    fn record_duration(&self, duration_ms: u64, labels: &AuthMetricLabels) {
        tracing::debug!(
            metric = "auth.validation.duration_ms",
            duration_ms = duration_ms,
            provider = ?labels.provider,
            issuer = ?labels.issuer,
            "Validation duration recorded"
        );
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_auth_event_metric_names() {
        assert_eq!(AuthEvent::JwtValid.metric_name(), "auth.jwt.valid");
        assert_eq!(AuthEvent::JwtInvalid.metric_name(), "auth.jwt.invalid");
        assert_eq!(
            AuthEvent::JwksRefreshSuccess.metric_name(),
            "auth.jwks.refresh.ok"
        );
        assert_eq!(
            AuthEvent::JwksRefreshFailure.metric_name(),
            "auth.jwks.refresh.fail"
        );
    }

    #[test]
    fn test_metric_labels_builder() {
        let labels = AuthMetricLabels::default()
            .with_provider("keycloak")
            .with_issuer("https://kc.example.com")
            .with_kid("key-123");

        assert_eq!(labels.provider, Some("keycloak".to_string()));
        assert_eq!(labels.issuer, Some("https://kc.example.com".to_string()));
        assert_eq!(labels.kid, Some("key-123".to_string()));
        assert_eq!(labels.error_type, None);
    }

    #[test]
    fn test_noop_metrics() {
        let metrics = NoOpMetrics;
        let labels = AuthMetricLabels::default();

        // Should not panic
        metrics.record_event(AuthEvent::JwtValid, &labels);
        metrics.record_duration(100, &labels);
    }

    #[test]
    fn test_logging_metrics() {
        let metrics = LoggingMetrics;
        let labels = AuthMetricLabels::default()
            .with_provider("test")
            .with_issuer("https://test.example.com");

        // Should not panic
        metrics.record_event(AuthEvent::JwtValid, &labels);
        metrics.record_duration(50, &labels);
    }
}
