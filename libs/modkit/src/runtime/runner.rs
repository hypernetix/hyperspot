//! ModKit runtime runner.
//!
//! Supported DB modes:
//!   - `DbOptions::None` — modules get no DB in their contexts.
//!   - `DbOptions::Manager` — modules use ModuleContextBuilder to resolve per-module DbHandles.
//!
//! Design notes:
//! - We use **ModuleContextBuilder** to resolve per-module DbHandles at runtime.
//! - Phase order: **DB → init → REST → start → wait → stop**.
//! - Modules receive a fully-scoped ModuleCtx with a resolved Option<DbHandle>.
//! - Shutdown can be driven by OS signals, an external `CancellationToken`,
//!   or an arbitrary future.

use crate::context::{ConfigProvider, ModuleContextBuilder};
use crate::runtime::shutdown;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio_util::sync::CancellationToken;

/// How the runtime should provide DBs to modules.
pub enum DbOptions {
    /// No database integration. `ModuleCtx::db()` will be `None`, `db_required()` will error.
    None,
    /// Use a DbManager to handle database connections with Figment-based configuration.
    Manager(Arc<modkit_db::DbManager>),
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
    /// DB strategy: none, or DbManager.
    pub db: DbOptions,
    /// Shutdown strategy.
    pub shutdown: ShutdownOptions,
}

/// Full cycle: DB → init → rest (sync) → start → wait → stop.
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

    // Build the context builder that will resolve per-module DbHandles.
    let db_manager = match &opts.db {
        DbOptions::Manager(mgr) => Some(mgr.clone()),
        DbOptions::None => None,
    };

    let ctx_builder = ModuleContextBuilder::new(
        opts.modules_cfg.clone(),
        hub.clone(),
        cancel.clone(),
        db_manager,
    );

    // DB MIGRATION phase (system modules first)
    tracing::info!("Phase: db (before init)");
    for entry in registry.modules_by_system_priority() {
        let ctx = ctx_builder.for_module(entry.name).await?;
        if let (Some(db), Some(dbm)) = (ctx.db_optional(), entry.db.as_ref()) {
            tracing::debug!(module = entry.name, "Running DB migration");
            dbm.migrate(&db)
                .await
                .map_err(|e| crate::registry::RegistryError::DbMigrate {
                    module: entry.name,
                    source: e,
                })?;
        } else if entry.db.is_some() {
            tracing::debug!(
                module = entry.name,
                "Module has DbModule trait but no DB handle (no config)"
            );
        }
    }

    // INIT phase (system modules first)
    tracing::info!("Phase: init");
    for entry in registry.modules_by_system_priority() {
        let ctx = ctx_builder.for_module(entry.name).await?;
        entry
            .core
            .init(&ctx)
            .await
            .map_err(|e| crate::registry::RegistryError::Init {
                module: entry.name,
                source: e,
            })?;
    }

    // REST phase (synchronous router composition against ingress).
    tracing::info!("Phase: rest (sync)");
    let _router = registry
        .run_rest_phase_with_builder(&ctx_builder, axum::Router::new())
        .await?;

    // GRPC registration phase
    tracing::info!("Phase: grpc (registration)");
    registry.run_grpc_phase(&ctx_builder).await?;

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
