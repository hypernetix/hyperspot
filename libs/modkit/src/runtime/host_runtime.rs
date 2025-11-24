//! Host Runtime - orchestrates the full ModKit lifecycle
//!
//! This module contains the HostRuntime type that owns and coordinates
//! the execution of all lifecycle phases: system_wire → DB → init → REST → gRPC → start → wait → stop.

use axum::Router;
use std::collections::HashSet;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use crate::client_hub::ClientHub;
use crate::config::ConfigProvider;
use crate::context::ModuleContextBuilder;
use crate::contracts::RegisterGrpcServiceFn;
use crate::registry::{ModuleRegistry, RegistryError};
use crate::runtime::{GrpcInstallerStore, ModuleManager, SystemContext};

/// How the runtime should provide DBs to modules.
#[derive(Clone)]
pub enum DbOptions {
    /// No database integration. `ModuleCtx::db()` will be `None`, `db_required()` will error.
    None,
    /// Use a DbManager to handle database connections with Figment-based configuration.
    Manager(Arc<modkit_db::DbManager>),
}

/// HostRuntime owns the lifecycle orchestration for ModKit.
///
/// It encapsulates all runtime state and drives modules through the full lifecycle:
/// system_wire → DB → init → REST → gRPC → start → wait → stop.
pub struct HostRuntime {
    registry: ModuleRegistry,
    ctx_builder: ModuleContextBuilder,
    module_manager: Arc<ModuleManager>,
    grpc_installers: Arc<GrpcInstallerStore>,
    #[allow(dead_code)]
    client_hub: Arc<ClientHub>,
    cancel: CancellationToken,
    #[allow(dead_code)]
    db_options: DbOptions,
}

impl HostRuntime {
    /// Create a new HostRuntime instance.
    ///
    /// This prepares all runtime components but does not start any lifecycle phases.
    pub fn new(
        registry: ModuleRegistry,
        modules_cfg: Arc<dyn ConfigProvider>,
        db_options: DbOptions,
        client_hub: Arc<ClientHub>,
        cancel: CancellationToken,
    ) -> Self {
        // Create runtime-owned components for system modules
        let module_manager = Arc::new(ModuleManager::new());
        let grpc_installers = Arc::new(GrpcInstallerStore::new());

        // Build the context builder that will resolve per-module DbHandles
        let db_manager = match &db_options {
            DbOptions::Manager(mgr) => Some(mgr.clone()),
            DbOptions::None => None,
        };

        let ctx_builder =
            ModuleContextBuilder::new(modules_cfg, client_hub.clone(), cancel.clone(), db_manager);

        Self {
            registry,
            ctx_builder,
            module_manager,
            grpc_installers,
            client_hub,
            cancel,
            db_options,
        }
    }

    /// SYSTEM WIRING phase: wire runtime internals into system modules.
    ///
    /// This phase runs before init and only for modules with the "system" capability.
    pub async fn wire_system(&self) -> Result<(), RegistryError> {
        tracing::info!("Phase: system_wire");

        let sys_ctx = SystemContext::new(
            Arc::clone(&self.module_manager),
            Arc::clone(&self.grpc_installers),
        );

        for entry in self.registry.modules() {
            if entry.is_system {
                if let Some(sys_mod) = entry.core.as_system_module() {
                    tracing::debug!(module = entry.name, "Wiring system context");
                    sys_mod.wire_system(&sys_ctx);
                }
            }
        }

        Ok(())
    }

