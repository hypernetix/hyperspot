//! License Enforcer Gateway Module
//!
//! Gateway module that discovers and routes to platform and cache plugins
//! for license enforcement.
//!
//! ## Architecture
//!
//! - **Gateway**: Exposes public API, discovers plugins via GTS, routes requests
//! - **Platform Plugins**: Provide license data from external systems
//! - **Cache Plugins**: Provide caching for license check results
//!
//! ## Plugin Discovery
//!
//! Plugins are discovered via types-registry using GTS schema IDs:
//! - Platform: `gts.x.core.modkit.plugin.v1~x.core.license_resolver.plugin.v1~`
//! - Cache: `gts.x.core.modkit.plugin.v1~x.core.license_cache.plugin.v1~`

// Re-export SDK types
pub use license_enforcer_sdk::*;

#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod domain;
pub mod module;

// Re-export module for registration
pub use module::LicenseEnforcerGateway;
