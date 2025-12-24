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
//! - `ThrPluginSpec` - Plugin instance schema (`gts.x.core.plugins.thr_plugin.v1~`)
//! - `TenantSpec` - Tenant entity schema (`gts.x.core.tenants.tenant.v1~`)

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
pub use gts::{TenantSpec, ThrPluginSpec};

// Client models
pub use models::{
    AccessOptions, GetParentsResponse, Tenant, TenantFilter, TenantResult, TenantStatus,
};

// Pagination primitives (re-exported for convenience)
pub use modkit_odata::{ODataQuery, Page, PageInfo};
