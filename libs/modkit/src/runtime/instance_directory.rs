//! Instance Directory - tracks all active module instances in the runtime

use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

/// Common module identifier
pub type ModuleName = &'static str;

/// Represents an endpoint where a module instance can be reached
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Endpoint {
    pub uri: String,
}

impl Endpoint {
    pub fn from_uri<S: Into<String>>(s: S) -> Self {
        Self { uri: s.into() }
    }

    pub fn uds(path: impl AsRef<std::path::Path>) -> Self {
        Self {
            uri: format!("unix://{}", path.as_ref().display()),
        }
    }

    pub fn tcp(host: &str, port: u16) -> Self {
        Self {
            uri: format!("tcp://{}:{}", host, port),
        }
    }
}

/// Represents a single instance of a module
#[derive(Clone, Debug)]
pub struct ModuleInstance {
    pub module: ModuleName,
    pub instance_id: String,
    pub control: Option<Endpoint>,
    pub rest_invoke: Option<Endpoint>,
    pub grpc_services: HashMap<String, Endpoint>,
    pub ready: bool,
    pub last_heartbeat: Instant,
    pub version: Option<String>,
}

impl ModuleInstance {
    pub fn new(module: ModuleName, instance_id: impl Into<String>) -> Self {
        Self {
            module,
            instance_id: instance_id.into(),
            control: None,
            rest_invoke: None,
            grpc_services: HashMap::new(),
            ready: false,
            last_heartbeat: Instant::now(),
            version: None,
        }
    }

    pub fn with_control(mut self, ep: Endpoint) -> Self {
        self.control = Some(ep);
        self
    }

    pub fn with_rest_invoke(mut self, ep: Endpoint) -> Self {
        self.rest_invoke = Some(ep);
        self
    }

    pub fn with_version(mut self, v: impl Into<String>) -> Self {
        self.version = Some(v.into());
        self
    }

    pub fn with_grpc_service(mut self, name: impl Into<String>, ep: Endpoint) -> Self {
        self.grpc_services.insert(name.into(), ep);
        self
    }
}

/// Directory that tracks all active module instances
pub struct InstanceDirectory {
    inner: DashMap<ModuleName, Vec<Arc<ModuleInstance>>>,
    rr_counters: DashMap<ModuleName, usize>,
}

impl std::fmt::Debug for InstanceDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let modules: Vec<String> = self.inner.iter().map(|e| e.key().to_string()).collect();
        f.debug_struct("InstanceDirectory")
            .field("instances_count", &self.inner.len())
            .field("modules", &modules)
            .finish()
    }
}

impl InstanceDirectory {
    pub fn new() -> Self {
        Self {
            inner: DashMap::new(),
            rr_counters: DashMap::new(),
        }
    }

    /// Register or update a module instance
    pub fn register_instance(&self, instance: Arc<ModuleInstance>) {
        let module = instance.module;
        let mut vec = self.inner.entry(module).or_default();
        // replace by instance_id if it already exists
        if let Some(pos) = vec
            .iter()
            .position(|i| i.instance_id == instance.instance_id)
        {
            vec[pos] = instance;
        } else {
            vec.push(instance);
        }
    }

    /// Mark an instance as ready
    pub fn mark_ready(&self, module: ModuleName, instance_id: &str) {
        if let Some(mut vec) = self.inner.get_mut(module) {
            if let Some(inst) = vec.iter_mut().find(|i| i.instance_id == instance_id) {
                Arc::make_mut(inst).ready = true;
            }
        }
    }

    /// Update the heartbeat timestamp for an instance
    pub fn update_heartbeat(&self, module: ModuleName, instance_id: &str, at: Instant) {
        if let Some(mut vec) = self.inner.get_mut(module) {
            if let Some(inst) = vec.iter_mut().find(|i| i.instance_id == instance_id) {
                Arc::make_mut(inst).last_heartbeat = at;
            }
        }
    }

