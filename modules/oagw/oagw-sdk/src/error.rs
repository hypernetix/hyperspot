//! OAGW error types.
//!
//! Transport-agnostic error definitions for the OAGW module.

use thiserror::Error;
use uuid::Uuid;

/// Error type for OAGW operations.
#[derive(Error, Debug, Clone)]
pub enum OagwError {
    /// Route not found.
    #[error("route not found: {id}")]
    RouteNotFound { id: Uuid },

    /// Link not found.
    #[error("link not found: {id}")]
    LinkNotFound { id: Uuid },

    /// Link is unavailable (disabled or all filtered out).
    #[error("link unavailable for route {route_id}")]
    LinkUnavailable { route_id: Uuid },

    /// Circuit breaker is open.
    #[error("circuit breaker open for link {link_id}")]
    CircuitBreakerOpen {
        link_id: Uuid,
        retry_after_sec: Option<u64>,
    },

    /// Connection timeout.
    #[error("connection timeout")]
    ConnectionTimeout,

    /// Request timeout.
    #[error("request timeout")]
    RequestTimeout,

    /// Idle timeout.
    #[error("idle timeout")]
    IdleTimeout,

    /// Rate limit exceeded.
    #[error("rate limit exceeded")]
    RateLimitExceeded { retry_after_sec: Option<u64> },

    /// Payload too large.
    #[error("payload too large: {size} bytes exceeds limit of {limit} bytes")]
    PayloadTooLarge { size: u64, limit: u64 },

    /// Protocol error.
    #[error("protocol error: {message}")]
    ProtocolError { message: String },

    /// Authentication failed.
    #[error("authentication failed: {message}")]
    AuthenticationFailed { message: String },

    /// Secret not found in `cred_store`.
    #[error("secret not found: {secret_ref}")]
    SecretNotFound { secret_ref: Uuid },

    /// Plugin not found for required capabilities.
    #[error("no plugin found for protocol {protocol} and auth type {auth_type}")]
    PluginNotFound { protocol: String, auth_type: String },

    /// Downstream API error.
    #[error("downstream error: status {status_code}")]
    DownstreamError {
        status_code: u16,
        retry_after_sec: Option<u64>,
    },

    /// Stream was aborted.
    #[error("stream aborted: {reason}")]
    StreamAborted {
        reason: String,
        bytes_received: u64,
        resumable: bool,
    },

    /// Validation error.
    #[error("validation error: {message}")]
    ValidationError { message: String },

    /// Internal error.
    #[error("internal error: {message}")]
    Internal { message: String },

    /// Authorization error.
    #[error("forbidden: {message}")]
    Forbidden { message: String },

    /// Database error.
    #[error("database error")]
    Database,
}

impl OagwError {
    /// Create a route not found error.
    #[must_use]
    pub fn route_not_found(id: Uuid) -> Self {
        Self::RouteNotFound { id }
    }

    /// Create a link not found error.
    #[must_use]
    pub fn link_not_found(id: Uuid) -> Self {
        Self::LinkNotFound { id }
    }

    /// Create a link unavailable error.
    #[must_use]
    pub fn link_unavailable(route_id: Uuid) -> Self {
        Self::LinkUnavailable { route_id }
    }

    /// Create a circuit breaker open error.
    #[must_use]
    pub fn circuit_breaker_open(link_id: Uuid, retry_after_sec: Option<u64>) -> Self {
        Self::CircuitBreakerOpen {
            link_id,
            retry_after_sec,
        }
    }

    /// Create a rate limit exceeded error.
    #[must_use]
    pub fn rate_limit_exceeded(retry_after_sec: Option<u64>) -> Self {
        Self::RateLimitExceeded { retry_after_sec }
    }

    /// Create a payload too large error.
    #[must_use]
    pub fn payload_too_large(size: u64, limit: u64) -> Self {
        Self::PayloadTooLarge { size, limit }
    }

    /// Create a protocol error.
    #[must_use]
    pub fn protocol_error(message: impl Into<String>) -> Self {
        Self::ProtocolError {
            message: message.into(),
        }
    }

    /// Create an authentication failed error.
    #[must_use]
    pub fn authentication_failed(message: impl Into<String>) -> Self {
        Self::AuthenticationFailed {
            message: message.into(),
        }
    }

    /// Create a secret not found error.
    #[must_use]
    pub fn secret_not_found(secret_ref: Uuid) -> Self {
        Self::SecretNotFound { secret_ref }
    }

    /// Create a plugin not found error.
    #[must_use]
    pub fn plugin_not_found(protocol: impl Into<String>, auth_type: impl Into<String>) -> Self {
        Self::PluginNotFound {
            protocol: protocol.into(),
            auth_type: auth_type.into(),
        }
    }

