//! ModKit runtime runner.
//!
//! Supported DB modes:
//!   - `DbOptions::None` — modules get no DB in their contexts.
//!   - `DbOptions::Manager` — modules use ModuleContextBuilder to resolve per-module DbHandles.
//!
//! Design notes:
//! - We use **ModuleContextBuilder** to resolve per-module DbHandles at runtime.
//! - Phase order: **system_wire → DB → init → REST → gRPC → start → wait → stop**.
//! - Modules receive a fully-scoped ModuleCtx with a resolved Option<DbHandle>.
//! - Shutdown can be driven by OS signals, an external `CancellationToken`,
//!   or an arbitrary future.

use crate::client_hub::ClientHub;
use crate::registry::ModuleRegistry;
use crate::runtime::shutdown;
use crate::runtime::{DbOptions, HostRuntime};
use modkit_bootstrap::ConfigProvider;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio_util::sync::CancellationToken;

/// How the runtime should decide when to stop.
pub enum ShutdownOptions {
    /// Listen for OS signals (Ctrl+C / SIGTERM).
    Signals,
    /// An external `CancellationToken` controls the lifecycle.
    Token(CancellationToken),
    /// An arbitrary future; when it completes, we initiate shutdown.
    Future(Pin<Box<dyn Future<Output = ()> + Send>>),
}

/// Options for running the ModKit runner.
pub struct RunOptions {
    /// Provider of module config sections (raw JSON by module name).
    pub modules_cfg: Arc<dyn ConfigProvider>,
    /// DB strategy: none, or DbManager.
    pub db: DbOptions,
    /// Shutdown strategy.
    pub shutdown: ShutdownOptions,
}

/// Full cycle: system_wire → DB → init → REST → gRPC → start → wait → stop.
///
/// This function is a thin wrapper around HostRuntime that handles shutdown signal setup
/// and then delegates all lifecycle orchestration to the HostRuntime.
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

    // 5. Instantiate HostRuntime
    let host = HostRuntime::new(registry, opts.modules_cfg, opts.db, hub, cancel.clone());

    // 6. Run full lifecycle
    host.run_full_cycle().await
}
