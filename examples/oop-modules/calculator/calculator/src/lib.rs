//! Calculator Module
//!
//! A trivial example gRPC service that performs addition.
//! This module demonstrates the OoP (out-of-process) module pattern.
//!
//! ## Architecture
//!
//! - `domain/service.rs` - Core business logic
//! - `api/grpc/server.rs` - gRPC server implementation
//! - `module.rs` - Module registration and lifecycle
//!
//! External consumers should use `calculator-sdk` crate which provides
//! the gRPC client implementation. The `local_calculator` module registers
//! the client in ClientHub for in-process modules to use.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
// === MODULE DEFINITION ===
mod module;
pub use module::CalculatorModule;

// === INTERNAL MODULES ===
#[doc(hidden)]
pub mod api;
#[doc(hidden)]
pub mod domain;