    /// Create a downstream error.
    #[must_use]
    pub fn downstream_error(status_code: u16, retry_after_sec: Option<u64>) -> Self {
        Self::DownstreamError {
            status_code,
            retry_after_sec,
        }
    }

    /// Create a validation error.
    #[must_use]
    pub fn validation(message: impl Into<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
        }
    }

    /// Create an internal error.
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Create a forbidden error.
    #[must_use]
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden {
            message: message.into(),
        }
    }

    /// Check if this error is retriable.
    #[must_use]
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            Self::ConnectionTimeout
                | Self::RequestTimeout
                | Self::IdleTimeout
                | Self::RateLimitExceeded { .. }
                | Self::CircuitBreakerOpen { .. }
                | Self::LinkUnavailable { .. }
        )
    }

    /// Get the retry-after hint if available.
    #[must_use]
    pub fn retry_after_sec(&self) -> Option<u64> {
        match self {
            Self::RateLimitExceeded { retry_after_sec }
            | Self::CircuitBreakerOpen {
                retry_after_sec, ..
            }
            | Self::DownstreamError {
                retry_after_sec, ..
            } => *retry_after_sec,
            _ => None,
        }
    }

    /// Get the GTS error instance ID for this error.
    #[must_use]
    pub fn gts_id(&self) -> &'static str {
        match self {
            Self::RouteNotFound { .. } => "gts.x.core.errors.err.v1~x.oagw.route.not_found.v1",
            Self::LinkNotFound { .. } => "gts.x.core.errors.err.v1~x.oagw.link.not_found.v1",
            Self::LinkUnavailable { .. } => "gts.x.core.errors.err.v1~x.oagw.link.unavailable.v1",
            Self::CircuitBreakerOpen { .. } => {
                "gts.x.core.errors.err.v1~x.oagw.circuit_breaker.open.v1"
            }
            Self::ConnectionTimeout => "gts.x.core.errors.err.v1~x.oagw.timeout.connection.v1",
            Self::RequestTimeout => "gts.x.core.errors.err.v1~x.oagw.timeout.request.v1",
            Self::IdleTimeout => "gts.x.core.errors.err.v1~x.oagw.timeout.idle.v1",
            Self::RateLimitExceeded { .. } => {
                "gts.x.core.errors.err.v1~x.oagw.rate_limit.exceeded.v1"
            }
            Self::PayloadTooLarge { .. } => "gts.x.core.errors.err.v1~x.oagw.payload.too_large.v1",
            Self::ProtocolError { .. } => "gts.x.core.errors.err.v1~x.oagw.protocol.error.v1",
            Self::AuthenticationFailed { .. } => "gts.x.core.errors.err.v1~x.oagw.auth.failed.v1",
            Self::SecretNotFound { .. } => "gts.x.core.errors.err.v1~x.oagw.secret.not_found.v1",
            Self::PluginNotFound { .. } => "gts.x.core.errors.err.v1~x.oagw.plugin.not_found.v1",
            Self::DownstreamError { .. } => "gts.x.core.errors.err.v1~x.oagw.downstream.error.v1",
            Self::StreamAborted { .. } => "gts.x.core.errors.err.v1~x.oagw.stream.aborted.v1",
            Self::ValidationError { .. } => "gts.x.core.errors.err.v1~x.oagw.validation.error.v1",
            Self::Internal { .. } | Self::Database => {
                "gts.x.core.errors.err.v1~x.oagw.internal.error.v1"
            }
            Self::Forbidden { .. } => "gts.x.core.errors.err.v1~x.oagw.access.forbidden.v1",
        }
    }

    /// Get the HTTP status code for this error.
    #[must_use]
    pub fn status_code(&self) -> u16 {
        match self {
            Self::RouteNotFound { .. } | Self::LinkNotFound { .. } => 404,
            Self::LinkUnavailable { .. }
            | Self::CircuitBreakerOpen { .. }
            | Self::PluginNotFound { .. } => 503,
            Self::ConnectionTimeout | Self::RequestTimeout | Self::IdleTimeout => 504,
            Self::RateLimitExceeded { .. } => 429,
            Self::PayloadTooLarge { .. } => 413,
            Self::ProtocolError { .. }
            | Self::DownstreamError { .. }
            | Self::StreamAborted { .. } => 502,
            Self::AuthenticationFailed { .. } => 401,
            Self::SecretNotFound { .. } | Self::Internal { .. } | Self::Database => 500,
            Self::ValidationError { .. } => 400,
            Self::Forbidden { .. } => 403,
        }
    }
}
