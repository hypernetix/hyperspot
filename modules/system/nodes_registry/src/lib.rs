//! Nodes Registry Module
//!
//! This module manages node information in the Hyperspot deployment.
//! A node represents a deployment unit (host, VM, container) where Hyperspot is running.
//!
//! Each node contains:
//! - System information (OS, CPU, memory, etc.)
//! - System capabilities (hardware, software features)
//! - Node metadata (ID, hostname, IP, etc.)
//!
//! The module provides REST API endpoints to:
//! - List all nodes
//! - Get node information by ID
//! - Access node sysinfo via /nodes/{id}/sysinfo
//! - Access node syscap via /nodes/{id}/syscap
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

// === PUBLIC CONTRACT ===
pub mod contract;
pub use contract::{client, error};

// === MODULE DEFINITION ===
pub mod module;
pub use module::NodesRegistry;

// === INTERNAL MODULES ===
#[doc(hidden)]
pub mod api;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod domain;
#[doc(hidden)]
pub mod gateways;
