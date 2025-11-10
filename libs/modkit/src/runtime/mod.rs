mod backend;
mod instance_directory;
mod runner;
mod shutdown;

#[cfg(test)]
mod tests;

// Backend module with trait and implementations
pub mod backends;

// Re-export backend configuration types
pub use backend::{BackendKind, InstanceHandle, OopModuleConfig};

// Re-export backend trait and implementations for convenience
pub use backends::{LocalProcessBackend, ModuleRuntimeBackend};

pub use instance_directory::{
    get_global_instance_directory, set_global_instance_directory, Endpoint, InstanceDirectory,
    ModuleInstance, ModuleName,
};
pub use runner::{run, DbOptions, RunOptions, ShutdownOptions};
