//! System Context - runtime internals exposed to system modules

use std::sync::Arc;

use crate::runtime::{GrpcInstallerStore, ModuleManager};

/// System-level context provided to system modules during the wiring phase.
///
/// This gives system modules access to runtime internals like the module manager
/// and gRPC installer store. Only modules with the "system" capability receive this.
///
/// Normal user modules do not see SystemContext - they only get ModuleCtx during init.
pub struct SystemContext {
    /// Module instance registry and manager
    pub module_manager: Arc<ModuleManager>,

    /// gRPC service installer store
    pub grpc_installers: Arc<GrpcInstallerStore>,
}

impl SystemContext {
    /// Create a new system context from runtime components
    pub fn new(
        module_manager: Arc<ModuleManager>,
        grpc_installers: Arc<GrpcInstallerStore>,
    ) -> Self {
        Self {
            module_manager,
            grpc_installers,
        }
    }
}
