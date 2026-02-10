//! Single-Tenant Resolver Plugin
//!
//! Zero-configuration plugin for single-tenant deployments.
//! Implements flat (single-tenant) semantics where only the security context's tenant exists.
//!
//! ## Behavior
//!
//! - `get_tenant`: Returns tenant info (name: "Default") only if ID matches security context
//! - `get_tenants`: Returns tenant info only for IDs matching the security context
//! - `get_ancestors`: Returns empty ancestors (tenant is root, no parent)
//! - `get_descendants`: Returns empty list (no children in flat model)
//! - `is_ancestor`: Returns `false` for self-check; errors for any other IDs (only one tenant exists)
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
