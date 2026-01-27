//! Local process backend implementation

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::log_forwarder::{StreamKind, spawn_stream_forwarder};
use super::{BackendKind, InstanceHandle, ModuleRuntimeBackend, OopModuleConfig};

/// Grace period before force-killing processes on shutdown
const SHUTDOWN_GRACE_PERIOD: Duration = Duration::from_secs(5);

/// Grace period for individual instance stop
const INSTANCE_STOP_GRACE_PERIOD: Duration = Duration::from_secs(2);

/// Timeout for waiting on forwarder tasks during shutdown
const FORWARDER_DRAIN_TIMEOUT: Duration = Duration::from_millis(100);

/// Send graceful termination signal to a child process.
///
/// # Returns
/// - `true` if signal was successfully sent
/// - `false` if:
///   - Process has no PID (already exited)
///   - PID cannot be converted to i32 (extremely rare: PID > 2,147,483,647)
///   - Signal delivery fails
///
/// On Unix: Sends SIGTERM which the process can handle for cleanup.
/// On Windows: Returns false since there's no reliable graceful termination
/// method for console applications.
#[cfg(unix)]
fn send_terminate_signal(child: &Child) -> bool {
    use nix::sys::signal::{Signal, kill};
    use nix::unistd::Pid;

    let Some(pid) = child.id() else {
        return false;
    };

    let Ok(pid_i32) = i32::try_from(pid) else {
        tracing::warn!(
            pid = pid,
            "Failed to convert PID to i32, cannot send SIGTERM (PID exceeds i32::MAX: {})",
            i32::MAX
        );
        return false;
    };

    kill(Pid::from_raw(pid_i32), Signal::SIGTERM).is_ok()
}

/// Send graceful termination signal to a child process.
///
/// # Returns
/// - `false` always on Windows (no reliable graceful termination available)
///
/// On Windows there's no reliable SIGTERM equivalent for console applications.
/// We return false to indicate that graceful termination is not available,
/// and the caller should proceed with force kill.
#[cfg(windows)]
fn send_terminate_signal(_child: &Child) -> bool {
    false
}

/// Stop a child process with graceful termination and timeout.
///
/// 1. Sends SIGTERM (Unix) via `send_terminate_signal`
/// 2. Waits for process exit within `grace` period
/// 3. On timeout: force kills the process
async fn stop_child_with_grace(
    child: &mut Child,
    handle: &InstanceHandle,
    grace: Duration,
    context: &str,
) {
    let pid = child.id();
    let sent = send_terminate_signal(child);

    // Log with module context if termination signal failed
    if !sent && pid.is_some() {
        tracing::debug!(
            module = %handle.module,
            instance_id = %handle.instance_id,
            pid = ?pid,
            "{context}: graceful termination not available, will force kill"
        );
    }

    tracing::debug!(
        module = %handle.module,
        instance_id = %handle.instance_id,
        pid = ?pid,
        graceful = sent,
        "{context}: sent termination signal"
    );

    match tokio::time::timeout(grace, child.wait()).await {
        Ok(Ok(status)) => {
            tracing::debug!(
                module = %handle.module,
                instance_id = %handle.instance_id,
                status = ?status,
                "{context}: process exited gracefully"
            );
        }
        Ok(Err(e)) => {
            tracing::warn!(
                module = %handle.module,
                instance_id = %handle.instance_id,
                error = %e,
                "{context}: failed to wait for process"
            );
        }
        Err(_) => {
            tracing::debug!(
                module = %handle.module,
                instance_id = %handle.instance_id,
                "{context}: grace period expired, force killing"
            );
            if let Err(e) = child.kill().await {
                tracing::warn!(
                    module = %handle.module,
                    instance_id = %handle.instance_id,
                    error = %e,
                    "{context}: failed to force kill"
                );
            }
        }
    }
}

/// Wait for a log forwarder task to finish with timeout.
async fn wait_forwarder(handle: Option<JoinHandle<()>>) {
    if let Some(h) = handle {
        let _ = tokio::time::timeout(FORWARDER_DRAIN_TIMEOUT, h).await;
    }
}

/// Internal representation of a local process instance
struct LocalInstance {
    handle: InstanceHandle,
    child: Child,
    /// Task handle for stdout log forwarder
    stdout_forwarder: Option<JoinHandle<()>>,
    /// Task handle for stderr log forwarder
    stderr_forwarder: Option<JoinHandle<()>>,
}

/// Map key type for instances - uses Uuid directly
type InstanceMap = HashMap<Uuid, LocalInstance>;

/// Backend that spawns modules as local child processes and manages their lifecycle.
///
/// When the cancellation token is triggered, the backend will:
/// 1. Send termination signal to all processes (SIGTERM on Unix, `TerminateProcess` on Windows)
/// 2. Wait up to 5 seconds for graceful shutdown
/// 3. Force kill any remaining processes
pub struct LocalProcessBackend {
    instances: Arc<RwLock<InstanceMap>>,
    cancel: CancellationToken,
}

