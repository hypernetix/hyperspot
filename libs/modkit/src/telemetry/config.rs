//! OpenTelemetry tracing configuration types
//!
//! These types define the configuration structure for OpenTelemetry distributed tracing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tracing configuration for OpenTelemetry distributed tracing
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TracingConfig {
    pub enabled: bool,
    pub service_name: Option<String>,
    pub exporter: Option<Exporter>,
    pub sampler: Option<Sampler>,
    pub propagation: Option<Propagation>,
    pub resource: Option<HashMap<String, String>>,
    pub http: Option<HttpOpts>,
    pub logs_correlation: Option<LogsCorrelation>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Exporter {
    pub kind: Option<String>, // "otlp_grpc" | "otlp_http"
    pub endpoint: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Sampler {
    pub strategy: Option<String>, // "parentbased_always_on" | "parentbased_ratio" | "always_on" | "always_off"
    pub ratio: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Propagation {
    pub w3c_trace_context: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpOpts {
    pub inject_request_id_header: Option<String>,
    pub record_headers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogsCorrelation {
    pub inject_trace_ids_into_logs: Option<bool>,
}
