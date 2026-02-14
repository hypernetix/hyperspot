#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! `AuthZ` Resolver SDK
//!
//! This crate provides the public API for the `authz_resolver` module:
//!
//! - [`AuthZResolverClient`] - Public API trait for consumers
//! - [`AuthZResolverPluginClient`] - Plugin API trait for implementations
//! - [`EvaluationRequest`], [`EvaluationResponse`] - Evaluation models
//! - [`Constraint`], [`Predicate`] - Constraint types
//! - [`AuthZResolverError`] - Error types
//! - [`AuthZResolverPluginSpecV1`] - GTS schema for plugin discovery
//! - [`pep`] - PEP helpers ([`PolicyEnforcer`], [`ResourceType`], compiler)
//!
//! ## Usage
//!
//! ```ignore
//! use authz_resolver_sdk::{
//!     AuthZResolverClient,
//!     pep::{AccessRequest, PolicyEnforcer, ResourceType},
//! };
//!
//! const USER: ResourceType = ResourceType {
//!     name: "users_info.user",
//!     supported_properties: &["owner_tenant_id", "id"],
//! };
//!
//! // Get the client from ClientHub
//! let authz = hub.get::<dyn AuthZResolverClient>()?;
//!
//! // Create an enforcer (once, during init — serves all resource types)
//! let enforcer = PolicyEnforcer::new(authz);
//!
//! // All CRUD operations return AccessScope (PDP always returns constraints)
//! let scope = enforcer.access_scope(&ctx, &USER, "get", Some(id)).await?;
//!
//! // CREATE — also returns AccessScope with constraints from PDP
//! let scope = enforcer.access_scope_with(
//!     &ctx, &USER, "create", None,
//!     &AccessRequest::new()
//!         .context_tenant_id(target_tenant_id)
//!         .resource_property("owner_tenant_id", target_tenant_id),
//! ).await?;
//! ```

pub mod api;
pub mod constraints;
pub mod error;
pub mod gts;
pub mod models;
pub mod pep;
pub mod plugin_api;

// Re-export main types at crate root
pub use api::AuthZResolverClient;
pub use constraints::{Constraint, EqPredicate, InPredicate, Predicate};
pub use error::AuthZResolverError;
pub use gts::AuthZResolverPluginSpecV1;
pub use models::{
    Action, BarrierMode, Capability, DenyReason, EvaluationRequest, EvaluationRequestContext,
    EvaluationResponse, EvaluationResponseContext, Resource, Subject, TenantContext, TenantMode,
};
pub use pep::{AccessRequest, EnforcerError, IntoPropertyValue, PolicyEnforcer, ResourceType};
pub use plugin_api::AuthZResolverPluginClient;
