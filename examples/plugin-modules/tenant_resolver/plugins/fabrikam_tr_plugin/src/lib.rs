//! Fabrikam tenant resolver plugin.
//!
//! This plugin provides a Fabrikam-specific implementation of the tenant resolver API.

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

// === MODULE DEFINITION ===
pub mod module;
pub use module::FabrikamTrPlugin;

// === INTERNAL MODULES ===
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod domain;
