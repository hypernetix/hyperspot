//! Static licenses plugin for license enforcer.
//!
//! Provides fixed license data for testing and bootstrap purposes.
//! This is a trivial stub implementation that returns fixed data.

pub mod config;
pub mod domain;
pub mod module;

pub use module::StaticLicensesPlugin;
