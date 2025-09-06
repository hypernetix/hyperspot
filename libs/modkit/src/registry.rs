// modkit/src/registry/mod.rs
use axum::Router;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use thiserror::Error;

// Re-exported contracts are referenced but not defined here.
use crate::context;
use crate::contracts;
use db;

pub struct ModuleEntry {
    pub name: &'static str,
    pub deps: &'static [&'static str],
    pub core: Arc<dyn contracts::Module>,
    pub rest: Option<Arc<dyn contracts::RestfulModule>>,
    pub rest_host: Option<Arc<dyn contracts::RestHostModule>>,
    pub db: Option<Arc<dyn contracts::DbModule>>,
    pub stateful: Option<Arc<dyn contracts::StatefulModule>>,
}

impl std::fmt::Debug for ModuleEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuleEntry")
            .field("name", &self.name)
            .field("deps", &self.deps)
            .field("has_rest", &self.rest.is_some())
            .field("is_rest_host", &self.rest_host.is_some())
            .field("has_db", &self.db.is_some())
            .field("has_stateful", &self.stateful.is_some())
            .finish()
    }
}

/// The function type submitted by the macro via `inventory::submit!`.
/// NOTE: It now takes a *builder*, not the final registry.
pub struct Registrator(pub fn(&mut RegistryBuilder));

inventory::collect!(Registrator);

/// The final, topo-sorted runtime registry.
pub struct ModuleRegistry {
    modules: Vec<ModuleEntry>, // topo-sorted
}

impl std::fmt::Debug for ModuleRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names: Vec<&'static str> = self.modules.iter().map(|m| m.name).collect();
        f.debug_struct("ModuleRegistry")
            .field("modules", &names)
            .finish()
    }
}

impl ModuleRegistry {
    pub fn modules(&self) -> &[ModuleEntry] {
        &self.modules
    }

    /// Discover via inventory, have registrators fill the builder, then build & topo-sort.
    pub fn discover_and_build() -> Result<Self, RegistryError> {
        let mut b = RegistryBuilder::default();
        for r in ::inventory::iter::<Registrator> {
            (r.0)(&mut b);
        }
        b.build_topo_sorted()
    }

    // ---- Ordered phases: init → DB → REST (sync) → start → stop ----

    pub async fn run_init_phase(&self, base_ctx: &context::ModuleCtx) -> Result<(), RegistryError> {
        for e in &self.modules {
            let ctx = base_ctx.clone().for_module(e.name);
            e.core
                .init(&ctx)
                .await
                .map_err(|source| RegistryError::Init {
                    module: e.name,
                    source: source.into(),
                })?;
        }
        Ok(())
    }

    pub async fn run_db_phase(&self, db: &db::DbHandle) -> Result<(), RegistryError> {
        for e in &self.modules {
            if let Some(dbm) = &e.db {
                // If you want advisory locks, do it here (kept minimal for portability):
                // let _lock = db.lock(e.name, "migration").await?;
                dbm.migrate(db)
                    .await
                    .map_err(|source| RegistryError::DbMigrate {
                        module: e.name,
                        source: source.into(),
                    })?;
            }
        }
        Ok(())
    }

