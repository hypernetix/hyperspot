//! Directory API - contract for service discovery and instance resolution

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use crate::runtime::{Endpoint, ModuleManager};

/// Information about a service instance
#[derive(Debug, Clone)]
pub struct ServiceInstanceInfo {
    pub module: String,
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
    async fn list_instances(&self, module: &str) -> Result<Vec<ServiceInstanceInfo>>;
}

pub struct LocalDirectoryApi {
    mgr: Arc<ModuleManager>,
}

impl LocalDirectoryApi {
    pub fn new(mgr: Arc<ModuleManager>) -> Self {
        Self { mgr }
    }
}

#[async_trait]
impl DirectoryApi for LocalDirectoryApi {
    async fn resolve_grpc_service(&self, service_name: &str) -> Result<Endpoint> {
        if let Some((_module, _inst, ep)) = self.mgr.pick_service_round_robin(service_name) {
            return Ok(ep);
        }

        anyhow::bail!(
            "Service not found or no healthy instances: {}",
            service_name
        )
    }

    async fn list_instances(&self, module: &str) -> Result<Vec<ServiceInstanceInfo>> {
        let mut result = Vec::new();

        for inst in self.mgr.instances_of_static(module) {
            if let Some((_, ep)) = inst.grpc_services.iter().next() {
                result.push(ServiceInstanceInfo {
                    module: module.to_string(),
                    instance_id: inst.instance_id.clone(),
                    endpoint: ep.clone(),
                    version: inst.version.clone(),
                });
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::runtime::{Endpoint, ModuleInstance, ModuleManager};
    use std::sync::Arc;
    use std::time::Instant;

    #[tokio::test]
    async fn test_resolve_rr() {
        let dir = Arc::new(ModuleManager::new());
        let api = LocalDirectoryApi::new(dir.clone());

        // Register two instances providing the same service
        let inst1 = Arc::new(
            ModuleInstance::new("test_module", "instance1")
                .with_grpc_service("test.Service", Endpoint::tcp("127.0.0.1", 8001)),
        );
        let inst2 = Arc::new(
            ModuleInstance::new("test_module", "instance2")
                .with_grpc_service("test.Service", Endpoint::tcp("127.0.0.1", 8002)),
        );

        dir.register_instance(inst1);
        dir.register_instance(inst2);

        // Mark both as healthy
        dir.update_heartbeat("test_module", "instance1", Instant::now());
        dir.update_heartbeat("test_module", "instance2", Instant::now());

        // Resolve should rotate between instances
        let ep1 = api.resolve_grpc_service("test.Service").await.unwrap();
        let ep2 = api.resolve_grpc_service("test.Service").await.unwrap();
        let ep3 = api.resolve_grpc_service("test.Service").await.unwrap();

        // First and third should be the same (round-robin)
        assert_eq!(ep1, ep3);
        // First and second should be different
        assert_ne!(ep1, ep2);
    }

    #[tokio::test]
    async fn test_list_instances_returns_string() {
        let dir = Arc::new(ModuleManager::new());
        let api = LocalDirectoryApi::new(dir.clone());

        let inst = Arc::new(
            ModuleInstance::new("test_module", "instance1")
                .with_grpc_service("test.Service", Endpoint::tcp("127.0.0.1", 8001)),
        );

        dir.register_instance(inst);

        let instances = api.list_instances("test_module").await.unwrap();
        assert_eq!(instances.len(), 1);
        // Verify that module is a String, not &'static str
        assert_eq!(instances[0].module, "test_module".to_string());
    }

    #[tokio::test]
    async fn test_resolve_filters_unhealthy() {
        let dir = Arc::new(ModuleManager::new());
        let api = LocalDirectoryApi::new(dir.clone());

        // Register instance but mark it as quarantined
        let inst = Arc::new(
            ModuleInstance::new("test_module", "instance1")
                .with_grpc_service("test.Service", Endpoint::tcp("127.0.0.1", 8001)),
        );

        dir.register_instance(inst);
        dir.mark_quarantined("test_module", "instance1");

        // Should not resolve quarantined instance
        let result = api.resolve_grpc_service("test.Service").await;
        assert!(result.is_err());
    }
}
