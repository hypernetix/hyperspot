//! Tenant resolver SDK (example).
//!
//! This crate defines the public API contract for the `tenant_resolver` gateway
//! and its plugin implementations.
//!
//! ## API Traits
//!
//! - `TenantResolverClient` - Public API exposed by the gateway to other modules
//! - `ThrPluginApi` - Internal API implemented by plugins
//!
//! ## GTS Types
//!
//! - `TenantResolverPluginSpecV1` - Plugin instance schema (`gts.x.core.modkit.plugin.v1~x.core.tenant_resolver.plugin.v1~`)
//! - `TenantSpecV1` - Tenant entity schema (`gts.x.core.tenants.tenant.v1~`)

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod api;
pub mod error;
pub mod gts;
pub mod models;

// API traits
pub use api::{TenantResolverClient, ThrPluginApi};

// Error types
pub use error::TenantResolverError;

// GTS schema types
pub use gts::{TenantResolverPluginSpecV1, TenantSpecV1};

// Client models
pub use models::{
    AccessOptions, GetParentsResponse, Tenant, TenantFilter, TenantResult, TenantStatus,
};

// GTS types
pub use ::gts::GtsSchemaId;

// Pagination primitives (re-exported for convenience)
pub use modkit_odata::{ODataQuery, Page, PageInfo};
