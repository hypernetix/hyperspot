mod backend;
mod grpc_installers;
mod host_runtime;
mod module_manager;
mod runner;
mod shutdown;
mod system_context;

#[cfg(test)]
mod tests;

// Backend module with trait and implementations
pub mod backends;

// Re-export backend configuration types
pub use backend::{BackendKind, InstanceHandle, OopModuleConfig};

// Re-export backend trait and implementations for convenience
pub use backends::{LocalProcessBackend, ModuleRuntimeBackend};

pub use grpc_installers::GrpcInstallerStore;
pub use host_runtime::{DbOptions, HostRuntime};
pub use module_manager::{Endpoint, InstanceState, ModuleInstance, ModuleManager, ModuleName};
pub use runner::{run, RunOptions, ShutdownOptions};
pub use system_context::SystemContext;
