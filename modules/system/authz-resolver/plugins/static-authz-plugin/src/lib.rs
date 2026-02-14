#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! Static `AuthZ` Resolver Plugin
//!
//! This plugin provides a static allow-all authorization policy for development and testing.
//!
//! ## Mode: `allow_all` (default)
//!
//! - `require_constraints=false` (CREATE) → `decision: true`, no constraints
//! - `require_constraints=true` (LIST/GET/UPDATE/DELETE) → `decision: true` with
//!   `in` predicate on `owner_tenant_id` using the context tenant ID.
//!
//! ## Configuration
//!
//! ```yaml
//! modules:
//!   static_authz_plugin:
//!     config:
//!       vendor: "hyperspot"
//!       priority: 100
//!       mode: allow_all
//! ```

pub mod config;
pub mod domain;
pub mod module;

pub use module::StaticAuthZPlugin;