impl LocalProcessBackend {
    /// Create a new `LocalProcessBackend` with the given cancellation token.
    ///
    /// When the token is cancelled, all spawned processes will be gracefully stopped.
    #[must_use]
    pub fn new(cancel: CancellationToken) -> Self {
        let backend = Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            cancel: cancel.clone(),
        };

        // Spawn background task to handle shutdown
        let instances = Arc::clone(&backend.instances);
        tokio::spawn(async move {
            cancel.cancelled().await;
            tracing::info!("LocalProcessBackend: shutdown signal received, stopping all processes");
            Self::shutdown_all_instances(instances).await;
        });

        backend
    }

    /// Gracefully stop all tracked instances with timeout.
    async fn shutdown_all_instances(instances: Arc<RwLock<InstanceMap>>) {
        let mut all_instances: Vec<LocalInstance> = {
            let mut guard = instances.write();
            guard.drain().map(|(_, inst)| inst).collect()
        };

        if all_instances.is_empty() {
            return;
        }

        tracing::info!(count = all_instances.len(), "Stopping OoP module processes");

        // Stop all processes with grace period
        for inst in &mut all_instances {
            stop_child_with_grace(
                &mut inst.child,
                &inst.handle,
                SHUTDOWN_GRACE_PERIOD,
                "shutdown",
            )
            .await;
        }

        // Wait for forwarders to drain
        for inst in all_instances {
            wait_forwarder(inst.stdout_forwarder).await;
            wait_forwarder(inst.stderr_forwarder).await;
        }

        tracing::info!("All OoP module processes stopped");
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
            .context("executable_path must be set for LocalProcess backend")?;

        // Generate unique instance ID using UUID v7
        let instance_id = Uuid::now_v7();

        // Build command
        let mut cmd = Command::new(binary);
        cmd.args(&cfg.args);
        cmd.envs(&cfg.env);

        // Pipe stdout/stderr for log forwarding
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Set working directory if specified
        if let Some(ref working_dir) = cfg.working_directory {
            let path = Path::new(working_dir);
            if path.exists() && path.is_dir() {
                cmd.current_dir(path);
            } else {
                tracing::warn!(
                    module = %cfg.name,
                    working_dir = %working_dir,
                    "Working directory does not exist or is not a directory, using current dir"
                );
            }
        }

        // Spawn the process
        let mut child = cmd
            .spawn()
            .with_context(|| format!("failed to spawn process: {}", binary.display()))?;

        // Get PID
        let pid = child.id();

        // Spawn log forwarder tasks for stdout/stderr with cancellation support
        let module_name = cfg.name.clone();
        let cancel = self.cancel.clone();
        let stdout_forwarder = child.stdout.take().map(|stdout| {
            spawn_stream_forwarder(
                stdout,
                module_name.clone(),
                instance_id,
                cancel.clone(),
                StreamKind::Stdout,
            )
        });
        let stderr_forwarder = child.stderr.take().map(|stderr| {
            spawn_stream_forwarder(
                stderr,
                module_name.clone(),
                instance_id,
                cancel.clone(),
                StreamKind::Stderr,
            )
        });

        tracing::info!(
            module = %cfg.name,
            instance_id = %instance_id,
            pid = ?pid,
            "Spawned OoP module with log forwarding"
        );

        // Create handle
        let handle = InstanceHandle {
            module: cfg.name.clone(),
            instance_id,
            backend: BackendKind::LocalProcess,
            pid,
            created_at: std::time::Instant::now(),
        };

        // Store in instances map
        {
            let mut instances = self.instances.write();
            instances.insert(
                instance_id,
                LocalInstance {
                    handle: handle.clone(),
                    child,
                    stdout_forwarder,
                    stderr_forwarder,
                },
            );
        }

        Ok(handle)
    }

    async fn stop_instance(&self, handle: &InstanceHandle) -> Result<()> {
        let local = {
            let mut instances = self.instances.write();
            instances.remove(&handle.instance_id)
        };

        if let Some(mut local) = local {
            stop_child_with_grace(
                &mut local.child,
                &local.handle,
                INSTANCE_STOP_GRACE_PERIOD,
                "stop_instance",
            )
            .await;

            // we do not await forwarders here, they'll stop on their own via CancellationToken and pipe close;
            // shutdown_all_instances handles draining for global shutdown
        } else {
            tracing::debug!(
                module = %handle.module,
                instance_id = %handle.instance_id,
                "stop_instance called for unknown instance, ignoring"
            );
        }

        Ok(())
    }

    async fn list_instances(&self, module: &str) -> Result<Vec<InstanceHandle>> {
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

    fn test_backend() -> LocalProcessBackend {
        LocalProcessBackend::new(CancellationToken::new())
    }

    #[tokio::test]
    async fn test_spawn_instance_requires_binary() {
        let backend = test_backend();
        let cfg = OopModuleConfig::new("test_module", BackendKind::LocalProcess);

        let result = backend.spawn_instance(&cfg).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("executable_path must be set")
        );
    }

    #[tokio::test]
    async fn test_spawn_instance_requires_correct_backend() {
        let backend = test_backend();
        let mut cfg = OopModuleConfig::new("test_module", BackendKind::K8s);
        cfg.binary = Some(PathBuf::from("/bin/echo"));

        let result = backend.spawn_instance(&cfg).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("can only spawn LocalProcess")
        );
    }

    #[tokio::test]
    async fn test_spawn_list_stop_lifecycle() {
        let backend = test_backend();

        // Create config with a valid binary that exists on most systems
        let mut cfg = OopModuleConfig::new("test_module", BackendKind::LocalProcess);

        // Use a simple command that exists cross-platform
        #[cfg(windows)]
        let binary = PathBuf::from("C:\\Windows\\System32\\cmd.exe");
        #[cfg(not(windows))]
        let binary = PathBuf::from("/bin/sleep");

        cfg.binary = Some(binary);
        cfg.args = vec!["10".to_owned()]; // sleep for 10 seconds

        // Spawn instance
        let handle = backend
            .spawn_instance(&cfg)
            .await
            .expect("should spawn instance");

        assert_eq!(handle.module, "test_module");
        assert!(!handle.instance_id.is_nil());
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
        let backend = test_backend();

        #[cfg(windows)]
        let binary = PathBuf::from("C:\\Windows\\System32\\cmd.exe");
        #[cfg(not(windows))]
        let binary = PathBuf::from("/bin/sleep");

        // Spawn instance for module_a
        let mut cfg_a = OopModuleConfig::new("module_a", BackendKind::LocalProcess);
        cfg_a.binary = Some(binary.clone());
        cfg_a.args = vec!["10".to_owned()];

        let handle_a = backend
            .spawn_instance(&cfg_a)
            .await
            .expect("should spawn module_a");

        // Spawn instance for module_b
        let mut cfg_b = OopModuleConfig::new("module_b", BackendKind::LocalProcess);
        cfg_b.binary = Some(binary);
        cfg_b.args = vec!["10".to_owned()];

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
        let backend = test_backend();
        let handle = InstanceHandle {
            module: "test_module".to_owned(),
            instance_id: Uuid::new_v4(),
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
        let backend = test_backend();
        let instances = backend
            .list_instances("nonexistent_module")
            .await
            .expect("should list instances");
        assert_eq!(instances.len(), 0);
    }

    mod send_terminate_signal_tests {
        #[cfg(unix)]
        use {super::send_terminate_signal, std::time::Duration};

        #[cfg(unix)]
        #[tokio::test]
        async fn test_send_terminate_signal_to_valid_process() {
            // Spawn a long-running process using sh -c 'sleep 30'
            // This works on all Unix systems (Linux, macOS, BSD)
            let mut cmd = tokio::process::Command::new("/bin/sh");
            cmd.args(["-c", "sleep 30"]);

            let mut child = cmd.spawn().expect("should spawn test process");

            // Send termination signal
            let result = send_terminate_signal(&child);

            // Should return true indicating signal was sent
            assert!(result, "Should successfully send SIGTERM to valid process");

            // Wait briefly for graceful shutdown
            tokio::time::timeout(Duration::from_millis(100), child.wait())
                .await
                .expect("process should exit within timeout")
                .expect("wait should succeed");
        }

        #[cfg(unix)]
        #[tokio::test]
        async fn test_send_terminate_signal_to_exited_process() {
            // Spawn a process that exits immediately using sh -c 'exit 0'
            // This works on all Unix systems (Linux, macOS, BSD)
            let mut cmd = tokio::process::Command::new("/bin/sh");
            cmd.args(["-c", "exit 0"]);
            let mut child = cmd.spawn().expect("should spawn test process");

            // Wait for it to exit
            tokio::time::timeout(Duration::from_millis(100), child.wait())
                .await
                .expect("process should exit within timeout")
                .expect("wait should succeed");

            // Try to send termination signal to exited process
            let result = send_terminate_signal(&child);

            // Should return false because PID is no longer available
            assert!(!result, "Should return false for already-exited process");
        }

        #[cfg(unix)]
        #[test]
        fn test_pid_conversion_edge_case_documentation() {
            // This test documents the edge case behavior for PIDs > i32::MAX
            // In practice, this is extremely rare as it would require:
            // 1. System uptime of weeks/months without reboot
            // 2. PID counter to wrap around multiple times
            // 3. Specific kernel configuration

            // The maximum value a u32 PID can have
            let max_u32_pid: u32 = u32::MAX;

            // This would fail to convert to i32
            let result = i32::try_from(max_u32_pid);
            assert!(result.is_err(), "u32::MAX should not fit in i32");

            // Our code handles this by logging a warning and returning false
            // preventing the dangerous unwrap_or(0) that would signal PID 0
        }

        #[cfg(unix)]
        #[test]
        fn test_pid_conversion_normal_range() {
            // Test that normal PIDs convert successfully
            let normal_pid: u32 = 12345;
            let result = i32::try_from(normal_pid);
            assert!(result.is_ok(), "Normal PID should convert to i32");
            assert_eq!(result.unwrap(), 12345);
        }
    }
}
