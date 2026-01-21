//! Calculator SDK
//!
//! This crate provides everything needed to consume the calculator service:
//! - API trait (`CalculatorClient`)
//! - Error types (`CalculatorError`)
//! - gRPC client implementation (`CalculatorGrpcClient`)
//! - Proto stubs for server implementation
//!
//! ## Usage
//!
//! ```ignore
//! use calculator_sdk::CalculatorClient;
//!
//! // Get the client from ClientHub (registered by local_calculator module)
//! let client = hub.get::<dyn CalculatorClient>()?;
//! let result = client.add(&ctx, 1, 2).await?;
//! ```

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

// === API TRAIT AND TYPES ===
mod api;
pub use api::{CalculatorClient, CalculatorError};

// === GRPC CLIENT ===
mod client;
pub use client::CalculatorGrpcClient;

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
