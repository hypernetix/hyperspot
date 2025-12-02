use crate::contract::{Node, NodeSysCap, NodeSysInfo, SysCap};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

/// Cached node data with timestamps
#[derive(Debug, Clone)]
struct CachedNodeData {
    node: Node,
    sysinfo: Option<NodeSysInfo>,
    /// System-collected capabilities from modkit-node-info
    syscap_system: Option<NodeSysCap>,
    /// Custom capabilities set through service interface
    syscap_custom: HashMap<String, SysCap>,
}

/// In-memory storage for nodes and their metadata
pub struct NodeStorage {
    nodes: RwLock<HashMap<Uuid, CachedNodeData>>,
}

impl NodeStorage {
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
        }
    }

    /// Register or update a node
    pub fn upsert_node(&self, node: Node) {
        let mut nodes = self.nodes.write().unwrap();
        nodes.insert(
            node.id,
            CachedNodeData {
                node,
                sysinfo: None,
                syscap_system: None,
                syscap_custom: HashMap::new(),
            },
        );
    }

    /// Get a node by ID
    pub fn get_node(&self, id: Uuid) -> Option<Node> {
        let nodes = self.nodes.read().unwrap();
        nodes.get(&id).map(|data| data.node.clone())
    }

    /// List all nodes
    pub fn list_nodes(&self) -> Vec<Node> {
        let nodes = self.nodes.read().unwrap();
        nodes.values().map(|data| data.node.clone()).collect()
    }

    /// Update sysinfo for a node
    pub fn update_sysinfo(&self, node_id: Uuid, sysinfo: NodeSysInfo) -> bool {
        let mut nodes = self.nodes.write().unwrap();
        if let Some(data) = nodes.get_mut(&node_id) {
            data.sysinfo = Some(sysinfo);
            true
        } else {
            false
        }
    }

    /// Get sysinfo for a node
    pub fn get_sysinfo(&self, node_id: Uuid) -> Option<NodeSysInfo> {
        let nodes = self.nodes.read().unwrap();
        nodes.get(&node_id).and_then(|data| data.sysinfo.clone())
    }

    /// Update system-collected syscap for a node
    pub fn update_syscap_system(&self, node_id: Uuid, syscap: NodeSysCap) -> bool {
        let mut nodes = self.nodes.write().unwrap();
        if let Some(data) = nodes.get_mut(&node_id) {
            data.syscap_system = Some(syscap);
            true
        } else {
            false
        }
    }

    /// Get merged syscap for a node (system + custom)
    pub fn get_syscap(&self, node_id: Uuid) -> Option<NodeSysCap> {
        let nodes = self.nodes.read().unwrap();
        if let Some(data) = nodes.get(&node_id) {
            // Merge system and custom capabilities
            let mut cap_map: HashMap<String, SysCap> = HashMap::new();

            // Add system capabilities first
            if let Some(ref syscap_system) = data.syscap_system {
                for cap in &syscap_system.capabilities {
                    cap_map.insert(cap.key.clone(), cap.clone());
                }
            }

            // Override/add custom capabilities
            for (key, cap) in &data.syscap_custom {
                cap_map.insert(key.clone(), cap.clone());
            }

            if cap_map.is_empty() {
                None
            } else {
                Some(NodeSysCap {
                    node_id,
                    capabilities: cap_map.into_values().collect(),
                    collected_at: chrono::Utc::now(),
                })
            }
        } else {
            None
        }
    }

    /// Set custom syscap entries (add or update)
    pub fn set_custom_syscap(&self, node_id: Uuid, caps: Vec<SysCap>) -> bool {
        let mut nodes = self.nodes.write().unwrap();
        if let Some(data) = nodes.get_mut(&node_id) {
            for cap in caps {
                data.syscap_custom.insert(cap.key.clone(), cap);
            }
            true
        } else {
            false
        }
    }

    /// Remove custom syscap entries by key
    pub fn remove_custom_syscap(&self, node_id: Uuid, keys: Vec<String>) -> bool {
        let mut nodes = self.nodes.write().unwrap();
        if let Some(data) = nodes.get_mut(&node_id) {
            for key in keys {
                data.syscap_custom.remove(&key);
            }
            true
        } else {
            false
        }
    }

    /// Clear all custom syscap entries for a node
    pub fn clear_custom_syscap(&self, node_id: Uuid) -> bool {
        let mut nodes = self.nodes.write().unwrap();
        if let Some(data) = nodes.get_mut(&node_id) {
            data.syscap_custom.clear();
            true
        } else {
            false
        }
    }

    /// Check if a system syscap entry needs refresh based on cache TTL
    #[allow(dead_code)]
    pub fn needs_syscap_refresh(&self, node_id: Uuid, key: &str) -> bool {
        let nodes = self.nodes.read().unwrap();
        if let Some(data) = nodes.get(&node_id) {
            if let Some(ref syscap_system) = data.syscap_system {
                if let Some(cap) = syscap_system.capabilities.iter().find(|c| c.key == key) {
                    let now = chrono::Utc::now().timestamp();
                    let age_secs = now - cap.fetched_at_secs;
                    return age_secs as u64 >= cap.cache_ttl_secs;
                }
            }
        }
        // If not found or no syscap, needs refresh
        true
    }

    /// Get all system syscap entries that need refresh
    pub fn get_expired_syscap_keys(&self, node_id: Uuid) -> Vec<String> {
        let nodes = self.nodes.read().unwrap();
        let mut expired_keys = Vec::new();

        if let Some(data) = nodes.get(&node_id) {
            if let Some(ref syscap_system) = data.syscap_system {
                let now = chrono::Utc::now().timestamp();
                for cap in &syscap_system.capabilities {
                    let age_secs = now - cap.fetched_at_secs;
                    if age_secs as u64 >= cap.cache_ttl_secs {
                        expired_keys.push(cap.key.clone());
                    }
                }
            }
        }

        expired_keys
    }
}

impl Default for NodeStorage {
    fn default() -> Self {
        Self::new()
    }
}
