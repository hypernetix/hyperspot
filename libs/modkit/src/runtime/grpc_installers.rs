use parking_lot::Mutex;

use crate::contracts::RegisterGrpcServiceFn;

/// Installers for a specific module (module name + service installers).
#[derive(Default)]
pub struct ModuleInstallers {
    pub module_name: String,
    pub installers: Vec<RegisterGrpcServiceFn>,
}

/// Grouped installers for all modules in the process.
#[derive(Default)]
pub struct GrpcInstallerData {
    pub modules: Vec<ModuleInstallers>,
}

/// Runtime-owned store for gRPC service installers.
///
/// This replaces the previous global static storage with a proper
/// runtime-scoped type that gets injected into the grpc_hub module.
pub struct GrpcInstallerStore {
    inner: Mutex<Option<GrpcInstallerData>>,
}

impl GrpcInstallerStore {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    /// Set installers once. Fails if already initialized.
    pub fn set(&self, data: GrpcInstallerData) -> anyhow::Result<()> {
        let mut guard = self.inner.lock();
        if guard.is_some() {
            anyhow::bail!("gRPC installers already initialized");
        }
        *guard = Some(data);
        Ok(())
    }

    /// Consume and return all installers grouped by module.
    pub fn take(&self) -> Option<GrpcInstallerData> {
        let mut guard = self.inner.lock();
        guard.take()
    }

    /// Check if installers are present (optional helper).
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.inner.lock().is_none()
    }
}

impl Default for GrpcInstallerStore {
    fn default() -> Self {
        Self::new()
    }
}
