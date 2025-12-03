use crate::contract::client::NodesRegistryApi;
use crate::contract::error::NodesRegistryError;
use crate::contract::{Node, NodeSysCap, NodeSysInfo};
use crate::domain::service::Service;
use std::sync::Arc;

/// Local client implementation for the nodes registry
pub struct NodesRegistryLocalClient {
    service: Arc<Service>,
}

impl NodesRegistryLocalClient {
    pub fn new(service: Arc<Service>) -> Self {
        Self { service }
    }
}

#[async_trait::async_trait]
impl NodesRegistryApi for NodesRegistryLocalClient {
    async fn get_node(&self, id: uuid::Uuid) -> Result<Node, NodesRegistryError> {
        self.service.get_node(id).await.map_err(|e| e.into())
    }

    async fn list_nodes(&self) -> Result<Vec<Node>, NodesRegistryError> {
        self.service.list_nodes().await.map_err(|e| e.into())
    }

    async fn get_node_sysinfo(
        &self,
        node_id: uuid::Uuid,
    ) -> Result<NodeSysInfo, NodesRegistryError> {
        self.service
            .get_node_sysinfo(node_id)
            .await
            .map_err(|e| e.into())
    }

    async fn get_node_syscap(&self, node_id: uuid::Uuid) -> Result<NodeSysCap, NodesRegistryError> {
        self.service
            .get_node_syscap(node_id, false)
            .await
            .map_err(|e| e.into())
    }
}
