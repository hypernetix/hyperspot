//! Local process backend implementation

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::{Child, Command};
use uuid::Uuid;

use super::super::backend::{BackendKind, OopModuleConfig};
use super::ModuleRuntimeBackend;
use crate::runtime::{InstanceHandle, ModuleName};

/// Internal representation of a local process instance
struct LocalInstance {
    handle: InstanceHandle,
    child: Child,
}

/// Backend that spawns modules as local child processes and manages their lifecycle
pub struct LocalProcessBackend {
    instances: Arc<RwLock<HashMap<String, LocalInstance>>>,
}

impl LocalProcessBackend {
    pub fn new() -> Self {
        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for LocalProcessBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ModuleRuntimeBackend for LocalProcessBackend {
    async fn spawn_instance(&self, cfg: &OopModuleConfig) -> Result<InstanceHandle> {
        // Verify backend kind
        if cfg.backend != BackendKind::LocalProcess {
            bail!(
                "LocalProcessBackend can only spawn LocalProcess instances, got {:?}",
                cfg.backend
            );
        }

        // Ensure binary is set
        let binary = cfg
            .binary
            .as_ref()
            .context("binary path must be set for LocalProcess backend")?;

        // Generate unique instance ID using UUID v7
        let instance_id = Uuid::now_v7().to_string();

        // Build command
        let mut cmd = Command::new(binary);
        cmd.args(&cfg.args);
        cmd.envs(&cfg.env);

        // Spawn the process
        let child = cmd
            .spawn()
            .context(format!("failed to spawn process: {:?}", binary))?;

        // Get PID
        let pid = child.id();

        // Create handle
        let handle = InstanceHandle {
            module: cfg.name,
            instance_id: instance_id.clone(),
            backend: BackendKind::LocalProcess,
            pid,
            created_at: std::time::Instant::now(),
        };

        // Store in instances map
        {
            let mut instances = self.instances.write();
            instances.insert(
                instance_id.clone(),
                LocalInstance {
                    handle: handle.clone(),
                    child,
                },
            );
        }

        Ok(handle)
    }

    async fn stop_instance(&self, handle: &InstanceHandle) -> Result<()> {
        // Take ownership of the LocalInstance so we can await on child without holding the lock.
        let local = {
            let mut instances = self.instances.write();
            instances.remove(&handle.instance_id)
        };

        if let Some(mut local) = local {
            // Try to kill the process. If the process has already exited or kill fails,
            // we treat it as best effort and do not propagate an error.
            if let Some(pid) = local.child.id() {
                tracing::debug!(
                    module = %handle.module,
                    instance_id = %handle.instance_id,
                    pid = pid,
                    "Stopping local process instance"
                );
            }

            if let Err(e) = local.child.kill().await {
                // If the child is already dead, treat it as non fatal.
                tracing::warn!(
                    module = %handle.module,
                    instance_id = %handle.instance_id,
                    error = %e,
                    "Failed to kill local process instance"
                );
            }
        } else {
            // Instance is not tracked anymore, treat as no op
            tracing::debug!(
                module = %handle.module,
                instance_id = %handle.instance_id,
                "stop_instance called for unknown instance, ignoring"
            );
        }

        Ok(())
    }

    async fn list_instances(&self, module: ModuleName) -> Result<Vec<InstanceHandle>> {
        let instances = self.instances.read();

        let result = instances
            .values()
            .filter(|inst| inst.handle.module == module)
            .map(|inst| inst.handle.clone())
            .collect();

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Instant;

    #[tokio::test]
    async fn test_spawn_instance_requires_binary() {
        let backend = LocalProcessBackend::new();
        let cfg = OopModuleConfig::new("test_module", BackendKind::LocalProcess);

        let result = backend.spawn_instance(&cfg).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("binary path must be set"));
    }

    #[tokio::test]
    async fn test_spawn_instance_requires_correct_backend() {
        let backend = LocalProcessBackend::new();
        let mut cfg = OopModuleConfig::new("test_module", BackendKind::K8s);
        cfg.binary = Some(PathBuf::from("/bin/echo"));

        let result = backend.spawn_instance(&cfg).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("can only spawn LocalProcess"));
    }

    #[tokio::test]
    async fn test_spawn_list_stop_lifecycle() {
        let backend = LocalProcessBackend::new();

        // Create config with a valid binary that exists on most systems
        let mut cfg = OopModuleConfig::new("test_module", BackendKind::LocalProcess);

        // Use a simple command that exists cross-platform
        #[cfg(windows)]
        let binary = PathBuf::from("C:\\Windows\\System32\\cmd.exe");
        #[cfg(not(windows))]
        let binary = PathBuf::from("/bin/sleep");

        cfg.binary = Some(binary);
        cfg.args = vec!["10".to_string()]; // sleep for 10 seconds

        // Spawn instance
        let handle = backend
            .spawn_instance(&cfg)
            .await
            .expect("should spawn instance");

        assert_eq!(handle.module, "test_module");
        assert!(!handle.instance_id.is_empty());
        assert_eq!(handle.backend, BackendKind::LocalProcess);

        // List instances
        let instances = backend
            .list_instances("test_module")
            .await
            .expect("should list instances");
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].module, "test_module");
        assert_eq!(instances[0].instance_id, handle.instance_id);