    /// DB MIGRATION phase: run migrations for all modules with DB capability.
    ///
    /// Runs before init, with system modules processed first.
    async fn run_db_phase(&self) -> Result<(), RegistryError> {
        tracing::info!("Phase: db (before init)");

        for entry in self.registry.modules_by_system_priority() {
            let ctx = self.ctx_builder.for_module(entry.name).await.map_err(|e| {
                RegistryError::DbMigrate {
                    module: entry.name,
                    source: e,
                }
            })?;

            if let (Some(db), Some(dbm)) = (ctx.db_optional(), entry.db.as_ref()) {
                tracing::debug!(module = entry.name, "Running DB migration");
                dbm.migrate(&db)
                    .await
                    .map_err(|e| RegistryError::DbMigrate {
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

        Ok(())
    }

    /// INIT phase: initialize all modules in topological order.
    ///
    /// System modules initialize first, followed by user modules.
    async fn run_init_phase(&self) -> Result<(), RegistryError> {
        tracing::info!("Phase: init");

        for entry in self.registry.modules_by_system_priority() {
            let ctx =
                self.ctx_builder
                    .for_module(entry.name)
                    .await
                    .map_err(|e| RegistryError::Init {
                        module: entry.name,
                        source: e,
                    })?;
            entry
                .core
                .init(&ctx)
                .await
                .map_err(|e| RegistryError::Init {
                    module: entry.name,
                    source: e,
                })?;
        }

        Ok(())
    }

    /// REST phase: compose the router against the REST host.
    ///
    /// This is a synchronous phase that builds the final Router by:
    /// 1. Preparing the host module
    /// 2. Registering all REST providers
    /// 3. Finalizing with OpenAPI endpoints
    async fn run_rest_phase(&self) -> Result<Router, RegistryError> {
        tracing::info!("Phase: rest (sync)");

        let mut router = Router::new();

        // Find host(s) and whether any rest modules exist
        let hosts: Vec<_> = self
            .registry
            .modules()
            .iter()
            .filter(|e| e.rest_host.is_some())
            .collect();

        match hosts.len() {
            0 => {
                return if self.registry.modules().iter().any(|e| e.rest.is_some()) {
                    Err(RegistryError::RestRequiresHost)
                } else {
                    Ok(router)
                }
            }
            1 => { /* proceed */ }
            _ => return Err(RegistryError::MultipleRestHosts),
        }

        // Resolve the single host entry and its module context
        let host_idx = self
            .registry
            .modules()
            .iter()
            .position(|e| e.rest_host.is_some())
            .ok_or(RegistryError::RestHostNotFoundAfterValidation)?;
        let host_entry = &self.registry.modules()[host_idx];
        let Some(host) = host_entry.rest_host.as_ref() else {
            return Err(RegistryError::RestHostMissingFromEntry);
        };
        let host_ctx = self
            .ctx_builder
            .for_module(host_entry.name)
            .await
            .map_err(|e| RegistryError::RestPrepare {
                module: host_entry.name,
                source: e,
            })?;

        // use host as the registry
        let registry: &dyn crate::contracts::OpenApiRegistry = host.as_registry();

        // 1) Host prepare: base Router / global middlewares / basic OAS meta
        router =
            host.rest_prepare(&host_ctx, router)
                .map_err(|source| RegistryError::RestPrepare {
                    module: host_entry.name,
                    source,
                })?;

        // 2) Register all REST providers (in the current discovery order)
        for e in self.registry.modules() {
            if let Some(rest) = &e.rest {
                let ctx = self.ctx_builder.for_module(e.name).await.map_err(|err| {
                    RegistryError::RestRegister {
                        module: e.name,
                        source: err,
                    }
                })?;
                router = rest
                    .register_rest(&ctx, router, registry)
                    .map_err(|source| RegistryError::RestRegister {
                        module: e.name,
                        source,
                    })?;
            }
        }

        // 3) Host finalize: attach /openapi.json and /docs, persist Router if needed (no server start)
        router = host.rest_finalize(&host_ctx, router).map_err(|source| {
            RegistryError::RestFinalize {
                module: host_entry.name,
                source,
            }
        })?;

        Ok(router)
    }

    /// gRPC registration phase: collect services from all grpc modules.
    ///
    /// Services are stored in the installer store for the grpc_hub to consume during start.
    async fn run_grpc_phase(&self) -> Result<(), RegistryError> {
        tracing::info!("Phase: grpc (registration)");

        // If no grpc_hub and no grpc_services, skip the phase
        if self.registry.grpc_hub.is_none() && self.registry.grpc_services.is_empty() {
            return Ok(());
        }

        // If there are grpc_services but no hub, that's an error
        if self.registry.grpc_hub.is_none() && !self.registry.grpc_services.is_empty() {
            return Err(RegistryError::GrpcRequiresHub);
        }

        // If there's a hub, collect all services and hand them off to the installer store
        if let Some(hub_name) = &self.registry.grpc_hub {
            let mut all_installers = Vec::<RegisterGrpcServiceFn>::new();
            let mut seen = HashSet::new();

            // Collect services from all grpc modules
            for (module_name, service_module) in &self.registry.grpc_services {
                let ctx = self
                    .ctx_builder
                    .for_module(module_name)
                    .await
                    .map_err(|err| RegistryError::GrpcRegister {
                        module: module_name.clone(),
                        source: err,
                    })?;

                let installers =
                    service_module
                        .get_grpc_services(&ctx)
                        .await
                        .map_err(|source| RegistryError::GrpcRegister {
                            module: module_name.clone(),
                            source,
                        })?;

                for reg in installers {
                    if !seen.insert(reg.service_name) {
                        return Err(RegistryError::GrpcRegister {
                            module: module_name.clone(),
                            source: anyhow::anyhow!(
                                "Duplicate gRPC service name: {}",
                                reg.service_name
                            ),
                        });
                    }
                    all_installers.push(reg);
                }
            }

            self.grpc_installers.set(all_installers).map_err(|source| {
                RegistryError::GrpcRegister {
                    module: hub_name.clone(),
                    source,
                }
            })?;
        }

        Ok(())
    }

    /// START phase: start all stateful modules.
    ///
    /// System modules start first, followed by user modules.
    async fn run_start_phase(&self) -> Result<(), RegistryError> {
        tracing::info!("Phase: start");

        for e in self.registry.modules_by_system_priority() {
            if let Some(s) = &e.stateful {
                tracing::debug!(
                    module = e.name,
                    is_system = e.is_system,
                    "Starting stateful module"
                );
                s.start(self.cancel.clone())
                    .await
                    .map_err(|source| RegistryError::Start {
                        module: e.name,
                        source,
                    })?;
                tracing::info!(module = e.name, "Started module");
            }
        }

        Ok(())
    }

    /// STOP phase: stop all stateful modules in reverse order.
    ///
    /// Errors are logged but do not fail the shutdown process.
    async fn run_stop_phase(&self) -> Result<(), RegistryError> {
        tracing::info!("Phase: stop");

        for e in self.registry.modules().iter().rev() {
            if let Some(s) = &e.stateful {
                if let Err(err) = s.stop(self.cancel.clone()).await {
                    tracing::warn!(module = e.name, error = %err, "Failed to stop module");
                }
                tracing::info!(module = e.name, "Stopped module");
            }
        }

        Ok(())
    }

    /// Run the full lifecycle: system_wire → DB → init → REST → gRPC → start → wait → stop.
    ///
    /// This is the main entry point for orchestrating the complete module lifecycle.
    pub async fn run_full_cycle(self) -> anyhow::Result<()> {
        // 1. System wiring phase (before init, only for system modules)
        self.wire_system().await?;

        // 2. DB migration phase (system modules first)
        self.run_db_phase().await?;

        // 3. Init phase (system modules first)
        self.run_init_phase().await?;

        // 4. REST phase (synchronous router composition)
        let _router = self.run_rest_phase().await?;

        // 5. gRPC registration phase
        self.run_grpc_phase().await?;

        // 6. Start phase
        self.run_start_phase().await?;

        // 7. Wait for cancellation
        self.cancel.cancelled().await;

        // 8. Stop phase
        self.run_stop_phase().await?;

        Ok(())
    }
}
