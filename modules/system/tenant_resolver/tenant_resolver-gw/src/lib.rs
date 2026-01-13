//! Tenant Resolver Gateway Module
//!
//! This module discovers tenant resolver plugins via types-registry
//! and routes API calls to the selected plugin based on vendor configuration.
//!
//! The gateway provides the `TenantResolverGatewayClient` trait registered
//! in `ClientHub` for consumption by other modules.

pub mod config;
pub mod domain;
pub mod module;
