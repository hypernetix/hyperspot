//! Unified bootstrap library for Modkit modules
//!
//! This crate provides bootstrap functionality for both host (in-process) and
//! `OoP` (out-of-process) Modkit modules.
//!
//! ## Modules
//!
//! - [`config`]: Configuration types and utilities
//! - [`host`]: Host/in-process bootstrap - logging, signals, and paths
//! - [`oop`]: Out-of-process module bootstrap - lifecycle management with `DirectoryService`
//!   (requires the `oop` feature)
//!
//! ## Backends
//!
//! Backend types for spawning `OoP` modules have been moved to `modkit::backends`.

pub mod config;
pub mod host;

pub mod oop;

// Re-export commonly used config types at crate root for convenience
pub use config::{
    AppConfig, CliArgs, LoggingConfig, MODKIT_MODULE_CONFIG_ENV, ModuleConfig, ModuleRuntime,
    RenderedModuleConfig, RuntimeKind, Section, ServerConfig, dump_effective_modules_config_json,
    dump_effective_modules_config_yaml, list_module_names, render_effective_modules_config,
};

// Re-export host types for convenience
pub use oop::{OopRunOptions, run_oop_with_options};

mod run;
pub use run::run_server;
