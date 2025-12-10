#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! Node Information Library
//!
//! This library provides system information collection for the current node
//! where the code is executed. It collects:
//! - System information (OS, CPU, memory, GPU, battery, host)
//! - System capabilities (hardware and OS capabilities)
//!
//! This is a standalone library that can be used by any module to collect
//! information about the current execution environment.

mod hardware_uuid;
mod syscap_collector;
mod sysinfo_collector;

// Platform-specific GPU collectors
#[cfg(target_os = "linux")]
mod gpu_collector_linux;
#[cfg(target_os = "macos")]
mod gpu_collector_macos;
#[cfg(target_os = "windows")]
mod gpu_collector_windows;

pub mod error;
pub mod model;

mod collector;

pub use collector::NodeInfoCollector;
pub use error::NodeInfoError;
pub use hardware_uuid::get_hardware_uuid;
pub use model::*;
