//! Local client implementation of DirectoryApi

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use modkit::runtime::{get_global_instance_directory, Endpoint, InstanceDirectory, ModuleName};
use modkit::{DirectoryApi, ServiceInstanceInfo};

/// Local implementation of DirectoryApi that reads from the global InstanceDirectory
pub struct DirectoryLocalClient {
    dir: Arc<InstanceDirectory>,
}

impl DirectoryLocalClient {
    pub fn new() -> Self {
        let dir = get_global_instance_directory();
        Self { dir }
    }
}

impl Default for DirectoryLocalClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DirectoryApi for DirectoryLocalClient {
    async fn resolve_grpc_service(&self, service_name: &str) -> Result<Endpoint> {
        for inst in self.dir.all_instances() {
            if let Some(ep) = inst.grpc_services.get(service_name) {
                return Ok(ep.clone());
            }
        }
        Err(anyhow::anyhow!("Service not found: {}", service_name))
    }

    async fn list_instances(&self, module: ModuleName) -> Result<Vec<ServiceInstanceInfo>> {
        let mut out = Vec::new();
        for inst in self.dir.instances_of(module) {
            for ep in inst.grpc_services.values() {
                out.push(ServiceInstanceInfo {
                    module: inst.module,
                    instance_id: inst.instance_id.clone(),
                    endpoint: ep.clone(),
                    version: inst.version.clone(),
                });
            }
        }
        Ok(out)
    }
}
