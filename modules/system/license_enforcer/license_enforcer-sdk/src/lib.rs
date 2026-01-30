//! License Enforcer SDK
//!
//! This crate provides the public API for the `license_enforcer` module:
//!
//! - [`LicenseEnforcerGatewayClient`] - Public API trait for consumers
//! - [`PlatformPluginClient`] - Platform request plugin API trait for implementations
//! - [`CachePluginClient`] - Cache plugin API trait for implementations
//! - Domain models for license enforcement
//! - [`LicenseEnforcerError`] - Error types
//! - GTS schemas for plugin discovery
//!
//! ## Usage
//!
//! Consumers obtain the client from `ClientHub`:
//!
//! ```ignore
//! use license_enforcer_sdk::LicenseEnforcerGatewayClient;
//!
//! // Get the client from ClientHub
//! let enforcer = hub.get::<dyn LicenseEnforcerGatewayClient>()?;
//!
//! // Check license access
//! let can_access = enforcer.check_access(&ctx, feature_id).await?;
//! ```
#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod api;
pub mod error;
pub mod gts_cache;
pub mod gts_platform;
pub mod models;
pub mod plugin_cache;
pub mod plugin_platform;

// Re-export main types at crate root
pub use api::LicenseEnforcerGatewayClient;
pub use error::LicenseEnforcerError;
pub use gts_cache::LicenseCachePluginSpecV1;
pub use gts_platform::LicensePlatformPluginSpecV1;
pub use models::{LicenseCheckRequest, LicenseCheckResponse, LicenseFeature, LicenseStatus};
pub use plugin_cache::CachePluginClient;
pub use plugin_platform::PlatformPluginClient;
