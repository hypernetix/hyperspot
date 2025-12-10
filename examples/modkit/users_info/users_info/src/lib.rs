//! Users Info Module
//!
//! This module provides user management functionality with REST API,
//! database storage, and inter-module communication via ClientHub.
//!
//! ## Public API
//!
//! The public API is defined in the `user_info-sdk` crate and re-exported here:
//! - `UsersInfoApi` - trait for inter-module communication
//! - `User`, `NewUser`, `UserPatch`, `UpdateUserRequest` - data models
//! - `UsersInfoError` - error types
//!
//! Other modules should use `hub.get::<dyn UsersInfoApi>()?` to obtain the client.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
// === PUBLIC API (from SDK) ===
pub use user_info_sdk::{
    NewUser, UpdateUserRequest, User, UserPatch, UsersInfoApi, UsersInfoError,
};

// === ERROR CATALOG ===
// Generated error catalog from gts/errors.json
pub mod errors;

// === MODULE DEFINITION ===
// ModKit needs access to the module struct for instantiation
pub mod module;
pub use module::UsersInfo;

// === LOCAL CLIENT ===
// Local client adapter that implements UsersInfoApi
pub mod local_client;

// === INTERNAL MODULES ===
// WARNING: These modules are internal implementation details!
// They are exposed only for comprehensive testing and should NOT be used by external consumers.
// Only use the SDK types for stable public APIs.
#[doc(hidden)]
pub mod api;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod domain;
#[doc(hidden)]
pub mod infra;
