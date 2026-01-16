#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod api;
pub mod error;

pub use api::NodesRegistryClient;
pub use error::NodesRegistryError;

pub use modkit_node_info::{
    BatteryInfo, CpuInfo, GpuInfo, HostInfo, MemoryInfo, Node, NodeSysCap, NodeSysInfo, OsInfo,
    SysCap,
};
