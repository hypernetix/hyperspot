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
//! the gRPC client and `wire_client()` for ClientHub integration.

// === MODULE DEFINITION ===
mod module;
pub use module::CalculatorModule;

// === INTERNAL MODULES ===
#[doc(hidden)]
pub mod api;
#[doc(hidden)]
pub mod domain;
