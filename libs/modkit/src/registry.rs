// modkit/src/registry/mod.rs
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use thiserror::Error;

// Re-exported contracts are referenced but not defined here.
use crate::contracts;

/// Type alias for REST host module configuration.
type RestHostEntry = (&'static str, Arc<dyn contracts::RestHostModule>);

pub struct ModuleEntry {
    pub name: &'static str,
    pub deps: &'static [&'static str],
    pub core: Arc<dyn contracts::Module>,
    pub rest: Option<Arc<dyn contracts::RestfulModule>>,
    pub rest_host: Option<Arc<dyn contracts::RestHostModule>>,
    pub db: Option<Arc<dyn contracts::DbModule>>,
    pub stateful: Option<Arc<dyn contracts::StatefulModule>>,
    pub is_system: bool,
    pub is_grpc_hub: bool,
    pub grpc_service: Option<Arc<dyn contracts::GrpcServiceModule>>,
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
            .field("is_system", &self.is_system)
            .field("is_grpc_hub", &self.is_grpc_hub)
            .field("has_grpc_service", &self.grpc_service.is_some())
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
    pub grpc_hub: Option<String>,
    pub grpc_services: Vec<(String, Arc<dyn contracts::GrpcServiceModule>)>,
}

impl std::fmt::Debug for ModuleRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names: Vec<&'static str> = self.modules.iter().map(|m| m.name).collect();
        f.debug_struct("ModuleRegistry")
            .field("modules", &names)
            .field("has_grpc_hub", &self.grpc_hub.is_some())
            .field("grpc_services_count", &self.grpc_services.len())
            .finish()
    }
}

impl ModuleRegistry {
    pub fn modules(&self) -> &[ModuleEntry] {
        &self.modules
    }

    /// Returns modules ordered by system priority.
    /// System modules come first, followed by non-system modules.
    /// Within each group, the original topological order is preserved.
    pub fn modules_by_system_priority(&self) -> Vec<&ModuleEntry> {
        let mut system_mods = Vec::new();
        let mut non_system_mods = Vec::new();

        for entry in &self.modules {
            if entry.is_system {
                system_mods.push(entry);
            } else {
                non_system_mods.push(entry);
            }
        }

        system_mods.extend(non_system_mods);
        system_mods
    }

