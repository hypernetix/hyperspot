pub mod client;
pub mod error;

// Re-export models from modkit-node-info
pub use modkit_node_info::{
    BatteryInfo, CpuInfo, GpuInfo, HostInfo, MemoryInfo, Node, NodeSysCap, NodeSysInfo, OsInfo,
    SysCap,
};
