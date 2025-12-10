//! Telemetry utilities for OpenTelemetry integration
//!
//! This module provides utilities for setting up and configuring
//! OpenTelemetry tracing layers for distributed tracing.

pub mod config;
pub mod init;

pub use config::{Exporter, HttpOpts, LogsCorrelation, Propagation, Sampler, TracingConfig};
pub use init::{init_tracing, shutdown_tracing};
