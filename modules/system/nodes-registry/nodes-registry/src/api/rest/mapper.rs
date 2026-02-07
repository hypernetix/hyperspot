use super::dto::{
    BatteryInfoDto, CpuInfoDto, GpuInfoDto, HostInfoDto, MemoryInfoDto, NodeDto, NodeSysCapDto,
    NodeSysInfoDto, OsInfoDto, SysCapDto,
};
use nodes_registry_sdk::{
    BatteryInfo, CpuInfo, GpuInfo, HostInfo, MemoryInfo, Node, NodeSysCap, NodeSysInfo, OsInfo,
    SysCap,
};

// Node mappings
impl From<Node> for NodeDto {
    fn from(node: Node) -> Self {
        Self {
            id: node.id,
            hostname: node.hostname,
            ip_address: node.ip_address,
            created_at: node.created_at,
            updated_at: node.updated_at,
            sysinfo: None,
            syscap: None,
        }
    }
}

// SysInfo mappings
impl From<NodeSysInfo> for NodeSysInfoDto {
    fn from(info: NodeSysInfo) -> Self {
        Self {
            node_id: info.node_id,
            os: info.os.into(),
            cpu: info.cpu.into(),
            memory: info.memory.into(),
            host: info.host.into(),
            gpus: info.gpus.into_iter().map(Into::into).collect(),
            battery: info.battery.map(Into::into),
            collected_at: info.collected_at,
        }
    }
}

impl From<OsInfo> for OsInfoDto {
    fn from(info: OsInfo) -> Self {
        Self {
            name: info.name,
            version: info.version,
            arch: info.arch,
        }
    }
}

impl From<CpuInfo> for CpuInfoDto {
    fn from(info: CpuInfo) -> Self {
        Self {
            model: info.model,
            num_cpus: info.num_cpus,
            cores: info.cores,
            frequency_mhz: info.frequency_mhz,
        }
    }
}

impl From<MemoryInfo> for MemoryInfoDto {
    fn from(info: MemoryInfo) -> Self {
        Self {
            total_bytes: info.total_bytes,
            available_bytes: info.available_bytes,
            used_bytes: info.used_bytes,
            used_percent: info.used_percent,
        }
    }
}

impl From<HostInfo> for HostInfoDto {
    fn from(info: HostInfo) -> Self {
        Self {
            hostname: info.hostname,
            uptime_seconds: info.uptime_seconds,
            ip_addresses: info.ip_addresses,
        }
    }
}

impl From<GpuInfo> for GpuInfoDto {
    fn from(info: GpuInfo) -> Self {
        Self {
            model: info.model,
            cores: info.cores,
            total_memory_mb: info.total_memory_mb,
            used_memory_mb: info.used_memory_mb,
        }
    }
}

impl From<BatteryInfo> for BatteryInfoDto {
    fn from(info: BatteryInfo) -> Self {
        Self {
            on_battery: info.on_battery,
            percentage: info.percentage,
        }
    }
}

// SysCap mappings
impl From<NodeSysCap> for NodeSysCapDto {
    fn from(cap: NodeSysCap) -> Self {
        Self {
            node_id: cap.node_id,
            capabilities: cap.capabilities.into_iter().map(Into::into).collect(),
            collected_at: cap.collected_at,
        }
    }
}

impl From<SysCap> for SysCapDto {
    fn from(cap: SysCap) -> Self {
        Self {
            key: cap.key,
            category: cap.category,
            name: cap.name,
            display_name: cap.display_name,
            present: cap.present,
            version: cap.version,
            amount: cap.amount,
            amount_dimension: cap.amount_dimension,
            details: cap.details,
            cache_ttl_secs: cap.cache_ttl_secs,
            fetched_at_secs: cap.fetched_at_secs,
        }
    }
}
