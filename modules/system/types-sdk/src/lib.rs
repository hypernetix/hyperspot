//! Types SDK
//!
//! This crate provides the public API for the `types` module:
//! - `TypesClient` trait for inter-module communication
//! - `TypesError` for error handling
//!
//! ## Purpose
//!
//! The `types` module is responsible for registering core GTS types that are used
//! throughout the framework (e.g., `BaseModkitPluginV1` for plugin systems).
//!
//! ## Usage
//!
//! Consumers obtain the client from `ClientHub`:
//! ```ignore
//! use types_sdk::TypesClient;
//!
//! // Get the client from ClientHub
//! let client = hub.get::<dyn TypesClient>()?;
//!
//! // Check if core types are registered
//! let ready = client.is_ready().await?;
//! ```

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod api;
pub mod error;

// Re-export main types at crate root for convenience
pub use api::TypesClient;
pub use error::TypesError;
