//! Telemetry utilities for OpenTelemetry integration
//!
//! This module provides utilities for setting up and configuring
//! OpenTelemetry tracing layers for distributed tracing.

pub mod init;

pub use init::{init_tracing, shutdown_tracing};