    pub fn run_rest_phase(
        &self,
        base_ctx: &context::ModuleCtx,
        mut router: Router,
    ) -> Result<Router, RegistryError> {
        // Find host(s) and whether any rest modules exist
        let hosts: Vec<_> = self
            .modules
            .iter()
            .filter(|e| e.rest_host.is_some())
            .collect();

        match hosts.len() {
            0 => {
                if self.modules.iter().any(|e| e.rest.is_some()) {
                    return Err(RegistryError::RestRequiresHost);
                } else {
                    return Ok(router);
                }
            }
            1 => { /* proceed */ }
            _ => return Err(RegistryError::MultipleRestHosts),
        }

        // Resolve the single host entry and its module context
        let host_idx = self
            .modules
            .iter()
            .position(|e| e.rest_host.is_some())
            .ok_or(RegistryError::RestHostNotFoundAfterValidation)?;
        let host_entry = &self.modules[host_idx];
        let Some(host) = host_entry.rest_host.as_ref() else {
            return Err(RegistryError::RestHostMissingFromEntry);
        };
        let host_ctx = base_ctx.clone().for_module(host_entry.name);

        // use host as the registry
        let registry: &dyn contracts::OpenApiRegistry = host.as_registry();

        // 1) Host prepare: base Router / global middlewares / basic OAS meta
        router =
            host.rest_prepare(&host_ctx, router)
                .map_err(|source| RegistryError::RestPrepare {
                    module: host_entry.name,
                    source: source.into(),
                })?;

        // 2) Register all REST providers (in the current discovery order)
        for e in &self.modules {
            if let Some(rest) = &e.rest {
                let ctx = base_ctx.clone().for_module(e.name);
                router = rest
                    .register_rest(&ctx, router, registry)
                    .map_err(|source| RegistryError::RestRegister {
                        module: e.name,
                        source: source.into(),
                    })?;
            }
        }

        // 3) Host finalize: attach /openapi.json and /docs, persist Router if needed (no server start)
        router = host.rest_finalize(&host_ctx, router).map_err(|source| {
            RegistryError::RestFinalize {
                module: host_entry.name,
                source: source.into(),
            }
        })?;

        Ok(router)
    }

    pub async fn run_start_phase(&self, cancel: CancellationToken) -> Result<(), RegistryError> {
        for e in &self.modules {
            if let Some(s) = &e.stateful {
                s.start(cancel.clone())
                    .await
                    .map_err(|source| RegistryError::Start {
                        module: e.name,
                        source: source.into(),
                    })?;
            }
        }
        Ok(())
    }

    pub async fn run_stop_phase(&self, cancel: CancellationToken) -> Result<(), RegistryError> {
        for e in self.modules.iter().rev() {
            if let Some(s) = &e.stateful {
                if let Err(err) = s.stop(cancel.clone()).await {
                    tracing::warn!(module = e.name, error = %err, "Failed to stop module");
                }
            }
        }
        Ok(())
    }

    /// (Optional) quick lookup if you need it.
    pub fn get_module(&self, name: &str) -> Option<Arc<dyn contracts::Module>> {
        self.modules
            .iter()
            .find(|e| e.name == name)
            .map(|e| e.core.clone())
    }
}

