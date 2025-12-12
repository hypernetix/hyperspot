//! Calculator Gateway Module
//!
//! An in-process module that exposes a REST API for addition.
//! It delegates the actual computation to the calculator service via gRPC.
//!
//! ## Architecture
//!
//! - `Service` contains the domain logic
//! - REST handlers call Service directly
//! - External consumers use the SDK (`calculator_gateway-sdk`) which provides
//!   `CalculatorGatewayClient` trait and `wire_client()` for ClientHub integration

// === MODULE DEFINITION ===
mod module;
pub use module::CalculatorGateway;

// === PUBLIC EXPORTS (for SDK) ===
pub mod domain;
pub use domain::{Service, ServiceError};

// === INTERNAL MODULES ===
#[doc(hidden)]
pub mod api;
