//! Module Orchestrator gRPC Layer
//!
//! This crate provides gRPC transport for the module orchestrator.
//! It includes generated protobuf types and client/server implementations.

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

mod client;

// Generated protobuf types for DirectoryService
pub mod directory {
    tonic::include_proto!("module_orchestrator.v1.directory");
}

// Re-export common types for DirectoryService
pub use directory::directory_service_client::DirectoryServiceClient;
pub use directory::directory_service_server::{DirectoryService, DirectoryServiceServer};
pub use directory::{
    DeregisterInstanceRequest, GrpcServiceEndpoint, HeartbeatRequest, InstanceInfo,
    ListInstancesRequest, ListInstancesResponse, RegisterInstanceRequest,
    ResolveGrpcServiceRequest, ResolveGrpcServiceResponse,
};

// Re-export the gRPC client implementation
pub use client::DirectoryGrpcClient;

/// Service name constant for DirectoryService
pub const DIRECTORY_SERVICE_NAME: &str =
    <DirectoryServiceServer<()> as tonic::server::NamedService>::NAME;