/// Internal builder that macro registrators will feed.
/// Keys are module **names**; uniqueness enforced at build time.
#[derive(Default)]
pub struct RegistryBuilder {
    core: HashMap<&'static str, Arc<dyn contracts::Module>>,
    deps: HashMap<&'static str, &'static [&'static str]>,
    rest: HashMap<&'static str, Arc<dyn contracts::RestfulModule>>,
    rest_host: Option<(&'static str, Arc<dyn contracts::RestHostModule>)>,
    db: HashMap<&'static str, Arc<dyn contracts::DbModule>>,
    stateful: HashMap<&'static str, Arc<dyn contracts::StatefulModule>>,
    errors: Vec<String>,
}

impl RegistryBuilder {
    pub fn register_core_with_meta(
        &mut self,
        name: &'static str,
        deps: &'static [&'static str],
        m: Arc<dyn contracts::Module>,
    ) {
        if self.core.contains_key(name) {
            self.errors
                .push(format!("Module '{name}' is already registered"));
            return;
        }
        self.core.insert(name, m);
        self.deps.insert(name, deps);
    }

    pub fn register_rest_with_meta(
        &mut self,
        name: &'static str,
        m: Arc<dyn contracts::RestfulModule>,
    ) {
        self.rest.insert(name, m);
    }

    pub fn register_rest_host_with_meta(
        &mut self,
        name: &'static str,
        m: Arc<dyn contracts::RestHostModule>,
    ) {
        if let Some((existing, _)) = &self.rest_host {
            self.errors.push(format!(
                "Multiple REST host modules detected: '{}' and '{}'. Only one REST host is allowed.",
                existing, name
            ));
            return;
        }
        self.rest_host = Some((name, m));
    }

    pub fn register_db_with_meta(&mut self, name: &'static str, m: Arc<dyn contracts::DbModule>) {
        self.db.insert(name, m);
    }

    pub fn register_stateful_with_meta(
        &mut self,
        name: &'static str,
        m: Arc<dyn contracts::StatefulModule>,
    ) {
        self.stateful.insert(name, m);
    }

    /// Finalize & topo-sort; verify deps & capability binding to known cores.
    pub fn build_topo_sorted(self) -> Result<ModuleRegistry, RegistryError> {
        if let Some((host_name, _)) = &self.rest_host {
            if !self.core.contains_key(host_name) {
                return Err(RegistryError::UnknownModule(host_name.to_string()));
            }
        }
        if !self.errors.is_empty() {
            return Err(RegistryError::InvalidRegistryConfiguration {
                errors: self.errors,
            });
        }

        // 1) ensure every capability references a known core
        for (n, _) in self.rest.iter() {
            if !self.core.contains_key(n) {
                return Err(RegistryError::UnknownModule((*n).to_string()));
            }
        }
        if let Some((n, _)) = &self.rest_host {
            if !self.core.contains_key(n) {
                return Err(RegistryError::UnknownModule((*n).to_string()));
            }
        }
        for (n, _) in self.db.iter() {
            if !self.core.contains_key(n) {
                return Err(RegistryError::UnknownModule((*n).to_string()));
            }
        }
        for (n, _) in self.stateful.iter() {
            if !self.core.contains_key(n) {
                return Err(RegistryError::UnknownModule((*n).to_string()));
            }
        }

        // 2) build graph over core modules
        let names: Vec<&'static str> = self.core.keys().copied().collect();
        let mut idx: HashMap<&'static str, usize> = HashMap::new();
        for (i, &n) in names.iter().enumerate() {
            idx.insert(n, i);
        }

        let mut indeg = vec![0usize; names.len()];
        let mut adj = vec![Vec::<usize>::new(); names.len()];

        for (&n, &deps) in self.deps.iter() {
            let u = *idx
                .get(n)
                .ok_or_else(|| RegistryError::UnknownModule(n.to_string()))?;
            for &d in deps {
                let v = *idx.get(d).ok_or_else(|| RegistryError::UnknownDependency {
                    module: n.to_string(),
                    depends_on: d.to_string(),
                })?;
                // edge d -> n (dep before module)
                adj[v].push(u);
                indeg[u] += 1;
            }
        }

        // 3) Kahn’s algorithm
        let mut q = VecDeque::new();
        for i in 0..names.len() {
            if indeg[i] == 0 {
                q.push_back(i);
            }
        }

        let mut order = Vec::with_capacity(names.len());
        while let Some(u) = q.pop_front() {
            order.push(u);
            for &w in &adj[u] {
                indeg[w] -= 1;
                if indeg[w] == 0 {
                    q.push_back(w);
                }
            }
        }
        if order.len() != names.len() {
            return Err(RegistryError::CyclicDependency);
        }

        // 4) Build final entries in topo order
        let mut entries = Vec::with_capacity(order.len());
        for i in order {
            let name = names[i];
            let deps = *self
                .deps
                .get(name)
                .ok_or_else(|| RegistryError::MissingDeps(name.to_string()))?;

            let core = self
                .core
                .get(name)
                .cloned()
                .ok_or_else(|| RegistryError::CoreNotFound(name.to_string()))?;

            let entry = ModuleEntry {
                name,
                deps,
                core,
                rest: self.rest.get(name).cloned(),
                rest_host: self
                    .rest_host
                    .as_ref()
                    .filter(|(host_name, _)| *host_name == name)
                    .map(|(_, module)| module.clone()),
                db: self.db.get(name).cloned(),
                stateful: self.stateful.get(name).cloned(),
            };
            entries.push(entry);
        }

        tracing::info!(
            modules = ?entries.iter().map(|e| e.name).collect::<Vec<_>>(),
            "Module dependency order resolved (topo)"
        );

        Ok(ModuleRegistry { modules: entries })
    }
}

/// Structured errors for the module registry.
#[derive(Debug, Error)]
pub enum RegistryError {
    // Phase errors with module context
    #[error("initialization failed for module '{module}'")]
    Init {
        module: &'static str,
        #[source]
        source: anyhow::Error,
    },
    #[error("start failed for '{module}'")]
    Start {
        module: &'static str,
        #[source]
        source: anyhow::Error,
    },