    /// Discover via inventory, have registrators fill the builder, then build & topo-sort.
    pub fn discover_and_build() -> Result<Self, RegistryError> {
        let mut b = RegistryBuilder::default();
        for r in ::inventory::iter::<Registrator> {
            r.0(&mut b);
        }
        b.build_topo_sorted()
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
    rest_host: Option<RestHostEntry>,
    db: HashMap<&'static str, Arc<dyn contracts::DbModule>>,
    stateful: HashMap<&'static str, Arc<dyn contracts::StatefulModule>>,
    system_modules: std::collections::HashSet<&'static str>,
    grpc_hub: Option<&'static str>,
    grpc_services: HashMap<&'static str, Arc<dyn contracts::GrpcServiceModule>>,
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

    pub fn register_system_with_meta(&mut self, name: &'static str) {
        self.system_modules.insert(name);
    }

    pub fn register_grpc_hub_with_meta(&mut self, name: &'static str) {
        if let Some(existing) = &self.grpc_hub {
            self.errors.push(format!(
                "Multiple gRPC hub modules detected: '{}' and '{}'. Only one gRPC hub is allowed.",
                existing, name
            ));
            return;
        }
        self.grpc_hub = Some(name);
    }

    pub fn register_grpc_service_with_meta(
        &mut self,
        name: &'static str,
        m: Arc<dyn contracts::GrpcServiceModule>,
    ) {
        self.grpc_services.insert(name, m);
    }

    /// Detect cycles in the dependency graph using DFS with path tracking.
    /// Returns the cycle path if found, None otherwise.
    fn detect_cycle_with_path(
        names: &[&'static str],
        adj: &[Vec<usize>],
    ) -> Option<Vec<&'static str>> {
        #[derive(Clone, Copy, PartialEq)]
        enum Color {
            White, // unvisited
            Gray,  // visiting (on current path)
            Black, // visited (finished)
        }

        let mut colors = vec![Color::White; names.len()];
        let mut path = Vec::new();

        fn dfs(
            node: usize,
            names: &[&'static str],
            adj: &[Vec<usize>],
            colors: &mut [Color],
            path: &mut Vec<usize>,
        ) -> Option<Vec<&'static str>> {
            colors[node] = Color::Gray;
            path.push(node);

            for &neighbor in &adj[node] {
                match colors[neighbor] {
                    Color::Gray => {
                        // Found a back edge - cycle detected
                        // Find the cycle start in the current path
                        if let Some(cycle_start) = path.iter().position(|&n| n == neighbor) {
                            let cycle_indices = &path[cycle_start..];
                            let mut cycle_path: Vec<&'static str> =
                                cycle_indices.iter().map(|&i| names[i]).collect();
                            // Close the cycle by adding the first node again
                            cycle_path.push(names[neighbor]);
                            return Some(cycle_path);
                        }
                    }
                    Color::White => {
                        if let Some(cycle) = dfs(neighbor, names, adj, colors, path) {
                            return Some(cycle);
                        }
                    }
                    Color::Black => {
                        // Already processed, no cycle through this path
                    }
                }
            }

            path.pop();
            colors[node] = Color::Black;
            None
        }

        for i in 0..names.len() {
            if colors[i] == Color::White {
                if let Some(cycle) = dfs(i, names, adj, &mut colors, &mut path) {
                    return Some(cycle);
                }
            }
        }

        None
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
        if let Some(n) = &self.grpc_hub {
            if !self.core.contains_key(n) {
                return Err(RegistryError::UnknownModule((*n).to_string()));
            }
        }
        for (n, _) in self.grpc_services.iter() {
            if !self.core.contains_key(n) {
                return Err(RegistryError::UnknownModule((*n).to_string()));
            }
        }

        // 2) build graph over core modules and detect cycles
        let names: Vec<&'static str> = self.core.keys().copied().collect();
        let mut idx: HashMap<&'static str, usize> = HashMap::new();
        for (i, &n) in names.iter().enumerate() {
            idx.insert(n, i);
        }

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
            }
        }

        // 3) Cycle detection using DFS with path tracking
        if let Some(cycle_path) = Self::detect_cycle_with_path(&names, &adj) {
            return Err(RegistryError::CycleDetected { path: cycle_path });
        }

        // 4) Kahn's algorithm for topological sorting (we know there are no cycles)
        let mut indeg = vec![0usize; names.len()];
        for adj_list in &adj {
            for &target in adj_list {
                indeg[target] += 1;
            }
        }

        let mut q = VecDeque::new();
        for (i, &degree) in indeg.iter().enumerate() {
            if degree == 0 {
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
                is_system: self.system_modules.contains(name),
                is_grpc_hub: self
                    .grpc_hub
                    .map(|hub_name| hub_name == name)
                    .unwrap_or(false),
                grpc_service: self.grpc_services.get(name).cloned(),
            };
            entries.push(entry);
        }

        // Collect grpc_hub and grpc_services for the final registry
        let grpc_hub = self.grpc_hub.map(|name| name.to_string());

        let grpc_services: Vec<(String, Arc<dyn contracts::GrpcServiceModule>)> = self
            .grpc_services
            .iter()
            .map(|(name, module)| (name.to_string(), module.clone()))
            .collect();

        tracing::info!(
            modules = ?entries.iter().map(|e| e.name).collect::<Vec<_>>(),
            "Module dependency order resolved (topo)"
        );

        Ok(ModuleRegistry {
            modules: entries,
            grpc_hub,
            grpc_services,
        })
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

