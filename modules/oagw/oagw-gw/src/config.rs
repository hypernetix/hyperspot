//! OAGW module configuration.

use serde::Deserialize;

/// OAGW module configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OagwConfig {
    // === Token Cache Settings (v3+) ===
    /// Maximum capacity of the token cache.
    pub token_cache_max_capacity: usize,
    /// Maximum TTL for cached tokens in seconds.
    pub token_cache_max_ttl_sec: u64,
    /// Safety margin to subtract from token expiry in seconds.
    pub token_cache_safety_margin_sec: u64,

    // === Response Cache Settings (v5+) ===
    /// Maximum capacity of the response cache.
    pub response_cache_max_capacity: usize,

    // === Request Limits ===
    /// Maximum in-flight bytes per invocation.
    pub max_inflight_bytes_per_invocation: usize,
    /// Maximum size of a streaming chunk in bytes.
    pub max_stream_chunk_size_bytes: usize,

    // === Timeout Defaults ===
    /// Default connection timeout in milliseconds.
    pub default_connection_timeout_ms: u64,
    /// Default request timeout in milliseconds.
    pub default_request_timeout_ms: u64,
    /// Default idle timeout in milliseconds.
    pub default_idle_timeout_ms: u64,

    // === Rate Limiting (v2+) ===
    /// Default rate limit per minute.
    pub default_rate_limit_per_min: u32,

    // === Circuit Breaker (v3+) ===
    /// Number of consecutive failures to open circuit.
    pub circuit_breaker_threshold: u32,
    /// Timeout in seconds before half-open state.
    pub circuit_breaker_timeout_sec: u32,
    /// Number of successes in half-open to close circuit.
    pub circuit_breaker_success_threshold: u32,

    // === Audit Logging (v2+) ===
    /// Audit guarantee level.
    pub audit_guarantee: AuditGuarantee,
    /// Maximum latency for audit logging in milliseconds.
    pub max_audit_latency_ms: u64,
}

impl Default for OagwConfig {
    fn default() -> Self {
        Self {
            // Token cache defaults
            token_cache_max_capacity: 10_000,
            token_cache_max_ttl_sec: 3600,
            token_cache_safety_margin_sec: 60,

            // Response cache defaults
            response_cache_max_capacity: 1_000,

            // Request limits
            max_inflight_bytes_per_invocation: 8_388_608, // 8 MiB
            max_stream_chunk_size_bytes: 65_536,          // 64 KiB

            // Timeout defaults
            default_connection_timeout_ms: 5_000,
            default_request_timeout_ms: 30_000,
            default_idle_timeout_ms: 60_000,

            // Rate limiting
            default_rate_limit_per_min: 1_000,

            // Circuit breaker
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_sec: 30,
            circuit_breaker_success_threshold: 2,

            // Audit logging
            audit_guarantee: AuditGuarantee::BestEffort,
            max_audit_latency_ms: 50,
        }
    }
}

/// Audit guarantee level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditGuarantee {
    /// Best-effort audit logging (non-blocking).
    #[default]
    BestEffort,
    /// Guaranteed audit logging (blocks until written).
    Guaranteed,
    /// Fail-closed audit logging (fail request if audit fails).
    FailClosed,
}
