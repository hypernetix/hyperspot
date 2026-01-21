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
//! // Check access
//! let can_access = resolver.can_access(&ctx, target_tenant_id).await?;
//!
//! // Get all accessible tenants
//! let accessible = resolver.get_accessible_tenants(&ctx, query).await?;
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
pub use models::{AccessOptions, TenantFilter, TenantId, TenantInfo, TenantStatus};
pub use plugin_api::TenantResolverPluginClient;
