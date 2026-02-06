//! No-cache plugin for license enforcer.
//!
//! Provides a no-op cache implementation that always returns cache miss.
//! This is a trivial stub for bootstrap/testing purposes.

pub mod config;
pub mod domain;
pub mod module;

pub use module::NoCachePlugin;
