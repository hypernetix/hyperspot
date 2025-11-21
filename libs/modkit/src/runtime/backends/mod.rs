//! Backend abstraction for out-of-process module management

use anyhow::Result;
use async_trait::async_trait;

use crate::runtime::backend::OopModuleConfig;
use crate::runtime::{InstanceHandle, ModuleName};

/// Trait for backends that can spawn and manage module instances
#[async_trait]
pub trait ModuleRuntimeBackend: Send + Sync {
    async fn spawn_instance(&self, cfg: &OopModuleConfig) -> Result<InstanceHandle>;

    async fn stop_instance(&self, handle: &InstanceHandle) -> Result<()>;

    async fn list_instances(&self, module: ModuleName) -> Result<Vec<InstanceHandle>>;
}

// Local backend submodule
pub mod local;
pub use local::LocalProcessBackend;
