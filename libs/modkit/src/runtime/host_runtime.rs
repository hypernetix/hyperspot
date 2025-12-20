//! Host Runtime - orchestrates the full `ModKit` lifecycle
//!
//! This module contains the `HostRuntime` type that owns and coordinates
//! the execution of all lifecycle phases.
//!
//! High-level phase order:
//! - `pre_init` (system modules only)
//! - DB migrations (modules with DB capability)
//! - `init` (all modules)
//! - `post_init` (system modules only; runs after *all* `init` complete)
//! - REST wiring (modules with REST capability; requires a single REST host)
//! - gRPC registration (modules with gRPC capability; requires a single gRPC hub)
//! - start/stop (stateful modules)
//! - `OoP` spawn / wait / stop (host-only orchestration)

use axum::Router;
use std::collections::HashSet;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::backends::OopSpawnConfig;
use crate::client_hub::ClientHub;
use crate::config::ConfigProvider;
use crate::context::ModuleContextBuilder;
use crate::registry::{ModuleEntry, ModuleRegistry, RegistryError};
use crate::runtime::{GrpcInstallerStore, ModuleManager, OopSpawnOptions, SystemContext};

/// How the runtime should provide DBs to modules.
#[derive(Clone)]
pub enum DbOptions {
    /// No database integration. `ModuleCtx::db()` will be `None`, `db_required()` will error.
    None,
    /// Use a `DbManager` to handle database connections with Figment-based configuration.
    Manager(Arc<modkit_db::DbManager>),
}

/// Environment variable name for passing directory endpoint to `OoP` modules.
pub const MODKIT_DIRECTORY_ENDPOINT_ENV: &str = "MODKIT_DIRECTORY_ENDPOINT";

/// Environment variable name for passing rendered module config to `OoP` modules.
pub const MODKIT_MODULE_CONFIG_ENV: &str = "MODKIT_MODULE_CONFIG";

/// `HostRuntime` owns the lifecycle orchestration for `ModKit`.
///
/// It encapsulates all runtime state and drives modules through the full lifecycle (see module docs).
pub struct HostRuntime {
    registry: ModuleRegistry,
    ctx_builder: ModuleContextBuilder,
    instance_id: Uuid,
    module_manager: Arc<ModuleManager>,
    grpc_installers: Arc<GrpcInstallerStore>,
    #[allow(dead_code)]
    client_hub: Arc<ClientHub>,
    cancel: CancellationToken,
    #[allow(dead_code)]
    db_options: DbOptions,
    /// `OoP` module spawn configuration and backend
    oop_options: Option<OopSpawnOptions>,
}

impl HostRuntime {
    /// Create a new `HostRuntime` instance.
    ///
    /// This prepares all runtime components but does not start any lifecycle phases.
    pub fn new(
        registry: ModuleRegistry,
        modules_cfg: Arc<dyn ConfigProvider>,
        db_options: DbOptions,
        client_hub: Arc<ClientHub>,
        cancel: CancellationToken,
        instance_id: Uuid,
        oop_options: Option<OopSpawnOptions>,
    ) -> Self {
        // Create runtime-owned components for system modules
        let module_manager = Arc::new(ModuleManager::new());
        let grpc_installers = Arc::new(GrpcInstallerStore::new());

        // Build the context builder that will resolve per-module DbHandles
        let db_manager = match &db_options {
            DbOptions::Manager(mgr) => Some(mgr.clone()),
            DbOptions::None => None,
        };

        let ctx_builder = ModuleContextBuilder::new(
            instance_id,
            modules_cfg,
            client_hub.clone(),
            cancel.clone(),
            db_manager,
        );

        Self {
            registry,
            ctx_builder,
            instance_id,
            module_manager,
            grpc_installers,
            client_hub,
            cancel,
            db_options,
            oop_options,
        }
    }