        // Stop instance
        backend
            .stop_instance(&handle)
            .await
            .expect("should stop instance");

        // Verify it's removed
        let instances = backend
            .list_instances("test_module")
            .await
            .expect("should list instances");
        assert_eq!(instances.len(), 0);
    }

    #[tokio::test]
    async fn test_list_instances_filters_by_module() {
        let backend = LocalProcessBackend::new();

        #[cfg(windows)]
        let binary = PathBuf::from("C:\\Windows\\System32\\cmd.exe");
        #[cfg(not(windows))]
        let binary = PathBuf::from("/bin/sleep");

        // Spawn instance for module_a
        let mut cfg_a = OopModuleConfig::new("module_a", BackendKind::LocalProcess);
        cfg_a.binary = Some(binary.clone());
        cfg_a.args = vec!["10".to_string()];

        let handle_a = backend
            .spawn_instance(&cfg_a)
            .await
            .expect("should spawn module_a");

        // Spawn instance for module_b
        let mut cfg_b = OopModuleConfig::new("module_b", BackendKind::LocalProcess);
        cfg_b.binary = Some(binary);
        cfg_b.args = vec!["10".to_string()];

        let handle_b = backend
            .spawn_instance(&cfg_b)
            .await
            .expect("should spawn module_b");

        // List module_a instances
        let instances_a = backend
            .list_instances("module_a")
            .await
            .expect("should list module_a");
        assert_eq!(instances_a.len(), 1);
        assert_eq!(instances_a[0].module, "module_a");

        // List module_b instances
        let instances_b = backend
            .list_instances("module_b")
            .await
            .expect("should list module_b");
        assert_eq!(instances_b.len(), 1);
        assert_eq!(instances_b[0].module, "module_b");

        // Clean up
        backend.stop_instance(&handle_a).await.ok();
        backend.stop_instance(&handle_b).await.ok();
    }

    #[tokio::test]
    async fn test_stop_nonexistent_instance() {
        let backend = LocalProcessBackend::new();
        let handle = InstanceHandle {
            module: "test_module",
            instance_id: "nonexistent".to_string(),
            backend: BackendKind::LocalProcess,
            pid: None,
            created_at: Instant::now(),
        };

        // Should not error even if instance doesn't exist
        let result = backend.stop_instance(&handle).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_instances_empty() {
        let backend = LocalProcessBackend::new();
        let instances = backend
            .list_instances("nonexistent_module")
            .await
            .expect("should list instances");
        assert_eq!(instances.len(), 0);
    }
}
