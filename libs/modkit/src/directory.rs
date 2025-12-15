//! Directory API - contract for service discovery and instance resolution
//!
//! This module re-exports the `DirectoryApi` trait and related types from
//! `module_orchestrator_contracts` and provides the `LocalDirectoryApi` implementation.

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::runtime::{Endpoint, ModuleInstance, ModuleManager};

// Re-export all types from contracts - this is the single source of truth
pub use module_orchestrator_contracts::{
    DirectoryApi, RegisterInstanceInfo, ServiceEndpoint, ServiceInstanceInfo,
};

/// Local implementation of `DirectoryApi` that delegates to `ModuleManager`
///
/// This is the in-process implementation used by modules running in the same
/// process as the module orchestrator.
pub struct LocalDirectoryApi {
    mgr: Arc<ModuleManager>,
}

impl LocalDirectoryApi {
    #[must_use]
    pub fn new(mgr: Arc<ModuleManager>) -> Self {
        Self { mgr }
    }
}

#[async_trait]
impl DirectoryApi for LocalDirectoryApi {
    async fn resolve_grpc_service(&self, service_name: &str) -> Result<ServiceEndpoint> {
        if let Some((_module, _inst, ep)) = self.mgr.pick_service_round_robin(service_name) {
            return Ok(ServiceEndpoint::new(ep.uri));
        }

        anyhow::bail!("Service not found or no healthy instances: {service_name}")
    }

    async fn list_instances(&self, module: &str) -> Result<Vec<ServiceInstanceInfo>> {
        let mut result = Vec::new();

        for inst in self.mgr.instances_of(module) {
            if let Some((_, ep)) = inst.grpc_services.iter().next() {
                result.push(ServiceInstanceInfo {
                    module: module.to_owned(),
                    instance_id: inst.instance_id.to_string(),
                    endpoint: ServiceEndpoint::new(ep.uri.clone()),
                    version: inst.version.clone(),
                });
            }
        }

        Ok(result)
    }

    async fn register_instance(&self, info: RegisterInstanceInfo) -> Result<()> {
        // Parse instance_id from string to Uuid
        let instance_id = Uuid::parse_str(&info.instance_id)
            .map_err(|e| anyhow::anyhow!("Invalid instance_id '{}': {}", info.instance_id, e))?;

        // Build a ModuleInstance from RegisterInstanceInfo
        let mut instance = ModuleInstance::new(info.module.clone(), instance_id);

        // Apply version if provided
        if let Some(version) = info.version {
            instance = instance.with_version(version);
        }

        // Add all gRPC services
        for (service_name, endpoint) in info.grpc_services {
            instance = instance.with_grpc_service(service_name, Endpoint::from_uri(endpoint.uri));
        }

        // Register the instance with the manager
        self.mgr.register_instance(Arc::new(instance));

        Ok(())
    }

    async fn deregister_instance(&self, module: &str, instance_id: &str) -> Result<()> {
        let instance_id = Uuid::parse_str(instance_id)
            .map_err(|e| anyhow::anyhow!("Invalid instance_id '{instance_id}': {e}"))?;
        self.mgr.deregister(module, instance_id);
        Ok(())
    }

    async fn send_heartbeat(&self, module: &str, instance_id: &str) -> Result<()> {
        let instance_id = Uuid::parse_str(instance_id)
            .map_err(|e| anyhow::anyhow!("Invalid instance_id '{instance_id}': {e}"))?;
        self.mgr
            .update_heartbeat(module, instance_id, std::time::Instant::now());
        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resolve_grpc_service_not_found() {
        let dir = Arc::new(ModuleManager::new());
        let api = LocalDirectoryApi::new(dir);

        let result = api.resolve_grpc_service("nonexistent.Service").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_instance_via_api() {
        let dir = Arc::new(ModuleManager::new());
        let api = LocalDirectoryApi::new(dir.clone());

        let instance_id = Uuid::new_v4();
        // Register an instance through the API
        let register_info = RegisterInstanceInfo {
            module: "test_module".to_owned(),
            instance_id: instance_id.to_string(),
            grpc_services: vec![(
                "test.Service".to_owned(),
                ServiceEndpoint::http("127.0.0.1", 8001),
            )],
            version: Some("1.0.0".to_owned()),
        };

        api.register_instance(register_info).await.unwrap();

        // Verify the instance was registered
        let instances = dir.instances_of("test_module");
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].instance_id, instance_id);
        assert_eq!(instances[0].version, Some("1.0.0".to_owned()));
        assert!(instances[0].grpc_services.contains_key("test.Service"));
    }

    #[tokio::test]
    async fn test_deregister_instance_via_api() {
        let dir = Arc::new(ModuleManager::new());
        let api = LocalDirectoryApi::new(dir.clone());

        let instance_id = Uuid::new_v4();
        // Register an instance first
        let inst = Arc::new(ModuleInstance::new("test_module", instance_id));
        dir.register_instance(inst);

        // Verify it exists
        assert_eq!(dir.instances_of("test_module").len(), 1);

        // Deregister via API
        api.deregister_instance("test_module", &instance_id.to_string())
            .await
            .unwrap();

        // Verify it's gone
        assert_eq!(dir.instances_of("test_module").len(), 0);
    }

    #[tokio::test]
    async fn test_send_heartbeat_via_api() {
        use crate::runtime::InstanceState;

        let dir = Arc::new(ModuleManager::new());
        let api = LocalDirectoryApi::new(dir.clone());

        let instance_id = Uuid::new_v4();
        // Register an instance first
        let inst = Arc::new(ModuleInstance::new("test_module", instance_id));
        dir.register_instance(inst);

        // Verify initial state is Registered
        let instances = dir.instances_of("test_module");
        assert_eq!(instances[0].state(), InstanceState::Registered);

        // Send heartbeat via API
        api.send_heartbeat("test_module", &instance_id.to_string())
            .await
            .unwrap();

        // Verify state transitioned to Healthy
        let instances = dir.instances_of("test_module");
        assert_eq!(instances[0].state(), InstanceState::Healthy);
    }
}