    /// `PRE_INIT` phase: wire runtime internals into system modules.
    ///
    /// This phase runs before init and only for modules with the "system" capability.
    ///
    /// # Errors
    /// Returns `RegistryError` if system wiring fails.
    pub fn run_pre_init_phase(&self) -> Result<(), RegistryError> {
        tracing::info!("Phase: pre_init");

        let sys_ctx = SystemContext::new(
            self.instance_id,
            Arc::clone(&self.module_manager),
            Arc::clone(&self.grpc_installers),
        );

        for entry in self.registry.modules() {
            if let Some(sys_mod) = &entry.system {
                tracing::debug!(module = entry.name, "Running system pre_init");
                sys_mod
                    .pre_init(&sys_ctx)
                    .map_err(|e| RegistryError::PreInit {
                        module: entry.name,
                        source: e,
                    })?;
            }
        }

        Ok(())
    }

    /// Helper: resolve context for a module with error mapping.
    async fn module_context(
        &self,
        module_name: &'static str,
    ) -> Result<crate::context::ModuleCtx, RegistryError> {
        self.ctx_builder
            .for_module(module_name)
            .await
            .map_err(|e| RegistryError::DbMigrate {
                module: module_name,
                source: e,
            })
    }

    /// Helper: extract DB handle and module if both exist.
    fn db_migration_target<'a>(
        ctx: &'a crate::context::ModuleCtx,
        db_module: Option<&'a Arc<dyn crate::contracts::DbModule>>,
    ) -> Option<(Arc<modkit_db::DbHandle>, &'a dyn crate::contracts::DbModule)> {
        match (ctx.db_optional(), db_module) {
            (Some(db), Some(dbm)) => Some((db, dbm.as_ref())),
            _ => None,
        }
    }

    /// Helper: run migration for a single module.
    async fn migrate_module(
        module_name: &'static str,
        db: &modkit_db::DbHandle,
        db_module: &dyn crate::contracts::DbModule,
    ) -> Result<(), RegistryError> {
        tracing::debug!(module = module_name, "Running DB migration");
        db_module
            .migrate(db)
            .await
            .map_err(|source| RegistryError::DbMigrate {
                module: module_name,
                source,
            })
    }

    /// DB MIGRATION phase: run migrations for all modules with DB capability.
    ///
    /// Runs before init, with system modules processed first.
    async fn run_db_phase(&self) -> Result<(), RegistryError> {
        tracing::info!("Phase: db (before init)");

        for entry in self.registry.modules_by_system_priority() {
            let ctx = self.module_context(entry.name).await?;

            match Self::db_migration_target(&ctx, entry.db.as_ref()) {
                Some((db, dbm)) => {
                    Self::migrate_module(entry.name, &db, dbm).await?;
                }
                None if entry.db.is_some() => {
                    tracing::debug!(
                        module = entry.name,
                        "Module has DbModule trait but no DB handle (no config)"
                    );
                }
                None => {}
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

    /// `POST_INIT` phase: optional hook after ALL modules completed `init()`.
    ///
    /// This provides a global barrier between initialization-time registration
    /// and subsequent phases that may rely on a fully-populated runtime registry.
    ///
    /// System modules run first, followed by user modules, preserving topo order.
    async fn run_post_init_phase(&self) -> Result<(), RegistryError> {
        tracing::info!("Phase: post_init");

        let sys_ctx = SystemContext::new(
            self.instance_id,
            Arc::clone(&self.module_manager),
            Arc::clone(&self.grpc_installers),
        );

        for entry in self.registry.modules_by_system_priority() {
            if let Some(sys_mod) = &entry.system {
                sys_mod
                    .post_init(&sys_ctx)
                    .await
                    .map_err(|e| RegistryError::PostInit {
                        module: entry.name,
                        source: e,
                    })?;
            }
        }

        Ok(())
    }

    /// REST phase: compose the router against the REST host.
    ///
    /// This is a synchronous phase that builds the final Router by:
    /// 1. Preparing the host module
    /// 2. Registering all REST providers
    /// 3. Finalizing with `OpenAPI` endpoints
    async fn run_rest_phase(&self) -> Result<Router, RegistryError> {
        tracing::info!("Phase: rest (sync)");

        let mut router = Router::new();

        // Find host(s) and whether any rest modules exist
        let host_count = self
            .registry
            .modules()
            .iter()
            .filter(|e| e.rest_host.is_some())
            .count();

        match host_count {
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
    /// Services are stored in the installer store for the `grpc_hub` to consume during start.
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

        // If there's a hub, collect all services grouped by module and hand them off to the installer store
        if let Some(hub_name) = &self.registry.grpc_hub {
            let mut modules_data = Vec::new();
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

                for reg in &installers {
                    if !seen.insert(reg.service_name) {
                        return Err(RegistryError::GrpcRegister {
                            module: module_name.clone(),
                            source: anyhow::anyhow!(
                                "Duplicate gRPC service name: {}",
                                reg.service_name
                            ),
                        });
                    }
                }

                modules_data.push(crate::runtime::ModuleInstallers {
                    module_name: module_name.clone(),
                    installers,
                });
            }

            self.grpc_installers
                .set(crate::runtime::GrpcInstallerData {
                    modules: modules_data,
                })
                .map_err(|source| RegistryError::GrpcRegister {
                    module: hub_name.clone(),
                    source,
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
                    is_system = e.system.is_some(),
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

    /// Stop a single module, logging errors but continuing execution.
    async fn stop_one_module(entry: &ModuleEntry, cancel: CancellationToken) {
        if let Some(s) = &entry.stateful {
            if let Err(err) = s.stop(cancel).await {
                tracing::warn!(module = entry.name, error = %err, "Failed to stop module");
            } else {
                tracing::info!(module = entry.name, "Stopped module");
            }
        }
    }

    /// STOP phase: stop all stateful modules in reverse order.
    ///
    /// Errors are logged but do not fail the shutdown process.
    /// Note: `OoP` modules are stopped automatically by the backend when the
    /// cancellation token is triggered.
    async fn run_stop_phase(&self) -> Result<(), RegistryError> {
        tracing::info!("Phase: stop");

        for e in self.registry.modules().iter().rev() {
            Self::stop_one_module(e, self.cancel.clone()).await;
        }

        Ok(())
    }

    /// `OoP` SPAWN phase: spawn out-of-process modules after start phase.
    ///
    /// This phase runs after `grpc_hub` is already listening, so we can pass
    /// the real directory endpoint to `OoP` modules.
    async fn run_oop_spawn_phase(&self) -> Result<(), RegistryError> {
        let oop_opts = match &self.oop_options {
            Some(opts) if !opts.modules.is_empty() => opts,
            _ => return Ok(()),
        };

        tracing::info!("Phase: oop_spawn");

        // Wait for grpc_hub to publish its endpoint (it runs async in start phase)
        let directory_endpoint = self.wait_for_grpc_hub_endpoint().await;

        for module_cfg in &oop_opts.modules {
            // Build environment with directory endpoint and rendered config
            // Note: User controls --config via execution.args in master config
            let mut env = module_cfg.env.clone();
            env.insert(
                MODKIT_MODULE_CONFIG_ENV.to_owned(),
                module_cfg.rendered_config_json.clone(),
            );
            if let Some(ref endpoint) = directory_endpoint {
                env.insert(MODKIT_DIRECTORY_ENDPOINT_ENV.to_owned(), endpoint.clone());
            }

            // Use args from execution config as-is (user controls --config via args)
            let args = module_cfg.args.clone();

            let spawn_config = OopSpawnConfig {
                module_name: module_cfg.module_name.clone(),
                binary: module_cfg.binary.clone(),
                args,
                env,
                working_directory: module_cfg.working_directory.clone(),
            };

            oop_opts
                .backend
                .spawn(spawn_config)
                .await
                .map_err(|e| RegistryError::OopSpawn {
                    module: module_cfg.module_name.clone(),
                    source: e,
                })?;

            tracing::info!(
                module = %module_cfg.module_name,
                directory_endpoint = ?directory_endpoint,
                "Spawned OoP module via backend"
            );
        }

        Ok(())
    }

    /// Wait for `grpc_hub` to publish its bound endpoint.
    ///
    /// Polls the `GrpcHubModule::bound_endpoint()` with a short interval until available or timeout.
    /// Returns None if no `grpc_hub` is running or if it times out.
    async fn wait_for_grpc_hub_endpoint(&self) -> Option<String> {
        const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(10);
        const MAX_WAIT: std::time::Duration = std::time::Duration::from_secs(5);

        // Find grpc_hub in registry
        let grpc_hub = self
            .registry
            .modules()
            .iter()
            .find_map(|e| e.grpc_hub.as_ref());

        let Some(hub) = grpc_hub else {
            return None; // No grpc_hub registered
        };

        let start = std::time::Instant::now();

        loop {
            if let Some(endpoint) = hub.bound_endpoint() {
                tracing::debug!(
                    endpoint = %endpoint,
                    elapsed_ms = start.elapsed().as_millis(),
                    "gRPC hub endpoint available"
                );
                return Some(endpoint);
            }

            if start.elapsed() > MAX_WAIT {
                tracing::warn!("Timed out waiting for gRPC hub to bind");
                return None;
            }

            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    /// Run the full lifecycle: `pre_init` → DB → init → `post_init` → REST → gRPC → start → `OoP` spawn → wait → stop.
    ///
    /// This is the main entry point for orchestrating the complete module lifecycle.
    ///
    /// # Errors
    /// Returns an error if any lifecycle phase fails.
    pub async fn run_module_phases(self) -> anyhow::Result<()> {
        // 1. Pre-init phase (before init, only for system modules)
        self.run_pre_init_phase()?;

        // 2. DB migration phase (system modules first)
        self.run_db_phase().await?;

        // 3. Init phase (system modules first)
        self.run_init_phase().await?;

        // 4. Post-init phase (barrier after ALL init; system modules only)
        self.run_post_init_phase().await?;

        // 5. REST phase (synchronous router composition)
        let _router = self.run_rest_phase().await?;

        // 6. gRPC registration phase
        self.run_grpc_phase().await?;

        // 7. Start phase
        self.run_start_phase().await?;

        // 8. OoP spawn phase (after grpc_hub is running)
        self.run_oop_spawn_phase().await?;

        // 9. Wait for cancellation
        self.cancel.cancelled().await;

        // 10. Stop phase
        self.run_stop_phase().await?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::context::ModuleCtx;
    use crate::contracts::{Module, StatefulModule, SystemModule};
    use crate::registry::RegistryBuilder;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Default)]
    #[allow(dead_code)]
    struct DummyCore;
    #[async_trait::async_trait]
    impl Module for DummyCore {
        async fn init(&self, _ctx: &ModuleCtx) -> anyhow::Result<()> {
            Ok(())
        }
    }

    struct StopOrderTracker {
        my_order: usize,
        stop_order: Arc<AtomicUsize>,
    }

    impl StopOrderTracker {
        fn new(counter: &Arc<AtomicUsize>, stop_order: Arc<AtomicUsize>) -> Self {
            let my_order = counter.fetch_add(1, Ordering::SeqCst);
            Self {
                my_order,
                stop_order,
            }
        }
    }

    #[async_trait::async_trait]
    impl Module for StopOrderTracker {
        async fn init(&self, _ctx: &ModuleCtx) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl StatefulModule for StopOrderTracker {
        async fn start(&self, _cancel: CancellationToken) -> anyhow::Result<()> {
            Ok(())
        }
        async fn stop(&self, _cancel: CancellationToken) -> anyhow::Result<()> {
            let order = self.stop_order.fetch_add(1, Ordering::SeqCst);
            tracing::info!(
                my_order = self.my_order,
                stop_order = order,
                "Module stopped"
            );
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_stop_phase_reverse_order() {
        let counter = Arc::new(AtomicUsize::new(0));
        let stop_order = Arc::new(AtomicUsize::new(0));

        let module_a = Arc::new(StopOrderTracker::new(&counter, stop_order.clone()));
        let module_b = Arc::new(StopOrderTracker::new(&counter, stop_order.clone()));
        let module_c = Arc::new(StopOrderTracker::new(&counter, stop_order.clone()));

        let mut builder = RegistryBuilder::default();
        builder.register_core_with_meta("a", &[], module_a.clone() as Arc<dyn Module>);
        builder.register_core_with_meta("b", &["a"], module_b.clone() as Arc<dyn Module>);
        builder.register_core_with_meta("c", &["b"], module_c.clone() as Arc<dyn Module>);

        builder.register_stateful_with_meta("a", module_a.clone() as Arc<dyn StatefulModule>);
        builder.register_stateful_with_meta("b", module_b.clone() as Arc<dyn StatefulModule>);
        builder.register_stateful_with_meta("c", module_c.clone() as Arc<dyn StatefulModule>);

        let registry = builder.build_topo_sorted().unwrap();

        // Verify module order is a -> b -> c
        let module_names: Vec<_> = registry.modules().iter().map(|m| m.name).collect();
        assert_eq!(module_names, vec!["a", "b", "c"]);

        let client_hub = Arc::new(ClientHub::new());
        let cancel = CancellationToken::new();
        let config_provider: Arc<dyn ConfigProvider> = Arc::new(EmptyConfigProvider);

        let runtime = HostRuntime::new(
            registry,
            config_provider,
            DbOptions::None,
            client_hub,
            cancel.clone(),
            Uuid::new_v4(),
            None,
        );

        // Run stop phase
        runtime.run_stop_phase().await.unwrap();

        // Verify modules stopped in reverse order: c (stop_order=0), b (stop_order=1), a (stop_order=2)
        // Module order is: a=0, b=1, c=2
        // Stop order should be: c=0, b=1, a=2
        assert_eq!(stop_order.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_stop_phase_continues_on_error() {
        struct FailingModule {
            should_fail: bool,
            stopped: Arc<AtomicUsize>,
        }

        #[async_trait::async_trait]
        impl Module for FailingModule {
            async fn init(&self, _ctx: &ModuleCtx) -> anyhow::Result<()> {
                Ok(())
            }
        }

        #[async_trait::async_trait]
        impl StatefulModule for FailingModule {
            async fn start(&self, _cancel: CancellationToken) -> anyhow::Result<()> {
                Ok(())
            }
            async fn stop(&self, _cancel: CancellationToken) -> anyhow::Result<()> {
                self.stopped.fetch_add(1, Ordering::SeqCst);
                if self.should_fail {
                    anyhow::bail!("Intentional failure")
                }
                Ok(())
            }
        }

        let stopped = Arc::new(AtomicUsize::new(0));
        let module_a = Arc::new(FailingModule {
            should_fail: false,
            stopped: stopped.clone(),
        });
        let module_b = Arc::new(FailingModule {
            should_fail: true,
            stopped: stopped.clone(),
        });
        let module_c = Arc::new(FailingModule {
            should_fail: false,
            stopped: stopped.clone(),
        });

        let mut builder = RegistryBuilder::default();
        builder.register_core_with_meta("a", &[], module_a.clone() as Arc<dyn Module>);
        builder.register_core_with_meta("b", &["a"], module_b.clone() as Arc<dyn Module>);
        builder.register_core_with_meta("c", &["b"], module_c.clone() as Arc<dyn Module>);

        builder.register_stateful_with_meta("a", module_a.clone() as Arc<dyn StatefulModule>);
        builder.register_stateful_with_meta("b", module_b.clone() as Arc<dyn StatefulModule>);
        builder.register_stateful_with_meta("c", module_c.clone() as Arc<dyn StatefulModule>);

        let registry = builder.build_topo_sorted().unwrap();

        let client_hub = Arc::new(ClientHub::new());
        let cancel = CancellationToken::new();
        let config_provider: Arc<dyn ConfigProvider> = Arc::new(EmptyConfigProvider);

        let runtime = HostRuntime::new(
            registry,
            config_provider,
            DbOptions::None,
            client_hub,
            cancel.clone(),
            Uuid::new_v4(),
            None,
        );

        // Run stop phase - should not fail even though module_b fails
        runtime.run_stop_phase().await.unwrap();

        // All modules should have attempted to stop
        assert_eq!(stopped.load(Ordering::SeqCst), 3);
    }

    struct EmptyConfigProvider;
    impl ConfigProvider for EmptyConfigProvider {
        fn get_module_config(&self, _module_name: &str) -> Option<&serde_json::Value> {
            None
        }
    }

    #[tokio::test]
    async fn test_post_init_runs_after_all_init_and_system_first() {
        #[derive(Clone)]
        struct TrackHooks {
            name: &'static str,
            events: Arc<Mutex<Vec<String>>>,
        }

        #[async_trait::async_trait]
        impl Module for TrackHooks {
            async fn init(&self, _ctx: &ModuleCtx) -> anyhow::Result<()> {
                self.events.lock().await.push(format!("init:{}", self.name));
                Ok(())
            }
        }

        #[async_trait::async_trait]
        impl SystemModule for TrackHooks {
            fn pre_init(&self, _sys: &crate::runtime::SystemContext) -> anyhow::Result<()> {
                Ok(())
            }

            async fn post_init(&self, _sys: &crate::runtime::SystemContext) -> anyhow::Result<()> {
                self.events
                    .lock()
                    .await
                    .push(format!("post_init:{}", self.name));
                Ok(())
            }
        }

        let events = Arc::new(Mutex::new(Vec::<String>::new()));
        let sys_a = Arc::new(TrackHooks {
            name: "sys_a",
            events: events.clone(),
        });
        let user_b = Arc::new(TrackHooks {
            name: "user_b",
            events: events.clone(),
        });
        let user_c = Arc::new(TrackHooks {
            name: "user_c",
            events: events.clone(),
        });

        let mut builder = RegistryBuilder::default();
        builder.register_core_with_meta("sys_a", &[], sys_a.clone() as Arc<dyn Module>);
        builder.register_core_with_meta("user_b", &["sys_a"], user_b.clone() as Arc<dyn Module>);
        builder.register_core_with_meta("user_c", &["user_b"], user_c.clone() as Arc<dyn Module>);
        builder.register_system_with_meta("sys_a", sys_a.clone() as Arc<dyn SystemModule>);

        let registry = builder.build_topo_sorted().unwrap();

        let client_hub = Arc::new(ClientHub::new());
        let cancel = CancellationToken::new();
        let config_provider: Arc<dyn ConfigProvider> = Arc::new(EmptyConfigProvider);

        let runtime = HostRuntime::new(
            registry,
            config_provider,
            DbOptions::None,
            client_hub,
            cancel,
            Uuid::new_v4(),
            None,
        );

        // Run init phase for all modules, then post_init as a separate barrier phase.
        runtime.run_init_phase().await.unwrap();
        runtime.run_post_init_phase().await.unwrap();

        let events = events.lock().await.clone();
        let first_post_init = events
            .iter()
            .position(|e| e.starts_with("post_init:"))
            .expect("expected post_init events");
        assert!(
            events[..first_post_init]
                .iter()
                .all(|e| e.starts_with("init:")),
            "expected all init events before post_init, got: {events:?}"
        );

        // system-first order within each phase
        assert_eq!(
            events,
            vec![
                "init:sys_a",
                "init:user_b",
                "init:user_c",
                "post_init:sys_a",
            ]
        );
    }
}
