//! System Context - runtime internals exposed to system modules

use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

use crate::registry::ModuleRegistrySnapshot;
use crate::runtime::{GrpcInstallerStore, ModuleManager};

/// System-level context provided to system modules during the wiring phase.
///
/// This gives system modules access to runtime internals like the module manager
/// and gRPC installer store. Only modules with the "system" capability receive this.
///
/// Normal user modules do not see `SystemContext` - they only get `ModuleCtx` during init.
pub struct SystemContext {
    /// Process-level instance ID (shared by all modules in this process)
    instance_id: Uuid,

    /// Module instance registry and manager
    pub module_manager: Arc<ModuleManager>,

    /// gRPC service installer store
    pub grpc_installers: Arc<GrpcInstallerStore>,

    /// Static snapshot of compiled-in module registry
    pub registry_snapshot: Arc<ModuleRegistrySnapshot>,

    /// Names of modules configured as out-of-process
    pub oop_module_names: Arc<HashSet<String>>,
}

impl SystemContext {
    /// Create a new system context from runtime components
    pub fn new(
        instance_id: Uuid,
        module_manager: Arc<ModuleManager>,
        grpc_installers: Arc<GrpcInstallerStore>,
        registry_snapshot: Arc<ModuleRegistrySnapshot>,
        oop_module_names: Arc<HashSet<String>>,
    ) -> Self {
        Self {
            instance_id,
            module_manager,
            grpc_installers,
            registry_snapshot,
            oop_module_names,
        }
    }

    /// Returns the process-level instance ID.
    ///
    /// This is a unique identifier for this process instance, shared by all modules
    /// in the same process. It is generated once at bootstrap.
    #[inline]
    #[must_use]
    pub fn instance_id(&self) -> Uuid {
        self.instance_id
    }
}