    /// Get all instances of a specific module
    pub fn instances_of(&self, module: ModuleName) -> Vec<Arc<ModuleInstance>> {
        self.inner
            .get(module)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Get all instances across all modules
    pub fn all_instances(&self) -> Vec<Arc<ModuleInstance>> {
        self.inner
            .iter()
            .flat_map(|entry| entry.value().clone())
            .collect()
    }

    /// Pick an instance using round-robin selection
    pub fn pick_instance_round_robin(&self, module: ModuleName) -> Option<Arc<ModuleInstance>> {
        let instances_entry = self.inner.get(module)?;
        let instances = instances_entry.value();

        if instances.is_empty() {
            return None;
        }

        let len = instances.len();
        let mut counter = self.rr_counters.entry(module).or_insert(0);
        let idx = *counter % len;
        *counter = (*counter + 1) % len;

        instances.get(idx).cloned()
    }
}

impl Default for InstanceDirectory {
    fn default() -> Self {
        Self::new()
    }
}

/// Global instance directory for system modules
static GLOBAL_INSTANCE_DIR: OnceLock<Arc<InstanceDirectory>> = OnceLock::new();

/// Set the global instance directory (called once by runtime bootstrap)
pub fn set_global_instance_directory(dir: Arc<InstanceDirectory>) {
    if GLOBAL_INSTANCE_DIR.set(dir).is_err() {
        panic!("GLOBAL_INSTANCE_DIR already initialized");
    }
}

/// Get the global instance directory (used by system modules)
pub fn get_global_instance_directory() -> Arc<InstanceDirectory> {
    GLOBAL_INSTANCE_DIR
        .get()
        .cloned()
        .expect("GLOBAL_INSTANCE_DIR not initialized")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_register_and_retrieve_instances() {
        let dir = InstanceDirectory::new();
        let instance = Arc::new(
            ModuleInstance::new("test_module", "instance1")
                .with_control(Endpoint::tcp("localhost", 8080))
                .with_version("1.0.0"),
        );

        dir.register_instance(instance.clone());

        let instances = dir.instances_of("test_module");
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].instance_id, "instance1");
        assert_eq!(instances[0].module, "test_module");
        assert_eq!(instances[0].version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_register_multiple_instances() {
        let dir = InstanceDirectory::new();

        let instance1 = Arc::new(ModuleInstance::new("test_module", "instance1"));
        let instance2 = Arc::new(ModuleInstance::new("test_module", "instance2"));

        dir.register_instance(instance1);
        dir.register_instance(instance2);

        let instances = dir.instances_of("test_module");
        assert_eq!(instances.len(), 2);

        let ids: Vec<_> = instances.iter().map(|i| i.instance_id.as_str()).collect();
        assert!(ids.contains(&"instance1"));
        assert!(ids.contains(&"instance2"));
    }

    #[test]
    fn test_update_existing_instance() {
        let dir = InstanceDirectory::new();

        let instance1 =
            Arc::new(ModuleInstance::new("test_module", "instance1").with_version("1.0.0"));
        dir.register_instance(instance1);

        let instance1_updated =
            Arc::new(ModuleInstance::new("test_module", "instance1").with_version("2.0.0"));
        dir.register_instance(instance1_updated);

        let instances = dir.instances_of("test_module");
        assert_eq!(instances.len(), 1, "Should not duplicate instance");
        assert_eq!(instances[0].version, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_mark_ready() {
        let dir = InstanceDirectory::new();
        let instance = Arc::new(ModuleInstance::new("test_module", "instance1"));

        assert!(!instance.ready);
        dir.register_instance(instance);

        dir.mark_ready("test_module", "instance1");

        let instances = dir.instances_of("test_module");
        assert_eq!(instances.len(), 1);
        assert!(instances[0].ready);
    }

    #[test]
    fn test_update_heartbeat() {
        let dir = InstanceDirectory::new();
        let instance = Arc::new(ModuleInstance::new("test_module", "instance1"));
        let initial_heartbeat = instance.last_heartbeat;

        dir.register_instance(instance);

        // Sleep to ensure time difference
        sleep(Duration::from_millis(10));

        let new_heartbeat = Instant::now();
        dir.update_heartbeat("test_module", "instance1", new_heartbeat);

        let instances = dir.instances_of("test_module");
        assert!(instances[0].last_heartbeat > initial_heartbeat);
    }

    #[test]
    fn test_all_instances() {
        let dir = InstanceDirectory::new();

        let instance1 = Arc::new(ModuleInstance::new("module_a", "instance1"));
        let instance2 = Arc::new(ModuleInstance::new("module_b", "instance2"));
        let instance3 = Arc::new(ModuleInstance::new("module_a", "instance3"));

        dir.register_instance(instance1);
        dir.register_instance(instance2);
        dir.register_instance(instance3);

        let all = dir.all_instances();
        assert_eq!(all.len(), 3);

        let modules: Vec<_> = all.iter().map(|i| i.module).collect();
        assert_eq!(modules.iter().filter(|&&m| m == "module_a").count(), 2);
        assert_eq!(modules.iter().filter(|&&m| m == "module_b").count(), 1);
    }

    #[test]
    fn test_pick_instance_round_robin() {
        let dir = InstanceDirectory::new();

        let instance1 = Arc::new(ModuleInstance::new("test_module", "instance1"));
        let instance2 = Arc::new(ModuleInstance::new("test_module", "instance2"));

        dir.register_instance(instance1);
        dir.register_instance(instance2);

        // Pick three times to verify round-robin behavior
        let picked1 = dir.pick_instance_round_robin("test_module").unwrap();
        let picked2 = dir.pick_instance_round_robin("test_module").unwrap();
        let picked3 = dir.pick_instance_round_robin("test_module").unwrap();

        let ids = vec![
            picked1.instance_id.as_str(),
            picked2.instance_id.as_str(),
            picked3.instance_id.as_str(),
        ];

        // With 2 instances, we expect round-robin pattern like A, B, A
        // Check that both instance IDs appear and that at least one repeats
        assert!(ids.contains(&"instance1"));
        assert!(ids.contains(&"instance2"));
        // First and third pick should be the same (round-robin wraps)
        assert_eq!(picked1.instance_id, picked3.instance_id);
        // Second pick should be different from the first
        assert_ne!(picked1.instance_id, picked2.instance_id);
    }

    #[test]
    fn test_pick_instance_none_available() {
        let dir = InstanceDirectory::new();
        let picked = dir.pick_instance_round_robin("nonexistent_module");
        assert!(picked.is_none());
    }

    #[test]
    fn test_endpoint_creation() {
        let tcp_ep = Endpoint::tcp("localhost", 8080);
        assert_eq!(tcp_ep.uri, "tcp://localhost:8080");

        let uds_ep = Endpoint::uds("/tmp/socket.sock");
        assert!(uds_ep.uri.starts_with("unix://"));
        assert!(uds_ep.uri.contains("socket.sock"));

        let custom_ep = Endpoint::from_uri("http://example.com");
        assert_eq!(custom_ep.uri, "http://example.com");
    }

    #[test]
    fn test_module_instance_builder() {
        let instance = ModuleInstance::new("test_module", "instance1")
            .with_control(Endpoint::tcp("localhost", 8080))
            .with_rest_invoke(Endpoint::tcp("localhost", 8081))
            .with_version("1.2.3")
            .with_grpc_service("service1", Endpoint::tcp("localhost", 8082))
            .with_grpc_service("service2", Endpoint::tcp("localhost", 8083));

        assert_eq!(instance.module, "test_module");
        assert_eq!(instance.instance_id, "instance1");
        assert!(instance.control.is_some());
        assert!(instance.rest_invoke.is_some());
        assert_eq!(instance.version, Some("1.2.3".to_string()));
        assert_eq!(instance.grpc_services.len(), 2);
        assert!(instance.grpc_services.contains_key("service1"));
        assert!(instance.grpc_services.contains_key("service2"));
    }

    #[test]
    fn test_instances_of_empty() {
        let dir = InstanceDirectory::new();
        let instances = dir.instances_of("nonexistent");
        assert!(instances.is_empty());
    }
}