    #[error("DB migration failed for module '{module}'")]
    DbMigrate {
        module: &'static str,
        #[source]
        source: anyhow::Error,
    },
    #[error("REST prepare failed for host module '{module}'")]
    RestPrepare {
        module: &'static str,
        #[source]
        source: anyhow::Error,
    },
    #[error("REST registration failed for module '{module}'")]
    RestRegister {
        module: &'static str,
        #[source]
        source: anyhow::Error,
    },
    #[error("REST finalize failed for host module '{module}'")]
    RestFinalize {
        module: &'static str,
        #[source]
        source: anyhow::Error,
    },
    #[error("REST phase requires an ingress host: modules with capability 'rest' found, but no module with capability 'rest_host'")]
    RestRequiresHost,
    #[error("multiple 'rest_host' modules detected; exactly one is allowed")]
    MultipleRestHosts,
    #[error("REST host module not found after validation")]
    RestHostNotFoundAfterValidation,
    #[error("REST host missing from entry")]
    RestHostMissingFromEntry,

    // Build/topo-sort errors
    #[error("unknown module '{0}'")]
    UnknownModule(String),
    #[error("module '{module}' depends on unknown '{depends_on}'")]
    UnknownDependency { module: String, depends_on: String },
    #[error("cyclic dependency detected among modules")]
    CyclicDependency,
    #[error("missing deps for '{0}'")]
    MissingDeps(String),
    #[error("core not found for '{0}'")]
    CoreNotFound(String),
    #[error("invalid registry configuration:\n{errors:#?}")]
    InvalidRegistryConfiguration { errors: Vec<String> },
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use std::sync::Arc;
    use tokio_util::sync::CancellationToken;

    // Use the real contracts/context APIs from the crate to avoid type mismatches.
    use crate::api::OpenApiRegistry;
    use crate::context::{ModuleCtx, ModuleCtxBuilder};
    use crate::contracts;

