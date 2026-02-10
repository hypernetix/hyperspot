use std::collections::HashSet;
use std::sync::Arc;

use modkit::registry::ModuleRegistryCatalog;
use modkit::runtime::ModuleManager;

use super::model::{DeploymentMode, InstanceInfo, ModuleInfo};

/// Service that assembles module information from catalog and runtime data.
pub struct ModulesService {
    module_catalog: Arc<ModuleRegistryCatalog>,
    module_manager: Arc<ModuleManager>,
    external_module_names: Arc<HashSet<String>>,
}

impl ModulesService {
    #[must_use]
    pub fn new(
        module_catalog: Arc<ModuleRegistryCatalog>,
        module_manager: Arc<ModuleManager>,
        external_module_names: Arc<HashSet<String>>,
    ) -> Self {
        Self {
            module_catalog,
            module_manager,
            external_module_names,
        }
    }

    /// List all registered modules, merging compile-time catalog data with runtime instances.
    #[must_use]
    pub fn list_modules(&self) -> Vec<ModuleInfo> {
        let mut modules = Vec::new();
        let mut seen_names = HashSet::new();

        // 1. Emit all compiled-in modules from the catalog.
        //    If a module is also in external_module_names, it means the config overrides
        //    it to run out-of-process.
        for descriptor in &self.module_catalog.modules {
            seen_names.insert(descriptor.name.clone());

            let deployment_mode = if self.external_module_names.contains(&descriptor.name) {
                DeploymentMode::OutOfProcess
            } else {
                DeploymentMode::CompiledIn
            };

            let instances = self.get_module_instances(&descriptor.name);

            modules.push(ModuleInfo {
                name: descriptor.name.clone(),
                capabilities: descriptor.capability_labels.clone(),
                dependencies: descriptor.deps.clone(),
                deployment_mode,
                instances,
            });
        }

        // 2. Add external modules from config that haven't been seen yet
        //    (they may or may not have registered instances)
        for ext_name in self.external_module_names.iter() {
            if seen_names.contains(ext_name) {
                continue;
            }
            seen_names.insert(ext_name.clone());

            let instances = self.get_module_instances(ext_name);

            modules.push(ModuleInfo {
                name: ext_name.clone(),
                capabilities: vec![],
                dependencies: vec![],
                deployment_mode: DeploymentMode::OutOfProcess,
                instances,
            });
        }

        // 3. Add any dynamically registered modules from ModuleManager
        //    that are not in the catalog or external config
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

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use modkit::registry::{ModuleDescriptor, ModuleRegistryCatalog};
    use modkit::runtime::{Endpoint, InstanceState, ModuleInstance, ModuleManager};
    use uuid::Uuid;

    #[test]
    fn list_compiled_in_modules_from_catalog() {
        let catalog = Arc::new(ModuleRegistryCatalog {
            modules: vec![
                ModuleDescriptor {
                    name: "api_gateway".to_owned(),
                    deps: vec![],
                    capability_labels: vec!["rest".to_owned(), "system".to_owned()],
                },
                ModuleDescriptor {
                    name: "nodes_registry".to_owned(),
                    deps: vec!["api_gateway".to_owned()],
                    capability_labels: vec!["rest".to_owned()],
                },
            ],
        });
        let manager = Arc::new(ModuleManager::new());
        let ext_names = Arc::new(HashSet::new());

        let svc = ModulesService::new(catalog, manager, ext_names);
        let modules = svc.list_modules();

        assert_eq!(modules.len(), 2);
        // Sorted by name
        assert_eq!(modules[0].name, "api_gateway");
        assert_eq!(modules[0].deployment_mode, DeploymentMode::CompiledIn);
        assert_eq!(modules[0].capabilities, vec!["rest", "system"]);
        assert!(modules[0].instances.is_empty());

        assert_eq!(modules[1].name, "nodes_registry");
        assert_eq!(modules[1].dependencies, vec!["api_gateway"]);
    }

    #[test]
    fn external_modules_from_config_appear_even_without_instances() {
        let catalog = Arc::new(ModuleRegistryCatalog { modules: vec![] });
        let manager = Arc::new(ModuleManager::new());
        let ext_names = Arc::new(HashSet::from(["calculator".to_owned()]));

        let svc = ModulesService::new(catalog, manager, ext_names);
        let modules = svc.list_modules();

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].name, "calculator");
        assert_eq!(modules[0].deployment_mode, DeploymentMode::OutOfProcess);
        assert!(modules[0].instances.is_empty());
    }

    #[test]
    fn dynamic_external_instances_appear_as_out_of_process() {
        let catalog = Arc::new(ModuleRegistryCatalog { modules: vec![] });
        let manager = Arc::new(ModuleManager::new());

        let instance = Arc::new(
            ModuleInstance::new("external_svc", Uuid::new_v4())
                .with_version("2.0.0")
                .with_grpc_service("ext.Service", Endpoint::http("127.0.0.1", 9001)),
        );
        manager.register_instance(instance);

        let ext_names = Arc::new(HashSet::new());
        let svc = ModulesService::new(catalog, manager, ext_names);
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
        let catalog = Arc::new(ModuleRegistryCatalog {
            modules: vec![ModuleDescriptor {
                name: "grpc_hub".to_owned(),
                deps: vec![],
                capability_labels: vec!["grpc_hub".to_owned(), "system".to_owned()],
            }],
        });
        let manager = Arc::new(ModuleManager::new());

        let instance =
            Arc::new(ModuleInstance::new("grpc_hub", Uuid::new_v4()).with_version("0.1.0"));
        manager.register_instance(instance);

        let ext_names = Arc::new(HashSet::new());
        let svc = ModulesService::new(catalog, manager, ext_names);
        let modules = svc.list_modules();

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].name, "grpc_hub");
        assert_eq!(modules[0].deployment_mode, DeploymentMode::CompiledIn);
        assert_eq!(modules[0].instances.len(), 1);
    }

    #[test]
    fn compiled_in_module_in_external_config_shows_out_of_process() {
        let catalog = Arc::new(ModuleRegistryCatalog {
            modules: vec![ModuleDescriptor {
                name: "calculator".to_owned(),
                deps: vec![],
                capability_labels: vec!["grpc".to_owned()],
            }],
        });
        let manager = Arc::new(ModuleManager::new());
        let ext_names = Arc::new(HashSet::from(["calculator".to_owned()]));

        let svc = ModulesService::new(catalog, manager, ext_names);
        let modules = svc.list_modules();

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].name, "calculator");
        assert_eq!(modules[0].deployment_mode, DeploymentMode::OutOfProcess);
        // Still has capabilities from compile-time catalog
        assert_eq!(modules[0].capabilities, vec!["grpc"]);
    }

    #[test]
    fn instance_state_maps_correctly() {
        let catalog = Arc::new(ModuleRegistryCatalog { modules: vec![] });
        let manager = Arc::new(ModuleManager::new());

        let instance = Arc::new(ModuleInstance::new("svc", Uuid::new_v4()));
        // Default state is Registered
        manager.register_instance(instance);

        let ext_names = Arc::new(HashSet::new());
        let svc = ModulesService::new(catalog, manager, ext_names);
        let modules = svc.list_modules();

        assert_eq!(modules[0].instances[0].state, InstanceState::Registered);
    }

    #[test]
    fn result_is_sorted_by_name() {
        let catalog = Arc::new(ModuleRegistryCatalog {
            modules: vec![
                ModuleDescriptor {
                    name: "zebra".to_owned(),
                    deps: vec![],
                    capability_labels: vec![],
                },
                ModuleDescriptor {
                    name: "alpha".to_owned(),
                    deps: vec![],
                    capability_labels: vec![],
                },
            ],
        });
        let manager = Arc::new(ModuleManager::new());
        let ext_names = Arc::new(HashSet::new());

        let svc = ModulesService::new(catalog, manager, ext_names);
        let modules = svc.list_modules();

        assert_eq!(modules[0].name, "alpha");
        assert_eq!(modules[1].name, "zebra");
    }
}
