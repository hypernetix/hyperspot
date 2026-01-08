//! Static Tenant Resolver Plugin
//!
//! This plugin provides tenant data and access rules from configuration.
//! Useful for testing, development, and simple deployments.
//!
//! ## Configuration
//!
//! ```yaml
//! modules:
//!   static_tr_plugin:
//!     vendor: "hyperspot"
//!     priority: 100
//!     tenants:
//!       - id: "550e8400-e29b-41d4-a716-446655440001"
//!         name: "Tenant A"
//!         status: active
//!     access_rules:
//!       - source: "550e8400-e29b-41d4-a716-446655440001"
//!         target: "550e8400-e29b-41d4-a716-446655440002"
//! ```

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod config;
pub mod domain;
pub mod module;

pub use module::StaticTrPlugin;
