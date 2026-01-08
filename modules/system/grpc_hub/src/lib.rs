#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! gRPC Hub Module
//!
//! This module builds and hosts the single `tonic::Server` instance for the process.

// === MODULE DEFINITION ===
pub mod module;
pub use module::{GrpcHub, GrpcHubConfig};
