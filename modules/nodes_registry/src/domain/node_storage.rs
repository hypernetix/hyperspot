use crate::contract::{Node, NodeSysCap, NodeSysInfo, SysCap};
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::warn;
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
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
        }
    }

    /// Register or update a node
    pub fn upsert_node(&self, node: Node) {
        match self.nodes.write() {
            Ok(mut nodes) => {
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
            Err(_) => {
                warn!("RwLock is poisoned in upsert_node, cannot update node");
            }
        }
    }

    /// Get a node by ID
    pub fn get_node(&self, id: Uuid) -> Option<Node> {
        if let Ok(nodes) = self.nodes.read() {
            nodes.get(&id).map(|data| data.node.clone())
        } else {
            warn!("RwLock is poisoned in get_node, cannot access node");
            None
        }
    }

    /// List all nodes
    pub fn list_nodes(&self) -> Vec<Node> {
        if let Ok(nodes) = self.nodes.read() {
            nodes.values().map(|data| data.node.clone()).collect()
        } else {
            warn!("RwLock is poisoned in list_nodes, cannot access nodes");
            Vec::new()
        }
    }

    /// Update sysinfo for a node
    pub fn update_sysinfo(&self, node_id: Uuid, sysinfo: NodeSysInfo) -> bool {
        if let Ok(mut nodes) = self.nodes.write() {
            if let Some(data) = nodes.get_mut(&node_id) {
                data.sysinfo = Some(sysinfo);
                true
            } else {
                false
            }
        } else {
            warn!("RwLock is poisoned in update_sysinfo, cannot update node");
            false
        }
    }

    /// Get sysinfo for a node
    pub fn get_sysinfo(&self, node_id: Uuid) -> Option<NodeSysInfo> {
        if let Ok(nodes) = self.nodes.read() {
            nodes.get(&node_id).and_then(|data| data.sysinfo.clone())
        } else {
            warn!("RwLock is poisoned in get_sysinfo, cannot access node");
            None
        }
    }

    /// Update system-collected syscap for a node
    pub fn update_syscap_system(&self, node_id: Uuid, syscap: NodeSysCap) -> bool {
        if let Ok(mut nodes) = self.nodes.write() {
            if let Some(data) = nodes.get_mut(&node_id) {
                data.syscap_system = Some(syscap);
                true
            } else {
                false
            }
        } else {
            warn!("RwLock is poisoned in update_syscap_system, cannot update node");
            false
        }
    }

    /// Get merged syscap for a node (system + custom)
    pub fn get_syscap(&self, node_id: Uuid) -> Option<NodeSysCap> {
        if let Ok(nodes) = self.nodes.read() {
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
        } else {
            warn!("RwLock is poisoned in get_syscap, cannot access node");
            None
        }
    }

    /// Set custom syscap entries (add or update)
    pub fn set_custom_syscap(&self, node_id: Uuid, caps: Vec<SysCap>) -> bool {
        if let Ok(mut nodes) = self.nodes.write() {
            if let Some(data) = nodes.get_mut(&node_id) {
                for cap in caps {
                    data.syscap_custom.insert(cap.key.clone(), cap);
                }
                true
            } else {
                false
            }
        } else {
            warn!("RwLock is poisoned in set_custom_syscap, cannot update node");
            false
        }
    }

    /// Remove custom syscap entries by key
    pub fn remove_custom_syscap(&self, node_id: Uuid, keys: Vec<String>) -> bool {
        if let Ok(mut nodes) = self.nodes.write() {
            if let Some(data) = nodes.get_mut(&node_id) {
                for key in keys {
                    data.syscap_custom.remove(&key);
                }
                true
            } else {
                false
            }
        } else {
            warn!("RwLock is poisoned in remove_custom_syscap, cannot update node");
            false
        }
    }

    /// Clear all custom syscap entries for a node
    pub fn clear_custom_syscap(&self, node_id: Uuid) -> bool {
        if let Ok(mut nodes) = self.nodes.write() {
            if let Some(data) = nodes.get_mut(&node_id) {
                data.syscap_custom.clear();
                true
            } else {
                false
            }
        } else {
            warn!("RwLock is poisoned in clear_custom_syscap, cannot update node");
            false
        }
    }

    /// Check if a system syscap entry needs refresh based on cache TTL
    #[allow(dead_code)]
    pub fn needs_syscap_refresh(&self, node_id: Uuid, key: &str) -> bool {
        if let Ok(nodes) = self.nodes.read() {
            if let Some(data) = nodes.get(&node_id) {
                if let Some(ref syscap_system) = data.syscap_system {
                    let now = chrono::Utc::now();

                    return syscap_system
                        .capabilities
                        .iter()
                        .any(|c| c.key == key && c.cache_is_expired(now));
                }
            }
            // If not found or no syscap, needs refresh
            true
        } else {
            warn!("RwLock is poisoned in needs_syscap_refresh, cannot access node");
            true // Assume needs refresh on error
        }
    }

    /// Get all system syscap entries that need refresh
    pub fn get_expired_syscap_keys(&self, node_id: Uuid) -> Vec<String> {
        if let Ok(nodes) = self.nodes.read() {
            let mut expired_keys = Vec::new();

            if let Some(data) = nodes.get(&node_id) {
                if let Some(ref syscap_system) = data.syscap_system {
                    let now = chrono::Utc::now();
                    syscap_system
                        .capabilities
                        .iter()
                        .filter(|cap| cap.cache_is_expired(now))
                        .for_each(|cap| expired_keys.push(cap.key.clone()));
                }
            }

            expired_keys
        } else {
            warn!("RwLock is poisoned in get_expired_syscap_keys, cannot access node");
            Vec::new()
        }
    }
}

