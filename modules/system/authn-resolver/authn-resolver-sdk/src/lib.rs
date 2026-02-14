//! `AuthN` Resolver SDK
//!
//! This crate provides the public API for the `authn_resolver` module:
//!
//! - [`AuthNResolverClient`] - Public API trait for consumers
//! - [`AuthNResolverPluginClient`] - Plugin API trait for implementations
//! - [`AuthenticationResult`] - Authentication result model
//! - [`AuthNResolverError`] - Error types
//! - [`AuthNResolverPluginSpecV1`] - GTS schema for plugin discovery
//!
//! ## Usage
//!
//! Consumers obtain the client from `ClientHub`:
//!
//! ```ignore
//! use authn_resolver_sdk::AuthNResolverClient;
//!
//! // Get the client from ClientHub
//! let authn = hub.get::<dyn AuthNResolverClient>()?;
//!
//! // Authenticate a bearer token
//! let result = authn.authenticate("Bearer xyz...").await?;
//! let security_context = result.security_context;
//! ```

pub mod api;
pub mod error;
pub mod gts;
pub mod models;
pub mod plugin_api;

// Re-export main types at crate root
pub use api::AuthNResolverClient;
pub use error::AuthNResolverError;
pub use gts::AuthNResolverPluginSpecV1;
pub use models::AuthenticationResult;
pub use plugin_api::AuthNResolverPluginClient;
