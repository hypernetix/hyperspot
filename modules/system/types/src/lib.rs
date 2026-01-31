//! Core Types Registration Module
//!
//! This system module is responsible for registering core GTS types that are used
//! throughout the framework, particularly for plugin systems.
//!
//! ## Purpose
//!
//! Previously, core types like `BaseModkitPluginV1` were registered directly
//! by the `types_registry` module, creating a circular dependency issue.
//! This module resolves that by:
//!
//! - Acting as the owner of core framework types
//! - Registering these types during its initialization
//! - Depended on by `types_registry` (not the other way around)
//!
//! ## Dependency Chain
//!
//! ```text
//! `types_registry` → `types` → `plugin_modules`
//! ```
//!
//! This ensures core types are available when plugin modules initialize.
//!
//! ## SDK
//!
//! Public API traits, error types, and models are exported from `types-sdk`.

mod domain;
pub mod module;

pub use module::Types;
