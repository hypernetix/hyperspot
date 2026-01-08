use crate::contract::error::NodesRegistryError;
use crate::contract::{Node, NodeSysCap, NodeSysInfo};

/// Client trait for accessing nodes registry functionality
#[async_trait::async_trait]
pub trait NodesRegistryApi: Send + Sync {
    /// Get a node by ID
    async fn get_node(&self, id: uuid::Uuid) -> Result<Node, NodesRegistryError>;

    /// List all nodes
    async fn list_nodes(&self) -> Result<Vec<Node>, NodesRegistryError>;

    /// Get system information for a node
    async fn get_node_sysinfo(
        &self,
        node_id: uuid::Uuid,
    ) -> Result<NodeSysInfo, NodesRegistryError>;

    /// Get system capabilities for a node
    async fn get_node_syscap(&self, node_id: uuid::Uuid) -> Result<NodeSysCap, NodesRegistryError>;
}
