//! Module Orchestrator
//!
//! System module for service discovery.
//! This module provides `DirectoryService` for gRPC service registration and discovery.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

// === MODULE DEFINITION ===
pub mod module;
pub use module::{ModuleOrchestrator, ModuleOrchestratorConfig};

// === INTERNAL MODULES (pub for integration tests) ===
pub mod api;
pub mod domain;
mod server;

// === RE-EXPORTS ===
pub use cf_system_sdks::directory::{
    DirectoryGrpcClient, RegisterInstanceInfo, ServiceEndpoint, ServiceInstanceInfo,
};
