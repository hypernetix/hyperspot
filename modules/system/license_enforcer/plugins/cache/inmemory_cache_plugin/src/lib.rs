//! In-memory cache plugin for license enforcer.
//!
//! Provides a TTL-based in-memory cache implementation for tenant-scoped
//! enabled global feature sets.

pub mod config;
pub mod domain;
pub mod module;

#[cfg(test)]
mod config_tests;

pub use module::InMemoryCachePlugin;
