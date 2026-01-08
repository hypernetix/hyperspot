//! OAGW domain models.
//!
//! These are transport-agnostic models used across the SDK.
//! Note: NO serde derives here - these are pure domain models.

use bytes::Bytes;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::pin::Pin;
use uuid::Uuid;

use crate::retry::RetryIntent;

/// HTTP method for outbound requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl HttpMethod {
    /// Convert to uppercase string representation.
    #[must_use]
    #[allow(clippy::trivially_copy_pass_by_ref)] // Consistent API with other enums
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Request to invoke an outbound API.
#[derive(Debug, Clone)]
pub struct OagwInvokeRequest {
    /// Optional link ID to use (if not provided, OAGW selects based on route).
    pub link_id: Option<Uuid>,
    /// Route ID identifying the downstream API configuration.
    pub route_id: Uuid,
    /// HTTP method for the request.
    pub method: HttpMethod,
    /// Path to append to the route's base URL.
    pub path: String,
    /// Optional query parameters.
    pub query: Option<HashMap<String, String>>,
    /// Optional custom headers.
    pub headers: Option<HashMap<String, String>>,
    /// Optional request body.
    pub body: Option<Bytes>,
    /// Optional timeout override in milliseconds.
    pub timeout_ms: Option<u64>,
    /// Retry intent (default: no retry).
    pub retry_intent: RetryIntent,
}

impl Default for OagwInvokeRequest {
    fn default() -> Self {
        Self {
            link_id: None,
            route_id: Uuid::nil(),
            method: HttpMethod::Get,
            path: String::new(),
            query: None,
            headers: None,
            body: None,
            timeout_ms: None,
            retry_intent: RetryIntent::default(),
        }
    }
}

/// Response from an outbound API invocation.
#[derive(Debug, Clone)]
pub struct OagwInvokeResponse {
    /// HTTP status code from downstream.
    pub status_code: u16,
    /// Response headers.
    pub headers: HashMap<String, String>,
    /// Response body.
    pub body: Bytes,
    /// Total duration in milliseconds.
    pub duration_ms: u64,
    /// Link ID that was used for the invocation.
    pub link_id: Uuid,
    /// Retry-After hint propagated from downstream (if any).
    pub retry_after_sec: Option<u64>,
    /// Which attempt succeeded (1-based).
    pub attempt_number: u32,
}

/// A chunk from a streaming response.
#[derive(Debug, Clone)]
pub struct OagwStreamChunk {
    /// Raw data bytes.
    pub data: Bytes,
    /// SSE event type (if applicable).
    pub event_type: Option<String>,
    /// SSE event ID for resume (if applicable).
    pub event_id: Option<String>,
}

/// Reason for stream abort.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamAbortReason {
    /// Network-level failure.
    Network,
    /// Protocol-level error.
    Protocol,
    /// Authentication failure.
    Auth,
    /// Timeout exceeded.
    Timeout,
}

impl std::fmt::Display for StreamAbortReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network => write!(f, "network"),
            Self::Protocol => write!(f, "protocol"),
            Self::Auth => write!(f, "auth"),
            Self::Timeout => write!(f, "timeout"),
        }
    }
}

/// Stream abort error with metadata for potential resume.
#[derive(Debug, Clone)]
pub struct OagwStreamAbort {
    /// GTS error ID.
    pub gts_id: String,
    /// Bytes received before failure.
    pub bytes_received: u64,
    /// Reason for abort.
    pub abort_reason: StreamAbortReason,
    /// Whether stream can potentially be resumed.
    pub resumable: bool,
    /// Resume hint (e.g., SSE Last-Event-ID).
    pub resume_hint: Option<String>,
    /// Additional detail message.
    pub detail: Option<String>,
}

/// Streaming response type.
pub type OagwResponseStream =
    Pin<Box<dyn futures::Stream<Item = Result<OagwStreamChunk, OagwStreamAbort>> + Send + 'static>>;

/// Outbound API route configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    /// Unique route identifier.
    pub id: Uuid,
    /// Tenant owning this route.
    pub tenant_id: Uuid,
    /// Base URL for the downstream API.
    pub base_url: String,
    /// Rate limit in requests per minute.
    pub rate_limit_req_per_min: i32,
    /// GTS ID of the auth type required.
    pub auth_type_gts_id: String,
    /// Optional cache TTL in seconds.
    pub cache_ttl_sec: Option<i32>,
    /// Supported protocol GTS IDs.
    pub supported_protocols: Vec<String>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

