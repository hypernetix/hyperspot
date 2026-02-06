//! Single-Tenant Resolver Plugin
//!
//! Zero-configuration plugin for single-tenant deployments.
//! Always returns the tenant from the security context as the only accessible tenant.
//!
//! ## Behavior
//!
//! - `get_tenant`: Returns tenant info (name: "Default") only if ID matches security context
//! - `can_access`: Always returns `false` (cross-tenant access not allowed; self-access handled by gateway)
//! - `get_accessible_tenants`: Returns empty list (gateway adds self-tenant automatically)
//!
//! ## Configuration
//!
//! No configuration required. The plugin registers itself automatically with:
//! - Vendor: `hyperspot`
//! - Priority: `1000` (lower than static plugin, so static wins when both are enabled)

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod domain;
pub mod module;

pub use module::SingleTenantTrPlugin;
