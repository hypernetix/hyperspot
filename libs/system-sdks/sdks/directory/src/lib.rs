//! Module Orchestrator Contracts
//!
//! Domain contracts and client interfaces for module orchestration.
//! This crate provides the `DirectoryClient` trait and related types that
//! define the contract for service discovery and instance management.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod api;
#[cfg(feature = "grpc")]
mod grpc;

pub use api::{DirectoryClient, RegisterInstanceInfo, ServiceEndpoint, ServiceInstanceInfo};
#[cfg(feature = "grpc")]
pub use grpc::*;