    // gRPC-related errors
    #[error("gRPC registration failed for module '{module}'")]
    GrpcRegister {
        module: String,
        #[source]
        source: anyhow::Error,
    },
    #[error("gRPC phase requires a hub: modules with capability 'grpc' found, but no module with capability 'grpc_hub'")]
    GrpcRequiresHub,
    #[error("multiple 'grpc_hub' modules detected; exactly one is allowed")]
    MultipleGrpcHubs,

    // Build/topo-sort errors
    #[error("unknown module '{0}'")]
    UnknownModule(String),
    #[error("module '{module}' depends on unknown '{depends_on}'")]
    UnknownDependency { module: String, depends_on: String },
    #[error("cyclic dependency detected: {}", path.join(" -> "))]
    CycleDetected { path: Vec<&'static str> },
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
    use std::sync::Arc;

    // Use the real contracts/context APIs from the crate to avoid type mismatches.
    use crate::context::ModuleCtx;
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

    /* ------------------------------- Tests ---------------------------- */

    #[test]
    fn topo_sort_happy_path() {
        let mut b = RegistryBuilder::default();
        // cores
        b.register_core_with_meta("core_a", &[], Arc::new(DummyCore));
        b.register_core_with_meta("core_b", &["core_a"], Arc::new(DummyCore));

        let reg = b.build_topo_sorted().unwrap();
        let order: Vec<_> = reg.modules().iter().map(|m| m.name).collect();
        assert_eq!(order, vec!["core_a", "core_b"]);
    }

    #[test]
    fn unknown_dependency_error() {
        let mut b = RegistryBuilder::default();
        b.register_core_with_meta("core_a", &["missing_dep"], Arc::new(DummyCore));

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
        b.register_core_with_meta("a", &["b"], Arc::new(DummyCore));
        b.register_core_with_meta("b", &["a"], Arc::new(DummyCore));

        let err = b.build_topo_sorted().unwrap_err();
        match err {
            RegistryError::CycleDetected { path } => {
                // Should contain both modules in the cycle
                assert!(path.contains(&"a"));
                assert!(path.contains(&"b"));
                assert!(path.len() >= 3); // At least a -> b -> a
            }
            other => panic!("expected CycleDetected, got: {other:?}"),
        }
    }

    #[test]
    fn complex_cycle_detection_with_path() {
        let mut b = RegistryBuilder::default();
        // Create a more complex cycle: a -> b -> c -> a
        b.register_core_with_meta("a", &["b"], Arc::new(DummyCore));
        b.register_core_with_meta("b", &["c"], Arc::new(DummyCore));
        b.register_core_with_meta("c", &["a"], Arc::new(DummyCore));
        // Add an unrelated module to ensure we only detect the actual cycle
        b.register_core_with_meta("d", &[], Arc::new(DummyCore));

        let err = b.build_topo_sorted().unwrap_err();
        match err {
            RegistryError::CycleDetected { path } => {
                // Should contain all modules in the cycle
                assert!(path.contains(&"a"));
                assert!(path.contains(&"b"));
                assert!(path.contains(&"c"));
                assert!(!path.contains(&"d")); // Should not include unrelated module
                assert!(path.len() >= 4); // At least a -> b -> c -> a

                // Verify the error message is helpful
                let error_msg = format!("{}", RegistryError::CycleDetected { path: path.clone() });
                assert!(error_msg.contains("cyclic dependency detected"));
                assert!(error_msg.contains("->"));
            }
            other => panic!("expected CycleDetected, got: {other:?}"),
        }
    }

    #[test]
    fn duplicate_core_reported_in_configuration_errors() {
        let mut b = RegistryBuilder::default();
        b.register_core_with_meta("a", &[], Arc::new(DummyCore));
        // duplicate
        b.register_core_with_meta("a", &[], Arc::new(DummyCore));

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
}
