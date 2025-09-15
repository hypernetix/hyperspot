//! ModKit runtime runner.
//!
//! Supported DB modes:
//!   - `DbOptions::None` — modules get no DB in their contexts.
//!   - `DbOptions::PerModuleFactory` — a factory is invoked per module name,
//!     returning an optional handle for that specific module.
//!
//! Design notes:
//! - We build **one stable ModuleCtx** (`base_ctx`) and reuse it across all phases
//!   (init → db → rest → start → wait → stop). Module-specific DB handles are
//!   injected by the registry when calling `run_init_phase_with_factory` /
//!   `run_db_phase_with_factory`, so the base context stays immutable and simple.
//! - Shutdown can be driven by OS signals, an external `CancellationToken`,
//!   or an arbitrary future.

use crate::context::{ConfigProvider, ModuleCtxBuilder};
use crate::runtime::shutdown;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio_util::sync::CancellationToken;

/// Per-module database factory: given a module name, returns a DB handle for that
/// module (or `None` if the module has no DB). The factory decides where to source
/// configuration (global+module cfg, env, etc.).
pub type PerModuleDbFactory = Box<
    dyn Fn(&str) -> Pin<Box<dyn Future<Output = anyhow::Result<Option<Arc<db::DbHandle>>>> + Send>>
        + Send
        + Sync,
>;

/// How the runtime should provide DBs to modules.
pub enum DbOptions {
    /// No database integration. `ModuleCtx::db()` will be `None`, `db_required()` will error.
    None,
    /// Use a per-module factory to construct a handle per module.
    PerModuleFactory(PerModuleDbFactory),
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

/// Options for running the ModKit runner.
pub struct RunOptions {
    /// Provider of module config sections (raw JSON by module name).
    pub modules_cfg: Arc<dyn ConfigProvider>,
    /// DB strategy: none, or per-module factory.
    pub db: DbOptions,
    /// Shutdown strategy.
    pub shutdown: ShutdownOptions,
}

/// Full cycle: init → db → rest (sync) → start → wait → stop.
pub async fn run(opts: RunOptions) -> anyhow::Result<()> {
    // Stable components shared across all phases.
    let hub = Arc::new(crate::client_hub::ClientHub::default());
    let cancel = match &opts.shutdown {
        ShutdownOptions::Token(t) => t.clone(),
        _ => CancellationToken::new(),
    };

    // Spawn the shutdown waiter according to the chosen strategy.
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
                        // Cross-platform fallback.
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
            // External owner controls lifecycle; nothing to spawn.
            tracing::info!("shutdown: external token will control lifecycle");
        }
    }

    // Discover modules upfront.
    let registry = crate::registry::ModuleRegistry::discover_and_build()?;

    // Build ONE stable base context used across all phases.
    let base_ctx = ModuleCtxBuilder::new(cancel.clone())
        .with_client_hub(hub.clone())
        .with_config_provider(opts.modules_cfg.clone())
        .build();

    // INIT phase
    tracing::info!("Phase: init");
    match &opts.db {
        DbOptions::PerModuleFactory(factory) => {
            // Per-module: registry builds a context per module and injects its DB (if any).
            registry
                .run_init_phase_with_factory(&base_ctx, factory)
                .await?;
        }
        DbOptions::None => {
            // No DB at all — just run init with the shared base context.
            registry.run_init_phase(&base_ctx).await?;
        }
    }

    // DB MIGRATION phase
    match &opts.db {
        DbOptions::PerModuleFactory(factory) => {
            tracing::info!("Phase: db (per-module)");
            registry.run_db_phase_with_factory(factory).await?;
        }
        DbOptions::None => {
            // No DB — nothing to migrate.
        }
    }

    // REST phase (synchronous router composition against ingress).
    tracing::info!("Phase: rest (sync)");
    let _ = registry.run_rest_phase(&base_ctx, axum::Router::new())?;

    // START phase
    tracing::info!("Phase: start");
    registry.run_start_phase(cancel.clone()).await?;

    // WAIT
    cancel.cancelled().await;

    // STOP phase
    tracing::info!("Phase: stop");
    registry.run_stop_phase(cancel).await?;
    Ok(())
}

#[cfg(feature = "hs-runtime")]
#[allow(dead_code)]
pub async fn run_with_hyperspot_signals(mut opts: RunOptions) -> anyhow::Result<()> {
    opts.shutdown = ShutdownOptions::Signals;
    run(opts).await
}
