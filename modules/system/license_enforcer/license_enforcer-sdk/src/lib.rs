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
//! ```
//! # use license_enforcer_sdk::{LicenseEnforcerGatewayClient, global_features};
//! # use modkit::client_hub::ClientHub;
//! # use modkit_security::SecurityContext;
//! # use std::sync::Arc;
//! # use uuid::Uuid;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let hub = Arc::new(ClientHub::new());
//! # let tenant_id = Uuid::new_v4();
//! # let ctx = SecurityContext::builder().tenant_id(tenant_id).subject_id(Uuid::new_v4()).build();
//! # let feature_id = global_features::to_feature_id(global_features::BASE);
//! // Get the client from ClientHub
//! let enforcer = hub.get::<dyn LicenseEnforcerGatewayClient>()?;
//!
//! // Check license access
//! let is_enabled = enforcer.is_global_feature_enabled(&ctx, tenant_id, &feature_id).await?;
//! # Ok(())
//! # }
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
pub use models::{EnabledGlobalFeatures, LicenseFeatureID, global_features};
pub use plugin_cache::CachePluginClient;
pub use plugin_platform::PlatformPluginClient;
