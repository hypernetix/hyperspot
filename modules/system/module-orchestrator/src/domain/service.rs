use std::collections::HashSet;
use std::sync::Arc;

use modkit::registry::{
    ApiGatewayCap, CapabilitySet, GrpcHubCap, GrpcServiceCap, ModuleRegistry, RestApiCap,
    RunnableCap, SystemCap,
};
use modkit::runtime::ModuleManager;
use modkit_macros::domain_model;

use super::model::{DeploymentMode, InstanceInfo, ModuleInfo};

/// Lightweight compiled-module metadata (owned data, no trait objects).
#[domain_model]
struct CompiledModule {
    name: String,
    capabilities: Vec<String>,
    deps: Vec<String>,
}

/// Service that assembles module information from catalog and runtime data.
#[domain_model]
pub struct ModulesService {
    /// Compiled modules snapshot (built once at init, immutable after).
    compiled: Vec<CompiledModule>,
    /// Runtime module manager for live instance queries.
    module_manager: Arc<ModuleManager>,
}

impl ModulesService {
    /// Build from a live `ModuleRegistry` and a `ModuleManager`.
    ///
    /// Extracts module metadata (names, deps, capability labels) from the registry
    /// and drops the registry afterwards — no trait objects are kept.
    #[must_use]
    pub fn new(registry: &ModuleRegistry, module_manager: Arc<ModuleManager>) -> Self {
        let compiled: Vec<CompiledModule> = registry
            .modules()
            .iter()
            .map(|entry| CompiledModule {
                name: entry.name().to_owned(),
                capabilities: capability_labels(entry.caps()),
                deps: entry.deps().iter().map(|d| (*d).to_owned()).collect(),
            })
            .collect();

        Self {
            compiled,
            module_manager,
        }
    }

    /// List all registered modules, merging compile-time catalog data with runtime instances.
    #[must_use]
    pub fn list_modules(&self) -> Vec<ModuleInfo> {
        let mut modules = Vec::new();
        let mut seen_names = HashSet::new();

        // 1. Emit all compiled-in modules from the catalog.
        for cm in &self.compiled {
            seen_names.insert(cm.name.clone());

            let instances = self.get_module_instances(&cm.name);

            modules.push(ModuleInfo {
                name: cm.name.clone(),
                capabilities: cm.capabilities.clone(),
                dependencies: cm.deps.clone(),
                deployment_mode: DeploymentMode::CompiledIn,
                instances,
            });
        }

        // 2. Add any dynamically registered modules from ModuleManager
        //    that are not in the compiled catalog (external / out-of-process).
        for instance in self.module_manager.all_instances() {
            if seen_names.contains(&instance.module) {
                continue;
            }
            seen_names.insert(instance.module.clone());

            let instances = self.get_module_instances(&instance.module);

            modules.push(ModuleInfo {
                name: instance.module.clone(),
                capabilities: vec![],
                dependencies: vec![],
                deployment_mode: DeploymentMode::OutOfProcess,
                instances,
            });
        }

        // Sort by name for deterministic output
        modules.sort_by(|a, b| a.name.cmp(&b.name));

        modules
    }

    fn get_module_instances(&self, module_name: &str) -> Vec<InstanceInfo> {
        self.module_manager
            .instances_of(module_name)
            .into_iter()
            .map(|inst| {
                let grpc_services = inst
                    .grpc_services
                    .iter()
                    .map(|(name, ep)| (name.clone(), ep.uri.clone()))
                    .collect();

                InstanceInfo {
                    instance_id: inst.instance_id,
                    version: inst.version.clone(),
                    state: inst.state(),
                    grpc_services,
                }
            })
            .collect()
    }
}

