//! Backend abstraction for out-of-process module management
//!
//! This module provides traits and types for spawning and managing `OoP` module instances.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;

/// The kind of backend used to spawn and manage module instances
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    LocalProcess,
    K8s,
    Static,
    Mock,
}

/// Configuration for an out-of-process module
pub struct OopModuleConfig {
    pub name: String,
    pub binary: Option<PathBuf>,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_directory: Option<String>,
    pub backend: BackendKind,
    pub version: Option<String>,
}

impl OopModuleConfig {
    pub fn new(name: impl Into<String>, backend: BackendKind) -> Self {
        Self {
            name: name.into(),
            binary: None,
            args: Vec::new(),
            env: HashMap::new(),
            working_directory: None,
            backend,
            version: None,
        }
    }
}

/// A handle to a running module instance
#[derive(Clone)]
pub struct InstanceHandle {
    pub module: String,
    pub instance_id: Uuid,
    pub backend: BackendKind,
    pub pid: Option<u32>,
    pub created_at: Instant,
}

impl std::fmt::Debug for InstanceHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstanceHandle")
            .field("module", &self.module)
            .field("instance_id", &self.instance_id)
            .field("backend", &self.backend)
            .field("pid", &self.pid)
            .field("created_at", &self.created_at)
            .finish()
    }
}

/// Trait for backends that can spawn and manage module instances
#[async_trait]
pub trait ModuleRuntimeBackend: Send + Sync {
    async fn spawn_instance(&self, cfg: &OopModuleConfig) -> Result<InstanceHandle>;
    async fn stop_instance(&self, handle: &InstanceHandle) -> Result<()>;
    async fn list_instances(&self, module: &str) -> Result<Vec<InstanceHandle>>;
}

/// Configuration passed to `OopBackend::spawn`
pub struct OopSpawnConfig {
    pub module_name: String,
    pub binary: PathBuf,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_directory: Option<String>,
}

/// A type-erased backend for spawning `OoP` modules.
///
/// This trait is used by `HostRuntime` to spawn `OoP` modules after the start phase.
#[async_trait]
pub trait OopBackend: Send + Sync {
    /// Spawn an `OoP` module instance.
    async fn spawn(&self, config: OopSpawnConfig) -> Result<()>;

    /// Shutdown all spawned instances (called during stop phase).
    async fn shutdown_all(&self);
}

pub mod local;
pub mod log_forwarder;

pub use local::LocalProcessBackend;

/// Adapter that implements `OopBackend` trait for `LocalProcessBackend`.
///
/// This allows `LocalProcessBackend` to be used by `HostRuntime` for spawning `OoP` modules.
#[async_trait]
impl OopBackend for LocalProcessBackend {
    async fn spawn(&self, config: OopSpawnConfig) -> Result<()> {
        let mut oop_config = OopModuleConfig::new(&config.module_name, BackendKind::LocalProcess);
        oop_config.binary = Some(config.binary);
        oop_config.args = config.args;
        oop_config.env = config.env;
        oop_config.working_directory = config.working_directory;

        self.spawn_instance(&oop_config).await?;
        Ok(())
    }

    async fn shutdown_all(&self) {
        // The LocalProcessBackend already handles shutdown via its cancellation token
        // when the token is triggered, it automatically stops all instances.
        // This method is a no-op because the backend's internal shutdown task handles it.
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_oop_module_config_builder() {
        let mut cfg = OopModuleConfig::new("my_module", BackendKind::LocalProcess);
        cfg.binary = Some(PathBuf::from("/usr/bin/myapp"));
        cfg.args = vec!["--port".to_owned(), "8080".to_owned()];
        cfg.env.insert("LOG_LEVEL".to_owned(), "debug".to_owned());
        cfg.version = Some("1.0.0".to_owned());

        assert_eq!(cfg.name, "my_module");
        assert_eq!(cfg.backend, BackendKind::LocalProcess);
        assert_eq!(cfg.binary, Some(PathBuf::from("/usr/bin/myapp")));
        assert_eq!(cfg.args.len(), 2);
        assert_eq!(cfg.env.len(), 1);
        assert_eq!(cfg.version, Some("1.0.0".to_owned()));
    }

    #[test]
    fn test_backend_kind_equality() {
        assert_eq!(BackendKind::LocalProcess, BackendKind::LocalProcess);
        assert_ne!(BackendKind::LocalProcess, BackendKind::K8s);
        assert_ne!(BackendKind::K8s, BackendKind::Static);
        assert_ne!(BackendKind::Static, BackendKind::Mock);
    }

    #[test]
    fn test_instance_handle_debug() {
        let instance_id = Uuid::new_v4();
        let handle = InstanceHandle {
            module: "test_module".to_owned(),
            instance_id,
            backend: BackendKind::LocalProcess,
            pid: Some(12345),
            created_at: Instant::now(),
        };

        let debug_str = format!("{handle:?}");
        assert!(debug_str.contains("test_module"));
        assert!(debug_str.contains(&instance_id.to_string()));
        assert!(debug_str.contains("LocalProcess"));
        assert!(debug_str.contains("12345"));
    }
}
