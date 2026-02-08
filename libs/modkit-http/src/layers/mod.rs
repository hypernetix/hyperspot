//! Tower layers for HTTP client middleware
//!
//! This module provides Tower service layers that can be composed to build
//! the HTTP client middleware stack.
//!
//! ## Available Layers
//!
//! - [`UserAgentLayer`] - Adds User-Agent header to all requests
//! - [`RetryLayer`] - Implements retry with exponential backoff and jitter
//! - [`OtelLayer`] - Adds OpenTelemetry tracing spans to outbound requests
//! - [`SecureRedirectPolicy`] - Security-hardened redirect policy

mod otel;
mod redirect;
mod retry;
mod user_agent;

pub use otel::{OtelLayer, OtelService};
pub use redirect::SecureRedirectPolicy;
pub use retry::{RETRY_ATTEMPT_HEADER, RetryLayer, RetryService};
pub use user_agent::{UserAgentLayer, UserAgentService};
