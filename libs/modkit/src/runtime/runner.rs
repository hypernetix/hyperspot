//! `ModKit` runtime runner.
//!
//! Supported DB modes:
//!   - `DbOptions::None` — modules get no DB in their contexts.
//!   - `DbOptions::Manager` — modules use `ModuleContextBuilder` to resolve per-module `DbHandles`.
//!
//! Design notes:
//! - We use **`ModuleContextBuilder`** to resolve per-module `DbHandles` at runtime.
//! - Phase order is orchestrated by `HostRuntime` (see `runtime/host_runtime.rs` docs).
//! - Modules receive a fully-scoped `ModuleCtx` with a resolved Option<DbHandle>.
//! - Shutdown can be driven by OS signals, an external `CancellationToken`,
//!   or an arbitrary future.
//! - Pre-registered clients can be injected into the `ClientHub` via `RunOptions::clients`.
//! - `OoP` modules are spawned after the start phase so that `grpc_hub` is already running
//!   and the real directory endpoint is known.

use crate::backends::OopBackend;
use crate::client_hub::ClientHub;
use crate::config::ConfigProvider;
use crate::registry::ModuleRegistry;
use crate::runtime::shutdown;
use crate::runtime::{DbOptions, HostRuntime};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// A type-erased client registration for injecting clients into the `ClientHub`.
///
/// This is used to pass pre-created clients (like gRPC clients) from bootstrap code
/// into the runtime's `ClientHub` before modules are initialized.
pub struct ClientRegistration {
    /// Callback that registers the client into the hub.
    register_fn: Box<dyn FnOnce(&ClientHub) + Send>,
}

impl ClientRegistration {
    /// Create a new client registration for a trait object type.
    ///
    /// # Example
    /// ```ignore
    /// let api: Arc<dyn DirectoryClient> = Arc::new(client);
    /// ClientRegistration::new::<dyn DirectoryClient>(api)
    /// ```
    pub fn new<T>(client: Arc<T>) -> Self
    where
        T: ?Sized + Send + Sync + 'static,
    {
        Self {
            register_fn: Box::new(move |hub| {
                hub.register::<T>(client);
            }),
        }
    }

    /// Execute the registration against the given hub.
    pub(crate) fn apply(self, hub: &ClientHub) {
        (self.register_fn)(hub);
    }
}

/// How the runtime should decide when to stop.
pub enum ShutdownOptions {
    /// Listen for OS signals (Ctrl+C / SIGTERM).
    Signals,
    /// An external `CancellationToken` controls the lifecycle.
    Token(CancellationToken),
    /// An arbitrary future; when it completes, we initiate shutdown.
    Future(Pin<Box<dyn Future<Output = ()> + Send>>),
}

/// Configuration for a single `OoP` module to be spawned.
#[derive(Clone)]
pub struct OopModuleSpawnConfig {
    /// Module name (e.g., "calculator")
    pub module_name: String,
    /// Path to the executable
    pub binary: PathBuf,
    /// Command-line arguments (user controls --config via execution.args in master config)
    pub args: Vec<String>,
    /// Environment variables to set
    pub env: HashMap<String, String>,
    /// Working directory for the process
    pub working_directory: Option<String>,
    /// Rendered module config JSON (for `MODKIT_MODULE_CONFIG` env var)
    pub rendered_config_json: String,
}

/// Options for spawning `OoP` modules.
pub struct OopSpawnOptions {
    /// List of `OoP` modules to spawn after the start phase
    pub modules: Vec<OopModuleSpawnConfig>,
    /// Backend for spawning `OoP` modules (e.g., `LocalProcessBackend`)
    pub backend: Box<dyn OopBackend>,
}

/// Options for running the `ModKit` runner.
pub struct RunOptions {
    /// Provider of module config sections (raw JSON by module name).
    pub modules_cfg: Arc<dyn ConfigProvider>,
    /// DB strategy: none, or `DbManager`.
    pub db: DbOptions,
    /// Shutdown strategy.
    pub shutdown: ShutdownOptions,
    /// Pre-registered clients to inject into the `ClientHub` before module initialization.
    ///
    /// This is useful for `OoP` bootstrap where clients (like `DirectoryGrpcClient`)
    /// are created before calling `run()` and need to be available in the `ClientHub`.
    pub clients: Vec<ClientRegistration>,
    /// Process-level instance ID.
    ///
    /// This is a unique identifier for this process instance, generated once at bootstrap
    /// (either in `run_oop_with_options` for `OoP` modules or in the main host).
    /// It is propagated to all modules via `ModuleCtx::instance_id()` and `SystemContext::instance_id()`.
    pub instance_id: Uuid,
    /// `OoP` module spawn configuration.
    ///
    /// These modules are spawned after the start phase, once `grpc_hub` is running
    /// and the real directory endpoint is known.
    pub oop: Option<OopSpawnOptions>,
}

/// Full cycle is orchestrated by `HostRuntime` (see `runtime/host_runtime.rs` docs).
///
/// This function is a thin wrapper around `HostRuntime` that handles shutdown signal setup
/// and then delegates all lifecycle orchestration to the `HostRuntime`.
///
/// # Errors
/// Returns an error if any lifecycle phase fails.
pub async fn run(opts: RunOptions) -> anyhow::Result<()> {
    // 1. Prepare cancellation token based on shutdown options
    let cancel = match &opts.shutdown {
        ShutdownOptions::Token(t) => t.clone(),
        _ => CancellationToken::new(),
    };

    // 2. Spawn shutdown waiter (Signals / Future) just like before
    match opts.shutdown {
        ShutdownOptions::Signals => {
            let c = cancel.clone();
            tokio::spawn(async move {
                match shutdown::wait_for_shutdown().await {
                    Ok(()) => {
                        tracing::info!(target: "", "------------------");
                        tracing::info!("shutdown: signal received");
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "shutdown: primary waiter failed; falling back to ctrl_c()"
                        );
                        let _ = tokio::signal::ctrl_c().await;
                    }
                }
                c.cancel();
            });
        }
        ShutdownOptions::Future(waiter) => {
            let c = cancel.clone();
            tokio::spawn(async move {
                waiter.await;
                tracing::info!("shutdown: external future completed");
                c.cancel();
            });
        }
        ShutdownOptions::Token(_) => {
            tracing::info!("shutdown: external token will control lifecycle");
        }
    }

    // 3. Discover modules
    let registry = ModuleRegistry::discover_and_build()?;

    // 4. Build shared ClientHub
    let hub = Arc::new(ClientHub::default());

    // 4b. Apply pre-registered clients from RunOptions
    for registration in opts.clients {
        registration.apply(&hub);
    }

    // 5. Instantiate HostRuntime
    let host = HostRuntime::new(
        registry,
        opts.modules_cfg.clone(),
        opts.db,
        hub,
        cancel.clone(),
        opts.instance_id,
        opts.oop,
    );

    // 6. Run full lifecycle
    host.run_module_phases().await
}
