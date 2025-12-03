use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Node response DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NodeDto {
    pub id: Uuid,
    pub hostname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// System information (included when details=true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sysinfo: Option<NodeSysInfoDto>,
    /// System capabilities (included when details=true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syscap: Option<NodeSysCapDto>,
}

/// System information response DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NodeSysInfoDto {
    pub node_id: uuid::Uuid,
    pub os: OsInfoDto,
    pub cpu: CpuInfoDto,
    pub memory: MemoryInfoDto,
    pub host: HostInfoDto,
    pub gpus: Vec<GpuInfoDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battery: Option<BatteryInfoDto>,
    pub collected_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OsInfoDto {
    pub name: String,
    pub version: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CpuInfoDto {
    pub model: String,
    pub num_cpus: u32,
    pub cores: u32,
    pub frequency_mhz: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MemoryInfoDto {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub used_percent: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HostInfoDto {
    pub hostname: String,
    pub uptime_seconds: u64,
    /// All detected IP addresses. The first one is the primary IP (used for default route).
    pub ip_addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GpuInfoDto {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cores: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_memory_mb: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_memory_mb: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BatteryInfoDto {
    pub on_battery: bool,
    pub percentage: u32,
}

/// System capabilities response DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NodeSysCapDto {
    pub node_id: uuid::Uuid,
    pub capabilities: Vec<SysCapDto>,
    pub collected_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SysCapDto {
    pub key: String,
    pub category: String,
    pub name: String,
    pub display_name: String,
    pub present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_dimension: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
    /// When this capability was last fetched (Unix timestamp in seconds)
    pub fetched_at_secs: i64,
}
