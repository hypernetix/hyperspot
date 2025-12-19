//! Types Registry SDK
//!
//! This crate provides the public API for the `types-registry` module:
//! - `TypesRegistryApi` trait for inter-module communication
//! - `GtsEntity` model representing registered GTS entities
//! - `ListQuery` for filtering entity listings
//! - `TypesRegistryError` for error handling
//!
//! ## Usage
//!
//! Consumers obtain the client from `ClientHub`:
//! ```ignore
//! use types_registry_sdk::TypesRegistryApi;
//!
//! // Get the client from ClientHub
//! let client = hub.get::<dyn TypesRegistryApi>()?;
//!
//! // Register entities
//! let entities = client.register(&ctx, json_values).await?;
//!
//! // List entities with filtering
//! let query = ListQuery::default().with_vendor("acme");
//! let entities = client.list(&ctx, query).await?;
//!
//! // Get a single entity
//! let entity = client.get(&ctx, "gts.acme.core.events.user_created.v1~").await?;
//! ```

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod api;
pub mod error;
pub mod models;

// Re-export main types at crate root for convenience
pub use api::TypesRegistryApi;
pub use error::TypesRegistryError;
pub use models::{GtsEntity, GtsEntityKind, GtsIdSegment, ListQuery};
