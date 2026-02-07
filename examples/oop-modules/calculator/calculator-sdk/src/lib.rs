//! Calculator SDK
//!
//! This crate provides everything needed to consume the calculator service:
//! - API trait (`CalculatorClientV1`)
//! - Error types (`CalculatorError`)
//! - Wiring function (`wire_client`)
//! - Proto stubs for server implementation
//!
//! ## Usage
//!
//! ```ignore
//! use calculator_sdk::{CalculatorClientV1, wire_client};
//!
//! // Wire the client into ClientHub
//! wire_client(&hub, &directory).await?;
//!
//! // Get the client from ClientHub
//! let client = hub.get::<dyn CalculatorClientV1>()?;
//! let result = client.add(&ctx, 1, 2).await?;
//! ```

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

// === API TRAIT AND TYPES ===
mod api;
pub use api::{CalculatorClientV1, CalculatorError};

// === WIRING ===
mod client;
mod wiring;
pub use wiring::wire_client;

// === GRPC PROTO STUBS (for server implementation) ===
/// Generated protobuf types for CalculatorService
pub mod proto {
    tonic::include_proto!("oop.calculator.v1");
}

// Re-export proto types needed by server
pub use proto::calculator_service_server::{CalculatorService, CalculatorServiceServer};
pub use proto::{AddRequest, AddResponse};

/// Service name constant for CalculatorService (used for service discovery)
pub const SERVICE_NAME: &str = "calculator.v1.CalculatorService";
