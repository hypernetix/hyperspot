//! Tenant Resolver SDK
//!
//! This crate provides the public API for the `tenant-resolver` module:
//!
//! - [`TenantResolverClient`] - Public API trait for consumers
//! - [`TenantResolverPluginClient`] - Plugin API trait for implementations
//! - [`TenantInfo`], [`TenantStatus`] - Domain models
//! - [`TenantResolverError`] - Error types
//! - [`TenantResolverPluginSpecV1`] - GTS schema for plugin discovery
//!
//! ## Usage
//!
//! Consumers obtain the client from `ClientHub`:
//!
//! ```ignore
//! use tenant_resolver_sdk::{
//!     TenantResolverClient, GetAncestorsOptions, GetDescendantsOptions,
//!     GetTenantsOptions, IsAncestorOptions,
//! };
//!
//! // Get the client from ClientHub
//! let resolver = hub.get::<dyn TenantResolverClient>()?;
//!
//! // Get tenant info
//! let tenant = resolver.get_tenant(&ctx, tenant_id).await?;
//!
//! // Get multiple tenants
//! let tenants = resolver.get_tenants(&ctx, &[id1, id2], &GetTenantsOptions::default()).await?;
//!
//! // Get ancestors
//! let response = resolver.get_ancestors(&ctx, tenant_id, &GetAncestorsOptions::default()).await?;
//!
//! // Get descendants
//! let descendants = resolver.get_descendants(&ctx, tenant_id, &GetDescendantsOptions::default()).await?;
//!
//! // Check ancestry
//! let is_anc = resolver.is_ancestor(&ctx, parent_id, child_id, &IsAncestorOptions::default()).await?;
//! ```

pub mod api;
pub mod error;
pub mod gts;
pub mod models;
pub mod plugin_api;

// Re-export main types at crate root
pub use api::TenantResolverClient;
pub use error::TenantResolverError;
pub use gts::TenantResolverPluginSpecV1;
pub use models::{
    BarrierMode, GetAncestorsOptions, GetAncestorsResponse, GetDescendantsOptions,
    GetDescendantsResponse, GetTenantsOptions, HasStatus, IsAncestorOptions, TenantId, TenantInfo,
    TenantRef, TenantStatus, matches_status,
};
pub use plugin_api::TenantResolverPluginClient;
