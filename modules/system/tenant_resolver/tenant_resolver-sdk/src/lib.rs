//! Tenant Resolver SDK
//!
//! This crate provides the public API for the `tenant_resolver` module:
//!
//! - [`TenantResolverGatewayClient`] - Public API trait for consumers
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
//! use tenant_resolver_sdk::TenantResolverGatewayClient;
//!
//! // Get the client from ClientHub
//! let resolver = hub.get::<dyn TenantResolverGatewayClient>()?;
//!
//! // Get tenant info
//! let tenant = resolver.get_tenant(&ctx, tenant_id).await?;
//!
//! // Get ancestors
//! let response = resolver.get_ancestors(&ctx, tenant_id, None).await?;
//!
//! // Get descendants
//! let descendants = resolver.get_descendants(&ctx, tenant_id, None, None, None).await?;
//!
//! // Check ancestry
//! let is_anc = resolver.is_ancestor(&ctx, parent_id, child_id, None).await?;
//! ```

pub mod api;
pub mod error;
pub mod gts;
pub mod models;
pub mod plugin_api;

// Re-export main types at crate root
pub use api::TenantResolverGatewayClient;
pub use error::TenantResolverError;
pub use gts::TenantResolverPluginSpecV1;
pub use models::{
    BarrierMode, GetAncestorsResponse, GetDescendantsResponse, HierarchyOptions, TenantFilter,
    TenantId, TenantInfo, TenantRef, TenantStatus,
};
pub use plugin_api::TenantResolverPluginClient;
