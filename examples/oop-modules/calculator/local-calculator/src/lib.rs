//! Local Calculator Module
//!
//! A module that provides LocalCalculatorClient implementation.
//! The client lazily initializes the gRPC connection to the calculator service
//! using DirectoryClient for service discovery.

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

// === MODULE DEFINITION ===
mod module;
pub use module::LocalCalculatorModule;

// === INTERNAL MODULES ===
mod client;
