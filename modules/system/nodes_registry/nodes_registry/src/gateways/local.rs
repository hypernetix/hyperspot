use crate::domain::service::Service;
use nodes_registry_sdk::{Node, NodeSysCap, NodeSysInfo, NodesRegistryClient, NodesRegistryError};
use std::sync::Arc;

/// Local client implementation for the nodes registry
pub struct NodesRegistryLocalClient {
    service: Arc<Service>,
}

impl NodesRegistryLocalClient {
    #[must_use]
    pub fn new(service: Arc<Service>) -> Self {
        Self { service }
    }
}

#[async_trait::async_trait]
impl NodesRegistryClient for NodesRegistryLocalClient {
    async fn get_node(&self, id: uuid::Uuid) -> Result<Node, NodesRegistryError> {
        self.service.get_node(id).map_err(Into::into)
    }

    async fn list_nodes(&self) -> Result<Vec<Node>, NodesRegistryError> {
        Ok(self.service.list_nodes())
    }

    async fn get_node_sysinfo(
        &self,
        node_id: uuid::Uuid,
    ) -> Result<NodeSysInfo, NodesRegistryError> {
        self.service.get_node_sysinfo(node_id).map_err(Into::into)
    }

    async fn get_node_syscap(&self, node_id: uuid::Uuid) -> Result<NodeSysCap, NodesRegistryError> {
        self.service
            .get_node_syscap(node_id, false)
            .map_err(Into::into)
    }
}
