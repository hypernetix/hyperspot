//! Host/in-process bootstrap module
//!
//! This module provides logging initialization, signal handling,
//! and path utilities for host processes.
//!
//! Configuration types are now in the top-level `config` module.

pub mod config_provider;
pub mod logging;
pub mod paths;
pub mod signals;

// Re-export config types from the crate-level config module for backwards compatibility
pub use crate::config::*;

pub use config_provider::*;
pub use logging::*;
pub use paths::{expand_tilde, normalize_executable_path, HomeDirError};
pub use signals::*;