trait CacheableCapability {
    fn cache_is_expired(&self, now: chrono::DateTime<chrono::Utc>) -> bool;
}

impl CacheableCapability for SysCap {
    fn cache_is_expired(&self, now: chrono::DateTime<chrono::Utc>) -> bool {
        let now_secs = now.timestamp();
        #[allow(clippy::cast_sign_loss)]
        let age_secs = (now_secs - self.fetched_at_secs).max(0) as u64;
        age_secs >= self.cache_ttl_secs
    }
}

impl Default for NodeStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::contract::SysCap;
    use chrono::Utc;

    fn make_syscap_with(fetched_at_secs: i64, cache_ttl_secs: u64) -> SysCap {
        SysCap {
            key: "k".to_owned(),
            category: "c".to_owned(),
            name: "n".to_owned(),
            display_name: "d".to_owned(),
            present: true,
            version: None,
            amount: None,
            amount_dimension: None,
            details: None,
            cache_ttl_secs,
            fetched_at_secs,
        }
    }

    #[test]
    fn cache_not_expired_before_ttl() {
        let now = Utc::now();
        let ttl = 10u64;
        let fetched_at = now.timestamp() - 9; // 9 seconds ago

        let cap = make_syscap_with(fetched_at, ttl);

        assert!(!cap.cache_is_expired(now));
    }

    #[test]
    fn cache_expired_at_ttl_boundary() {
        let now = Utc::now();
        let ttl = 10u64;
        let fetched_at = now.timestamp() - 10; // exactly ttl seconds ago

        let cap = make_syscap_with(fetched_at, ttl);

        assert!(cap.cache_is_expired(now));
    }

    #[test]
    fn future_fetched_at_counts_as_fresh_when_ttl_positive() {
        let now = Utc::now();
        let ttl = 5u64;
        let fetched_at = now.timestamp() + 60; // fetched in the future

        let cap = make_syscap_with(fetched_at, ttl);

        // age is treated as 0, so not expired for positive ttl
        assert!(!cap.cache_is_expired(now));
    }

    #[test]
    fn zero_ttl_always_expired() {
        let now = Utc::now();
        let ttl = 0u64;
        let fetched_at = now.timestamp();

        let cap = make_syscap_with(fetched_at, ttl);

        assert!(cap.cache_is_expired(now));
    }
}
