use parking_lot::Mutex;

use crate::contracts::RegisterGrpcServiceFn;

/// Runtime-owned store for gRPC service installers.
///
/// This replaces the previous global static storage with a proper
/// runtime-scoped type that gets injected into the grpc_hub module.
pub struct GrpcInstallerStore {
    inner: Mutex<Vec<RegisterGrpcServiceFn>>,
}

impl GrpcInstallerStore {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Vec::new()),
        }
    }

    /// Set installers once. Fails if installers are already non-empty.
    pub fn set(&self, installers: Vec<RegisterGrpcServiceFn>) -> anyhow::Result<()> {
        let mut guard = self.inner.lock();
        if !guard.is_empty() {
            anyhow::bail!("gRPC installers already initialized");
        }
        *guard = installers;
        Ok(())
    }

    /// Consume and return all installers.
    pub fn take(&self) -> Vec<RegisterGrpcServiceFn> {
        let mut guard = self.inner.lock();
        std::mem::take(&mut *guard)
    }

    /// Check if installers are present (optional helper).
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.inner.lock().is_empty()
    }
}

impl Default for GrpcInstallerStore {
    fn default() -> Self {
        Self::new()
    }
}
