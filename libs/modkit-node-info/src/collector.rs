use crate::error::NodeInfoError;
use crate::model::*;
use crate::syscap_collector::SysCapCollector;
use crate::sysinfo_collector::SysInfoCollector;
use std::sync::Arc;

/// Main collector for node information
pub struct NodeInfoCollector {
    sysinfo_collector: Arc<SysInfoCollector>,
    syscap_collector: Arc<SysCapCollector>,
}

impl NodeInfoCollector {
    pub fn new() -> Self {
        Self {
            sysinfo_collector: Arc::new(SysInfoCollector::new()),
            syscap_collector: Arc::new(SysCapCollector::new()),
        }
    }

    /// Create a Node instance for the current machine
    /// Uses hardware UUID for node ID and collects hostname and local IP
    pub fn create_current_node(&self) -> Node {
        let id = crate::get_hardware_uuid();
        let hostname = sysinfo::System::host_name().unwrap_or_else(|| "unknown".to_string());
        let ip_address = Self::detect_local_ip();
        let now = chrono::Utc::now();

        Node {
            id,
            hostname,
            ip_address,
            created_at: now,
            updated_at: now,
        }
    }

    /// Detect the local IP address used for the default route to the internet
    /// This returns the IP address of the network interface that would be used
    /// for outbound internet traffic, not the public external IP.
    fn detect_local_ip() -> Option<String> {
        match local_ip_address::local_ip() {
            Ok(ip) => {
                let ip_str = ip.to_string();
                tracing::debug!(ip = %ip_str, "Detected local IP address");
                Some(ip_str)
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to detect local IP address");
                None
            }
        }
    }

    /// Collect system information for the current node
    pub fn collect_sysinfo(&self, node_id: uuid::Uuid) -> Result<NodeSysInfo, NodeInfoError> {
        self.sysinfo_collector
            .collect(node_id)
            .map_err(|e| NodeInfoError::SysInfoCollectionFailed(e.to_string()))
    }

    /// Collect system capabilities for the current node
    pub fn collect_syscap(&self, node_id: uuid::Uuid) -> Result<NodeSysCap, NodeInfoError> {
        self.syscap_collector
            .collect(node_id)
            .map_err(|e| NodeInfoError::SysCapCollectionFailed(e.to_string()))
    }

    /// Collect both sysinfo and syscap
    pub fn collect_all(
        &self,
        node_id: uuid::Uuid,
    ) -> Result<(NodeSysInfo, NodeSysCap), NodeInfoError> {
        let sysinfo = self.collect_sysinfo(node_id)?;
        let syscap = self.collect_syscap(node_id)?;
        Ok((sysinfo, syscap))
    }
}

impl Default for NodeInfoCollector {
    fn default() -> Self {
        Self::new()
    }
}
