//! Directory API - contract for service discovery and instance resolution

use anyhow::Result;
use async_trait::async_trait;

use crate::runtime::{Endpoint, ModuleName};

/// Information about a service instance
#[derive(Debug, Clone)]
pub struct ServiceInstanceInfo {
    pub module: ModuleName,
    pub instance_id: String,
    pub endpoint: Endpoint,
    pub version: Option<String>,
}

/// Directory API trait for service discovery and instance management
#[async_trait]
pub trait DirectoryApi: Send + Sync {
    /// Resolve a gRPC service by its logical name to an endpoint
    async fn resolve_grpc_service(&self, service_name: &str) -> Result<Endpoint>;

    /// List all service instances for a given module
    async fn list_instances(&self, module: ModuleName) -> Result<Vec<ServiceInstanceInfo>>;
}
