use crate::contract::{Node, NodeSysCap, NodeSysInfo, SysCap};
use crate::domain::error::DomainError;
use crate::domain::node_storage::NodeStorage;
use modkit_node_info::NodeInfoCollector;
use std::sync::Arc;

/// Check if a UUID is a fallback UUID (hardware detection failed)
/// Fallback UUIDs have zeros in the first 8 bytes: 00000000-0000-0000-xxxx-xxxxxxxxxxxx
fn is_fallback_uuid(id: &uuid::Uuid) -> bool {
    let bytes = id.as_bytes();
    bytes[..8] == [0u8; 8]
}

/// Service for managing nodes and their metadata
#[derive(Clone)]
pub struct Service {
    storage: Arc<NodeStorage>,
    node_info_collector: Arc<NodeInfoCollector>,
}

impl Service {
    pub fn new() -> Self {
        let node_info_collector = Arc::new(NodeInfoCollector::new());
        let current_node = NodeInfoCollector::create_current_node();
        let storage = Arc::new(NodeStorage::new());

        // Check if hardware detection failed (hybrid UUID with zeros on left)
        let is_fallback = is_fallback_uuid(&current_node.id);

        if is_fallback {
            tracing::warn!(
                node_id = %current_node.id,
                "Hardware UUID detection failed, using fallback UUID (00000000-0000-0000-xxxx-xxxxxxxxxxxx)"
            );
        } else {
            tracing::info!(
                node_id = %current_node.id,
                hostname = %current_node.hostname,
                ip_address = ?current_node.ip_address,
                "Initialized node with hardware-based UUID"
            );
        }

        storage.upsert_node(current_node);

        Self {
            storage,
            node_info_collector,
        }
    }

    /// Get a node by ID
    pub fn get_node(&self, id: uuid::Uuid) -> Result<Node, DomainError> {
        self.storage
            .get_node(id)
            .ok_or(DomainError::NodeNotFound(id))
    }

    /// List all nodes
    pub fn list_nodes(&self) -> Vec<Node> {
        self.storage.list_nodes()
    }

    /// Get system information for a node (with caching)
    pub fn get_node_sysinfo(&self, node_id: uuid::Uuid) -> Result<NodeSysInfo, DomainError> {
        // Check if node exists
        if self.storage.get_node(node_id).is_none() {
            return Err(DomainError::NodeNotFound(node_id));
        }

        // Try to get cached sysinfo
        if let Some(cached) = self.storage.get_sysinfo(node_id) {
            return Ok(cached);
        }

        // Collect fresh sysinfo
        let sysinfo = self
            .node_info_collector
            .collect_sysinfo(node_id)
            .map_err(DomainError::from)?;

        // Cache it
        self.storage.update_sysinfo(node_id, sysinfo.clone());

        Ok(sysinfo)
    }

    /// Get system capabilities for a node (with caching and merging)
    pub fn get_node_syscap(
        &self,
        node_id: uuid::Uuid,
        force_refresh: bool,
    ) -> Result<NodeSysCap, DomainError> {
        // Check if node exists
        if self.storage.get_node(node_id).is_none() {
            return Err(DomainError::NodeNotFound(node_id));
        }

        // Check if we need to refresh system capabilities
        let expired_keys = self.storage.get_expired_syscap_keys(node_id);
        let needs_refresh =
            force_refresh || !expired_keys.is_empty() || self.storage.get_syscap(node_id).is_none();

        if needs_refresh {
            // Collect fresh system capabilities
            let syscap_system = self
                .node_info_collector
                .collect_syscap(node_id)
                .map_err(DomainError::from)?;

            // Update system capabilities in storage
            self.storage.update_syscap_system(node_id, syscap_system);
        }

        // Return merged syscap (system + custom)
        self.storage
            .get_syscap(node_id)
            .ok_or(DomainError::SysCapCollectionFailed(
                "Failed to get syscap after refresh".to_string(),
            ))
    }

    /// Set custom syscap entries for a node
    pub fn set_custom_syscap(
        &self,
        node_id: uuid::Uuid,
        caps: Vec<SysCap>,
    ) -> Result<(), DomainError> {
        if !self.storage.set_custom_syscap(node_id, caps) {
            return Err(DomainError::NodeNotFound(node_id));
        }
        Ok(())
    }

    /// Remove custom syscap entries by key
    pub fn remove_custom_syscap(
        &self,
        node_id: uuid::Uuid,
        keys: Vec<String>,
    ) -> Result<(), DomainError> {
        if !self.storage.remove_custom_syscap(node_id, keys) {
            return Err(DomainError::NodeNotFound(node_id));
        }
        Ok(())
    }

    /// Clear all custom syscap entries for a node
    pub fn clear_custom_syscap(&self, node_id: uuid::Uuid) -> Result<(), DomainError> {
        if !self.storage.clear_custom_syscap(node_id) {
            return Err(DomainError::NodeNotFound(node_id));
        }
        Ok(())
    }
}

impl Default for Service {
    fn default() -> Self {
        Self::new()
    }
}
