mod grpc_installers;
mod host_runtime;
mod module_manager;
mod runner;
mod system_context;

/// Shutdown signal handling utilities
pub mod shutdown;

#[cfg(test)]
mod tests;

pub use grpc_installers::{GrpcInstallerData, GrpcInstallerStore, ModuleInstallers};
pub use host_runtime::{
    DbOptions, HostRuntime, MODKIT_DIRECTORY_ENDPOINT_ENV, MODKIT_MODULE_CONFIG_ENV,
};
pub use module_manager::{Endpoint, InstanceState, ModuleInstance, ModuleManager};
pub use runner::{
    ClientRegistration, OopModuleSpawnConfig, OopSpawnOptions, RunOptions, ShutdownOptions, run,
};
pub use system_context::SystemContext;
