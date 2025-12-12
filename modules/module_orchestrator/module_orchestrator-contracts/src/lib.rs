//! Module Orchestrator Contracts
//!
//! Domain contracts and client interfaces for module orchestration.
//! This crate provides the `DirectoryApi` trait and related types that
//! define the contract for service discovery and instance management.

mod api;

pub use api::{DirectoryApi, RegisterInstanceInfo, ServiceEndpoint, ServiceInstanceInfo};
