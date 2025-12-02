/// Node represents a deployment unit where Hyperspot is running
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: uuid::Uuid,
    pub hostname: String,
    pub ip_address: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// System information for a node
#[derive(Debug, Clone, PartialEq)]
pub struct NodeSysInfo {
    pub node_id: uuid::Uuid,
    pub os: OsInfo,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub host: HostInfo,
    pub gpus: Vec<GpuInfo>,
    pub battery: Option<BatteryInfo>,
    pub collected_at: chrono::DateTime<chrono::Utc>,
}

/// Operating system information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub arch: String,
}

/// CPU information
#[derive(Debug, Clone, PartialEq)]
pub struct CpuInfo {
    pub model: String,
    pub num_cpus: u32,
    pub cores: u32,
    pub frequency_mhz: f64,
}

/// Memory information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub used_percent: u32,
}

/// Host information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostInfo {
    pub hostname: String,
    pub uptime_seconds: u64,
    /// All detected IP addresses. The first one is the primary IP (used for default route).
    pub ip_addresses: Vec<String>,
}

/// GPU information
#[derive(Debug, Clone, PartialEq)]
pub struct GpuInfo {
    pub model: String,
    pub cores: Option<u32>,
    pub total_memory_mb: Option<f64>,
    pub used_memory_mb: Option<f64>,
}

/// Battery information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatteryInfo {
    pub on_battery: bool,
    pub percentage: u32,
}

/// System capability information for a node
#[derive(Debug, Clone, PartialEq)]
pub struct NodeSysCap {
    pub node_id: uuid::Uuid,
    pub capabilities: Vec<SysCap>,
    pub collected_at: chrono::DateTime<chrono::Utc>,
}

/// Individual system capability
#[derive(Debug, Clone, PartialEq)]
pub struct SysCap {
    pub key: String,
    pub category: String,
    pub name: String,
    pub display_name: String,
    pub present: bool,
    pub version: Option<String>,
    pub amount: Option<f64>,
    pub amount_dimension: Option<String>,
    pub details: Option<String>,
    /// Cache TTL in seconds - how long this capability data is valid
    pub cache_ttl_secs: u64,
    /// When this capability was last fetched (Unix timestamp in seconds)
    pub fetched_at_secs: i64,
}