    /* --------------------------- Test helpers ------------------------- */
    #[derive(Default)]
    struct DummyCore;
    #[async_trait::async_trait]
    impl contracts::Module for DummyCore {
        async fn init(&self, _ctx: &ModuleCtx) -> anyhow::Result<()> {
            Ok(())
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[derive(Default)]
    struct DummyRegistry;
    impl OpenApiRegistry for DummyRegistry {
        fn register_operation(&self, _spec: &crate::api::OperationSpec) {}
        fn ensure_schema_raw(
            &self,
            name: &str,
            _schemas: Vec<(
                String,
                utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
            )>,
        ) -> String {
            name.to_string()
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[derive(Default)]
    struct DummyRestHost {
        reg: DummyRegistry,
    }
    #[async_trait::async_trait]
    impl contracts::RestHostModule for DummyRestHost {
        fn as_registry(&self) -> &dyn contracts::OpenApiRegistry {
            &self.reg
        }
        fn rest_prepare(&self, _ctx: &ModuleCtx, router: Router) -> Result<Router, anyhow::Error> {
            Ok(router)
        }
        fn rest_finalize(&self, _ctx: &ModuleCtx, router: Router) -> Result<Router, anyhow::Error> {
            Ok(router)
        }
    }

    #[derive(Default)]
    struct DummyRest;
    #[async_trait::async_trait]
    impl contracts::RestfulModule for DummyRest {
        fn register_rest(
            &self,
            _ctx: &ModuleCtx,
            router: Router,
            _registry: &dyn contracts::OpenApiRegistry,
        ) -> Result<Router, anyhow::Error> {
            Ok(router.route("/dummy", axum::routing::get(|| async { "ok" })))
        }
    }

    /* ------------------------------- Tests ---------------------------- */

    #[test]
    fn topo_sort_happy_path() {
        let mut b = RegistryBuilder::default();
        // cores
        b.register_core_with_meta("core_a", &[], Arc::new(DummyCore::default()));
        b.register_core_with_meta("core_b", &["core_a"], Arc::new(DummyCore::default()));

        let reg = b.build_topo_sorted().unwrap();
        let order: Vec<_> = reg.modules().iter().map(|m| m.name).collect();
        assert_eq!(order, vec!["core_a", "core_b"]);
    }

    #[test]
    fn unknown_dependency_error() {
        let mut b = RegistryBuilder::default();
        b.register_core_with_meta("core_a", &["missing_dep"], Arc::new(DummyCore::default()));

        let err = b.build_topo_sorted().unwrap_err();
        match err {
            RegistryError::UnknownDependency { module, depends_on } => {
                assert_eq!(module, "core_a");
                assert_eq!(depends_on, "missing_dep");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn cyclic_dependency_detected() {
        let mut b = RegistryBuilder::default();
        b.register_core_with_meta("a", &["b"], Arc::new(DummyCore::default()));
        b.register_core_with_meta("b", &["a"], Arc::new(DummyCore::default()));

        let err = b.build_topo_sorted().unwrap_err();
        matches!(err, RegistryError::CyclicDependency);
    }

    #[test]
    fn duplicate_core_reported_in_configuration_errors() {
        let mut b = RegistryBuilder::default();
        b.register_core_with_meta("a", &[], Arc::new(DummyCore::default()));
        // duplicate
        b.register_core_with_meta("a", &[], Arc::new(DummyCore::default()));

        let err = b.build_topo_sorted().unwrap_err();
        match err {
            RegistryError::InvalidRegistryConfiguration { errors } => {
                assert!(
                    errors.iter().any(|e| e.contains("already registered")),
                    "expected duplicate registration error, got {errors:?}"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn rest_requires_host_if_rest_modules_exist() {
        // Build with 1 core that has REST capability, but no rest host.
        let mut b = RegistryBuilder::default();
        b.register_core_with_meta("svc", &[], Arc::new(DummyCore::default()));
        b.register_rest_with_meta("svc", Arc::new(DummyRest::default()));
        let reg = b.build_topo_sorted().unwrap();

        let router = Router::new();
        let base_ctx = ModuleCtxBuilder::new(CancellationToken::new()).build();
        let err = reg.run_rest_phase(&base_ctx, router).unwrap_err();
        matches!(err, RegistryError::RestRequiresHost);
    }

    #[test]
    fn rest_single_host_and_provider_happy_path() {
        // Build with one host and one REST provider
        let mut b = RegistryBuilder::default();
        b.register_core_with_meta("host", &[], Arc::new(DummyCore::default()));
        b.register_rest_host_with_meta("host", Arc::new(DummyRestHost::default()));

        b.register_core_with_meta("svc", &[], Arc::new(DummyCore::default()));
        b.register_rest_with_meta("svc", Arc::new(DummyRest::default()));

        let reg = b.build_topo_sorted().unwrap();

        let router = Router::new();
        let base_ctx = ModuleCtxBuilder::new(CancellationToken::new()).build();
        let router = reg.run_rest_phase(&base_ctx, router).unwrap();

        // The DummyRest adds /dummy endpoint during register_rest
        // (We don't spin a server; just ensure Router returned successfully.)
        let _ = router;
    }

    #[tokio::test]
    async fn phases_run_without_errors_with_empty_implementations() {
        // No REST, DB, or stateful modules; only init/start/stop with defaults.
        let mut b = RegistryBuilder::default();
        b.register_core_with_meta("a", &[], Arc::new(DummyCore::default()));
        b.register_core_with_meta("b", &["a"], Arc::new(DummyCore::default()));
        let reg = b.build_topo_sorted().unwrap();

        // init
        let ctx = ModuleCtxBuilder::new(CancellationToken::new()).build();
        reg.run_init_phase(&ctx).await.unwrap();

        // db phase skipped because no modules implement DbModule

        // start/stop
        let cancel = CancellationToken::new();
        reg.run_start_phase(cancel.child_token()).await.unwrap();
        reg.run_stop_phase(cancel.child_token()).await.unwrap();
    }
}
