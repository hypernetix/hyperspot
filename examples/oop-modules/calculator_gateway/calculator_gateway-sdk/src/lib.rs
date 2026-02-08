//! Calculator Gateway SDK
//!
//! This crate provides everything needed to consume the calculator_gateway service:
//! - API trait (`CalculatorGatewayClientV1`)
//! - Error types (`CalculatorGatewayError`)
//! - Wiring function (`wire_client`)
//!
//! ## Usage
//!
//! ```ignore
//! use calculator_gateway_sdk::{CalculatorGatewayClientV1, wire_client};
//!
//! // Wire the client (module must be initialized first)
//! wire_client(ctx.client_hub())?;
//!
//! // Get the client from ClientHub
//! let client = hub.get::<dyn CalculatorGatewayClientV1>()?;
//! let result = client.add(&ctx, 1, 2).await?;
//! ```

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

mod api;
mod client;
mod wiring;

pub use api::{CalculatorGatewayClientV1, CalculatorGatewayError};
pub use wiring::wire_client;
