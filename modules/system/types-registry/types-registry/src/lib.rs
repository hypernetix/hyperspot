//! Types Registry Module Implementation
//!
//! This module provides GTS entity registration, storage, validation, and REST API endpoints.
//! The public API is defined in `types-registry-sdk` and re-exported here.
//!
//! ## Architecture
//!
//! - **Two-phase storage**: Configuration phase (no validation) → Ready phase (full validation)
//! - **gts-rust integration**: Uses the official GTS library for all operations
//! - **`ClientHub` registration**: Other modules access via `hub.get::<dyn TypesRegistryApi>()?`

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

// === PUBLIC API (from SDK) ===
pub use types_registry_sdk::{
    DynGtsEntity, DynRegisterResult, GtsEntity, GtsInstanceEntity, GtsTypeEntity, InstanceObject,
    ListQuery, RegisterResult, RegisterSummary, SegmentMatchScope, TypeSchema, TypesRegistryClient,
    TypesRegistryError,
};

// === MODULE DEFINITION ===
pub mod module;
pub use module::TypesRegistryModule;

// === CONFIGURATION ===
pub mod config;

// === INTERNAL MODULES ===
#[doc(hidden)]
pub mod api;
#[doc(hidden)]
pub mod domain;
#[doc(hidden)]
pub mod infra;