/// Data for creating a new route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewRoute {
    /// Optional ID (generated if not provided).
    pub id: Option<Uuid>,
    /// Tenant owning this route.
    pub tenant_id: Uuid,
    /// Base URL for the downstream API.
    pub base_url: String,
    /// Rate limit in requests per minute.
    pub rate_limit_req_per_min: Option<i32>,
    /// GTS ID of the auth type required.
    pub auth_type_gts_id: String,
    /// Optional cache TTL in seconds.
    pub cache_ttl_sec: Option<i32>,
    /// Supported protocol GTS IDs.
    pub supported_protocols: Vec<String>,
}

/// Partial update for a route.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoutePatch {
    /// Updated base URL.
    pub base_url: Option<String>,
    /// Updated rate limit.
    pub rate_limit_req_per_min: Option<i32>,
    /// Updated auth type GTS ID.
    pub auth_type_gts_id: Option<String>,
    /// Updated cache TTL. Use `Some(None)` to clear, `Some(Some(v))` to set.
    #[allow(clippy::option_option)] // Intentional: distinguish set-to-null from not-set
    pub cache_ttl_sec: Option<Option<i32>>,
    /// Updated supported protocols.
    pub supported_protocols: Option<Vec<String>>,
}

/// Outbound API link (tenant's connection to a route).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    /// Unique link identifier.
    pub id: Uuid,
    /// Tenant owning this link.
    pub tenant_id: Uuid,
    /// Reference to secret in `cred_store`.
    pub secret_ref: Uuid,
    /// Route this link connects to.
    pub route_id: Uuid,
    /// GTS ID of the secret type.
    pub secret_type_gts_id: String,
    /// Whether this link is enabled.
    pub enabled: bool,
    /// Priority for link selection (lower = higher priority).
    pub priority: i32,
    /// GTS ID of the selection strategy.
    pub strategy_gts_id: String,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

/// Data for creating a new link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewLink {
    /// Optional ID (generated if not provided).
    pub id: Option<Uuid>,
    /// Tenant owning this link.
    pub tenant_id: Uuid,
    /// Reference to secret in `cred_store`.
    pub secret_ref: Uuid,
    /// Route this link connects to.
    pub route_id: Uuid,
    /// GTS ID of the secret type.
    pub secret_type_gts_id: String,
    /// Whether this link is enabled.
    pub enabled: Option<bool>,
    /// Priority for link selection.
    pub priority: Option<i32>,
    /// GTS ID of the selection strategy.
    pub strategy_gts_id: String,
}

/// Partial update for a link.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LinkPatch {
    /// Updated secret reference.
    pub secret_ref: Option<Uuid>,
    /// Updated secret type.
    pub secret_type_gts_id: Option<String>,
    /// Updated enabled state.
    pub enabled: Option<bool>,
    /// Updated priority.
    pub priority: Option<i32>,
    /// Updated strategy.
    pub strategy_gts_id: Option<String>,
}

/// Secret material retrieved from `cred_store`.
#[derive(Debug, Clone)]
#[allow(clippy::struct_field_names)] // Prefix needed for clarity in domain model
pub struct Secret {
    /// Secret ID.
    pub id: Uuid,
    /// GTS ID of the secret type.
    pub secret_type_gts_id: String,
    /// The actual secret value.
    pub value: String,
    /// Optional metadata (e.g., header name for API key).
    pub metadata: Option<HashMap<String, String>>,
}

/// Health status for a route or link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Fully operational.
    Healthy,
    /// Some links are down.
    Degraded,
    /// All links are down.
    Unhealthy,
    /// Circuit breaker is open.
    CircuitOpen,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded => write!(f, "degraded"),
            Self::Unhealthy => write!(f, "unhealthy"),
            Self::CircuitOpen => write!(f, "circuit_open"),
        }
    }
}

/// Health check response for a route.
#[derive(Debug, Clone)]
pub struct RouteHealth {
    /// Route ID.
    pub route_id: Uuid,
    /// Overall status.
    pub status: HealthStatus,
    /// Per-link health status.
    pub links: Vec<LinkHealth>,
}

/// Health check response for a link.
#[derive(Debug, Clone)]
pub struct LinkHealth {
    /// Link ID.
    pub link_id: Uuid,
    /// Link status.
    pub status: HealthStatus,
}