/// Map a `CapabilitySet` to human-readable string labels.
///
/// Uses the existing `has::<T>()` API on `CapabilitySet` rather than adding
/// a new method to the framework.
fn capability_labels(caps: &CapabilitySet) -> Vec<String> {
    let mut labels = Vec::new();
    if caps.has_db() {
        labels.push("db".to_owned());
    }
    if caps.has::<RestApiCap>() {
        labels.push("rest".to_owned());
    }
    if caps.has::<ApiGatewayCap>() {
        labels.push("rest_host".to_owned());
    }
    if caps.has::<RunnableCap>() {
        labels.push("stateful".to_owned());
    }
    if caps.has::<SystemCap>() {
        labels.push("system".to_owned());
    }
    if caps.has::<GrpcHubCap>() {
        labels.push("grpc_hub".to_owned());
    }
    if caps.has::<GrpcServiceCap>() {
        labels.push("grpc".to_owned());
    }
    labels
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use modkit::registry::RegistryBuilder;
    use modkit::runtime::{Endpoint, InstanceState, ModuleInstance, ModuleManager};
    use uuid::Uuid;

    // ---- Test helpers ----

    // (name, deps, has_rest, has_system)
    type ModuleSpec = (&'static str, &'static [&'static str], bool, bool);

    #[domain_model]
    #[derive(Default)]
    struct DummyCore;
    #[async_trait::async_trait]
    impl modkit::Module for DummyCore {
        async fn init(&self, _ctx: &modkit::context::ModuleCtx) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[domain_model]
    #[derive(Default, Clone)]
    struct DummyRest;
    impl modkit::contracts::RestApiCapability for DummyRest {
        fn register_rest(
            &self,
            _ctx: &modkit::context::ModuleCtx,
            _router: axum::Router,
            _openapi: &dyn modkit::api::OpenApiRegistry,
        ) -> anyhow::Result<axum::Router> {
            Ok(axum::Router::new())
        }
    }

    #[domain_model]
    #[derive(Default)]
    struct DummySystem;
    #[async_trait::async_trait]
    impl modkit::contracts::SystemCapability for DummySystem {}

    fn build_registry(modules: &[ModuleSpec]) -> ModuleRegistry {
        let mut b = RegistryBuilder::default();
        for &(name, deps, has_rest, has_system) in modules {
            b.register_core_with_meta(name, deps, Arc::new(DummyCore));
            if has_rest {
                b.register_rest_with_meta(name, Arc::new(DummyRest));
            }
            if has_system {
                b.register_system_with_meta(name, Arc::new(DummySystem));
            }
        }
        b.build_topo_sorted().unwrap()
    }

    // ---- Tests ----

    #[test]
    fn list_compiled_in_modules_from_registry() {
        let registry = build_registry(&[
            ("api_gateway", &[], true, true),
            ("nodes_registry", &["api_gateway"], true, false),
        ]);
        let manager = Arc::new(ModuleManager::new());
        let svc = ModulesService::new(&registry, manager);
        let modules = svc.list_modules();

        assert_eq!(modules.len(), 2);
        // Sorted by name
        assert_eq!(modules[0].name, "api_gateway");
        assert_eq!(modules[0].deployment_mode, DeploymentMode::CompiledIn);
        assert!(modules[0].capabilities.contains(&"rest".to_owned()));
        assert!(modules[0].capabilities.contains(&"system".to_owned()));
        assert!(modules[0].instances.is_empty());

        assert_eq!(modules[1].name, "nodes_registry");
        assert_eq!(modules[1].dependencies, vec!["api_gateway"]);
    }

    #[test]
    fn dynamic_external_instances_appear_as_out_of_process() {
        let registry = build_registry(&[]);
        let manager = Arc::new(ModuleManager::new());

        let instance = Arc::new(
            ModuleInstance::new("external_svc", Uuid::new_v4())
                .with_version("2.0.0")
                .with_grpc_service("ext.Service", Endpoint::http("127.0.0.1", 9001)),
        );
        manager.register_instance(instance);

        let svc = ModulesService::new(&registry, manager);
        let modules = svc.list_modules();

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].name, "external_svc");
        assert_eq!(modules[0].deployment_mode, DeploymentMode::OutOfProcess);
        assert_eq!(modules[0].instances.len(), 1);
        assert_eq!(modules[0].instances[0].version, Some("2.0.0".to_owned()));
        assert!(
            modules[0].instances[0]
                .grpc_services
                .contains_key("ext.Service")
        );
    }

    #[test]
    fn compiled_in_modules_show_instances_from_manager() {
        let registry = build_registry(&[("grpc_hub", &[], false, true)]);
        let manager = Arc::new(ModuleManager::new());

        let instance =
            Arc::new(ModuleInstance::new("grpc_hub", Uuid::new_v4()).with_version("0.1.0"));
        manager.register_instance(instance);

        let svc = ModulesService::new(&registry, manager);
        let modules = svc.list_modules();

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].name, "grpc_hub");
        assert_eq!(modules[0].deployment_mode, DeploymentMode::CompiledIn);
        assert_eq!(modules[0].instances.len(), 1);
    }

    #[test]
    fn instance_state_maps_correctly() {
        let registry = build_registry(&[]);
        let manager = Arc::new(ModuleManager::new());

        let instance = Arc::new(ModuleInstance::new("svc", Uuid::new_v4()));
        // Default state is Registered
        manager.register_instance(instance);

        let svc = ModulesService::new(&registry, manager);
        let modules = svc.list_modules();

        assert_eq!(modules[0].instances[0].state, InstanceState::Registered);
    }

    #[test]
    fn result_is_sorted_by_name() {
        let registry =
            build_registry(&[("zebra", &[], false, false), ("alpha", &[], false, false)]);
        let manager = Arc::new(ModuleManager::new());

        let svc = ModulesService::new(&registry, manager);
        let modules = svc.list_modules();

        assert_eq!(modules[0].name, "alpha");
        assert_eq!(modules[1].name, "zebra");
    }
}
