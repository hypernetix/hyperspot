#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

// === PUBLIC CONTRACT ===
pub use tenant_resolver_sdk as contract;

// === MODULE DEFINITION ===
pub mod module;
pub use module::TenantResolverGateway;

// === INTERNAL ===
#[doc(hidden)]
pub mod api;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod domain;
