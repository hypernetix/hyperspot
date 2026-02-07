#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![warn(warnings)]

//! HTTP client infrastructure for `ModKit`
//!
//! This crate provides a hyper-based HTTP client with:
//! - Automatic TLS via rustls (HTTPS only by default)
//! - Connection pooling
//! - Configurable timeouts
//! - Automatic retries with exponential backoff
//! - User-Agent header injection
//! - Concurrency limiting
//! - **Transparent response decompression** (gzip, brotli, deflate)
//! - Optional OpenTelemetry tracing (feature-gated)
//!
//! # Transparent Decompression
//!
//! The client automatically:
//! - Sends `Accept-Encoding: gzip, br, deflate` header on all requests
//! - Decompresses response bodies based on `Content-Encoding` header
//! - Applies body size limits to **decompressed** bytes (protecting against zip bombs)
//!
//! No configuration is required; decompression is always enabled.
//!
//! # Example
//!
//! ```ignore
//! use modkit_http::{HttpClient, HttpClientBuilder};
//! use std::time::Duration;
//!
//! let client = HttpClient::builder()
//!     .timeout(Duration::from_secs(10))
//!     .user_agent("my-app/1.0")
//!     .build()?;
//!
//! // reqwest-like API: response has body-reading methods
//! // Compressed responses are automatically decompressed
//! let data: MyData = client
//!     .get("https://example.com/api")
//!     .send()
//!     .await?
//!     .json()
//!     .await?;
//! ```

mod builder;
mod client;
mod config;
mod error;
mod layers;
pub mod otel;
mod request;
mod response;
pub mod security;
mod tls;

pub use builder::HttpClientBuilder;
pub use client::HttpClient;
pub use config::{
    DEFAULT_USER_AGENT, ExponentialBackoff, HttpClientConfig, IDEMPOTENCY_KEY_HEADER,
    RateLimitConfig, RedirectConfig, RetryConfig, RetryTrigger, TlsRootConfig, TransportSecurity,
    is_idempotent_method,
};
pub use error::{HttpError, InvalidUriKind};
pub use layers::{
    OtelLayer, OtelService, RETRY_ATTEMPT_HEADER, RetryLayer, RetryService, SecureRedirectPolicy,
    UserAgentLayer, UserAgentService,
};
pub use request::RequestBuilder;
pub use response::{HttpResponse, LimitedBody, ResponseBody};
