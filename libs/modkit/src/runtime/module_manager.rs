//! Module Manager - tracks and manages all live module instances in the runtime

use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Represents an endpoint where a module instance can be reached
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Endpoint {
    pub uri: String,
}

/// Typed view of an endpoint for parsing and matching
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EndpointKind {
    /// TCP endpoint with resolved socket address
    Tcp(std::net::SocketAddr),
    /// Unix domain socket with file path
    Uds(std::path::PathBuf),
    /// Other/unparsed endpoint URI
    Other(String),
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

    pub fn http(host: &str, port: u16) -> Self {
        Self {
            uri: format!("http://{}:{}", host, port),
        }
    }

    pub fn https(host: &str, port: u16) -> Self {
        Self {
            uri: format!("https://{}:{}", host, port),
        }
    }

    /// Parse the endpoint URI into a typed view
    pub fn kind(&self) -> EndpointKind {
        if let Some(rest) = self.uri.strip_prefix("unix://") {
            return EndpointKind::Uds(std::path::PathBuf::from(rest));
        }
        if let Some(rest) = self.uri.strip_prefix("http://") {
            if let Ok(addr) = rest.parse::<std::net::SocketAddr>() {
                return EndpointKind::Tcp(addr);
            }
        }
        if let Some(rest) = self.uri.strip_prefix("https://") {
            if let Ok(addr) = rest.parse::<std::net::SocketAddr>() {
                return EndpointKind::Tcp(addr);
            }
        }
        EndpointKind::Other(self.uri.clone())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstanceState {
    Registered,
    Ready,
    Healthy,
    Quarantined,
    Draining,
}

/// Runtime state of an instance (guarded by RwLock for safe mutation)
#[derive(Clone, Debug)]
pub struct InstanceRuntimeState {
    pub last_heartbeat: Instant,
    pub state: InstanceState,
}

/// Represents a single instance of a module
#[derive(Debug)]
pub struct ModuleInstance {
    pub module: String,
    pub instance_id: Uuid,
    pub control: Option<Endpoint>,
    pub grpc_services: HashMap<String, Endpoint>,
    pub version: Option<String>,
    inner: Arc<parking_lot::RwLock<InstanceRuntimeState>>,
}

impl Clone for ModuleInstance {
    fn clone(&self) -> Self {
        Self {
            module: self.module.clone(),
            instance_id: self.instance_id,
            control: self.control.clone(),
            grpc_services: self.grpc_services.clone(),
            version: self.version.clone(),
            inner: Arc::clone(&self.inner),
        }
    }
}

impl ModuleInstance {
    pub fn new(module: impl Into<String>, instance_id: Uuid) -> Self {
        Self {
            module: module.into(),
            instance_id,
            control: None,
            grpc_services: HashMap::new(),
            version: None,
            inner: Arc::new(parking_lot::RwLock::new(InstanceRuntimeState {
                last_heartbeat: Instant::now(),
                state: InstanceState::Registered,
            })),
        }
    }

    pub fn with_control(mut self, ep: Endpoint) -> Self {
        self.control = Some(ep);
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

    /// Get the current state of this instance
    pub fn state(&self) -> InstanceState {
        self.inner.read().state
    }

    /// Get the last heartbeat timestamp
    pub fn last_heartbeat(&self) -> Instant {
        self.inner.read().last_heartbeat
    }
}

/// Central registry that tracks all running module instances in the system.
/// Provides discovery, health tracking, and round-robin load balancing.
#[derive(Clone)]
pub struct ModuleManager {
    inner: DashMap<String, Vec<Arc<ModuleInstance>>>,
    rr_counters: DashMap<String, usize>,
    hb_ttl: Duration,
    hb_grace: Duration,
}

impl std::fmt::Debug for ModuleManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let modules: Vec<String> = self.inner.iter().map(|e| e.key().to_string()).collect();
        f.debug_struct("ModuleManager")
            .field("instances_count", &self.inner.len())
            .field("modules", &modules)
            .field("heartbeat_ttl", &self.hb_ttl)
            .field("heartbeat_grace", &self.hb_grace)
            .finish()
    }
}

impl ModuleManager {
    pub fn new() -> Self {
        Self {
            inner: DashMap::new(),
            rr_counters: DashMap::new(),
            hb_ttl: Duration::from_secs(15),
            hb_grace: Duration::from_secs(30),
        }
    }

    pub fn with_heartbeat_policy(mut self, ttl: Duration, grace: Duration) -> Self {
        self.hb_ttl = ttl;
        self.hb_grace = grace;
        self
    }

    /// Register or update a module instance
    pub fn register_instance(&self, instance: Arc<ModuleInstance>) {
        let module = instance.module.clone();
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
    pub fn mark_ready(&self, module: &str, instance_id: Uuid) {
        if let Some(mut vec) = self.inner.get_mut(module) {
            if let Some(inst) = vec.iter_mut().find(|i| i.instance_id == instance_id) {
                let mut state = inst.inner.write();
                state.state = InstanceState::Ready;
            }
        }
    }

    /// Update the heartbeat timestamp for an instance
    pub fn update_heartbeat(&self, module: &str, instance_id: Uuid, at: Instant) {
        if let Some(mut vec) = self.inner.get_mut(module) {
            if let Some(inst) = vec.iter_mut().find(|i| i.instance_id == instance_id) {
                let mut state = inst.inner.write();
                state.last_heartbeat = at;
                // Transition Registered -> Healthy on first heartbeat
                if state.state == InstanceState::Registered {
                    state.state = InstanceState::Healthy;
                }
            }
        }
    }

    /// Mark an instance as quarantined
    pub fn mark_quarantined(&self, module: &str, instance_id: Uuid) {
        if let Some(mut vec) = self.inner.get_mut(module) {
            if let Some(inst) = vec.iter_mut().find(|i| i.instance_id == instance_id) {
                inst.inner.write().state = InstanceState::Quarantined;
            }
        }
    }

    /// Mark an instance as draining (graceful shutdown in progress)
    pub fn mark_draining(&self, module: &str, instance_id: Uuid) {
        if let Some(mut vec) = self.inner.get_mut(module) {
            if let Some(inst) = vec.iter_mut().find(|i| i.instance_id == instance_id) {
                inst.inner.write().state = InstanceState::Draining;
            }
        }
    }

    /// Remove an instance from the directory
    pub fn deregister(&self, module: &str, instance_id: Uuid) {
        let mut remove_module = false;
        {
            if let Some(mut vec) = self.inner.get_mut(module) {
                let list = vec.value_mut();
                list.retain(|inst| inst.instance_id != instance_id);
                if list.is_empty() {
                    remove_module = true;
                }
            }
        }

        if remove_module {
            self.inner.remove(module);
            self.rr_counters.remove(module);
        }
    }

    /// Get all instances of a specific module
    pub fn instances_of(&self, module: &str) -> Vec<Arc<ModuleInstance>> {
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

    /// Quarantine or evict stale instances based on heartbeat policy
    pub fn evict_stale(&self, now: Instant) {
        use InstanceState::*;
        let mut empty_modules = Vec::new();

        for mut entry in self.inner.iter_mut() {
            let module = entry.key().clone();
            let vec = entry.value_mut();
            vec.retain(|inst| {
                let state = inst.inner.read();
                let age = now.saturating_duration_since(state.last_heartbeat);

                // Quarantine instances that have exceeded TTL
                if age >= self.hb_ttl && !matches!(state.state, Quarantined | Draining) {
                    drop(state); // Release read lock before write
                    inst.inner.write().state = Quarantined;
                    return true; // Keep quarantined instances for now
                }

                // Evict quarantined instances that exceed grace period
                if state.state == Quarantined && age >= self.hb_ttl + self.hb_grace {
                    return false; // Remove from directory
                }

                true
            });

            if vec.is_empty() {
                empty_modules.push(module);
            }
        }

        for module in empty_modules {
            self.inner.remove(&module);
            self.rr_counters.remove(&module);
        }
    }

    /// Pick an instance using round-robin selection, preferring healthy instances
    pub fn pick_instance_round_robin(&self, module: &str) -> Option<Arc<ModuleInstance>> {
        let instances_entry = self.inner.get(module)?;
        let instances = instances_entry.value();

        if instances.is_empty() {
            return None;
        }

        // Prefer healthy or ready instances
        let healthy: Vec<_> = instances
            .iter()
            .filter(|inst| matches!(inst.state(), InstanceState::Healthy | InstanceState::Ready))
            .cloned()
            .collect();

        let candidates: Vec<_> = if healthy.is_empty() {
            instances.to_vec()
        } else {
            healthy
        };

        if candidates.is_empty() {
            return None;
        }

        let len = candidates.len();
        let mut counter = self.rr_counters.entry(module.to_string()).or_insert(0);
        let idx = *counter % len;
        *counter = (*counter + 1) % len;

        candidates.get(idx).cloned()
    }

    /// Pick a service endpoint using round-robin, returning (module, instance, endpoint).
    /// Prefers healthy/ready instances and automatically rotates among them.
    pub fn pick_service_round_robin(
        &self,
        service_name: &str,
    ) -> Option<(String, Arc<ModuleInstance>, Endpoint)> {
        // Collect all instances that provide this service
        let mut candidates = Vec::new();
        for entry in self.inner.iter() {
            let module = entry.key().clone();
            for inst in entry.value().iter() {
                if let Some(ep) = inst.grpc_services.get(service_name) {
                    let state = inst.state();
                    if matches!(state, InstanceState::Healthy | InstanceState::Ready) {
                        candidates.push((module.clone(), inst.clone(), ep.clone()));
                    }
                }
            }
        }

        if candidates.is_empty() {
            return None;
        }

        // Use a counter keyed by service name for round-robin
        let len = candidates.len();
        let service_key = service_name.to_string();
        let mut counter = self.rr_counters.entry(service_key).or_insert(0);
        let idx = *counter % len;
        *counter = (*counter + 1) % len;

        candidates.get(idx).cloned()
    }
}

impl Default for ModuleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_register_and_retrieve_instances() {
        let dir = ModuleManager::new();
        let instance_id = Uuid::new_v4();
        let instance = Arc::new(
            ModuleInstance::new("test_module", instance_id)
                .with_control(Endpoint::http("localhost", 8080))
                .with_version("1.0.0"),
        );

        dir.register_instance(instance.clone());

        let instances = dir.instances_of("test_module");
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].instance_id, instance_id);
        assert_eq!(instances[0].module, "test_module");
        assert_eq!(instances[0].version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_register_multiple_instances() {
        let dir = ModuleManager::new();

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let instance1 = Arc::new(ModuleInstance::new("test_module", id1));
        let instance2 = Arc::new(ModuleInstance::new("test_module", id2));

        dir.register_instance(instance1);
        dir.register_instance(instance2);

        let instances = dir.instances_of("test_module");
        assert_eq!(instances.len(), 2);

        let ids: Vec<_> = instances.iter().map(|i| i.instance_id).collect();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[test]
    fn test_update_existing_instance() {
        let dir = ModuleManager::new();
        let instance_id = Uuid::new_v4();

        let instance1 =
            Arc::new(ModuleInstance::new("test_module", instance_id).with_version("1.0.0"));
        dir.register_instance(instance1);

        let instance1_updated =
            Arc::new(ModuleInstance::new("test_module", instance_id).with_version("2.0.0"));
        dir.register_instance(instance1_updated);

        let instances = dir.instances_of("test_module");
        assert_eq!(instances.len(), 1, "Should not duplicate instance");
        assert_eq!(instances[0].version, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_mark_ready() {
        let dir = ModuleManager::new();
        let instance_id = Uuid::new_v4();
        let instance = Arc::new(ModuleInstance::new("test_module", instance_id));

        dir.register_instance(instance);

        dir.mark_ready("test_module", instance_id);

        let instances = dir.instances_of("test_module");
        assert_eq!(instances.len(), 1);
        assert!(matches!(instances[0].state(), InstanceState::Ready));
    }

    #[test]
    fn test_update_heartbeat() {
        let dir = ModuleManager::new();
        let instance_id = Uuid::new_v4();
        let instance = Arc::new(ModuleInstance::new("test_module", instance_id));
        let initial_heartbeat = instance.last_heartbeat();

        dir.register_instance(instance);

        // Sleep to ensure time difference
        sleep(Duration::from_millis(10));

        let new_heartbeat = Instant::now();
        dir.update_heartbeat("test_module", instance_id, new_heartbeat);

        let instances = dir.instances_of("test_module");
        assert!(instances[0].last_heartbeat() > initial_heartbeat);
        assert!(matches!(instances[0].state(), InstanceState::Healthy));
    }

    #[test]
    fn test_all_instances() {
        let dir = ModuleManager::new();

        let instance1 = Arc::new(ModuleInstance::new("module_a", Uuid::new_v4()));
        let instance2 = Arc::new(ModuleInstance::new("module_b", Uuid::new_v4()));
        let instance3 = Arc::new(ModuleInstance::new("module_a", Uuid::new_v4()));

        dir.register_instance(instance1);
        dir.register_instance(instance2);
        dir.register_instance(instance3);

        let all = dir.all_instances();
        assert_eq!(all.len(), 3);

        let modules: Vec<_> = all.iter().map(|i| i.module.as_str()).collect();
        assert_eq!(modules.iter().filter(|&m| *m == "module_a").count(), 2);
        assert_eq!(modules.iter().filter(|&m| *m == "module_b").count(), 1);
    }

    #[test]
    fn test_pick_instance_round_robin() {
        let dir = ModuleManager::new();

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let instance1 = Arc::new(ModuleInstance::new("test_module", id1));
        let instance2 = Arc::new(ModuleInstance::new("test_module", id2));

        dir.register_instance(instance1);
        dir.register_instance(instance2);

        // Pick three times to verify round-robin behavior
        let picked1 = dir.pick_instance_round_robin("test_module").unwrap();
        let picked2 = dir.pick_instance_round_robin("test_module").unwrap();
        let picked3 = dir.pick_instance_round_robin("test_module").unwrap();

        let ids = [
            picked1.instance_id,
            picked2.instance_id,
            picked3.instance_id,
        ];

        // With 2 instances, we expect round-robin pattern like A, B, A
        // Check that both instance IDs appear and that at least one repeats
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
        // First and third pick should be the same (round-robin wraps)
        assert_eq!(picked1.instance_id, picked3.instance_id);
        // Second pick should be different from the first
        assert_ne!(picked1.instance_id, picked2.instance_id);
    }

    #[test]
    fn test_pick_instance_none_available() {
        let dir = ModuleManager::new();
        let picked = dir.pick_instance_round_robin("nonexistent_module");
        assert!(picked.is_none());
    }

    #[test]
    fn test_endpoint_creation() {
        let http_ep = Endpoint::http("localhost", 8080);
        assert_eq!(http_ep.uri, "http://localhost:8080");

        let https_ep = Endpoint::https("localhost", 8443);
        assert_eq!(https_ep.uri, "https://localhost:8443");

        let uds_ep = Endpoint::uds("/tmp/socket.sock");
        assert!(uds_ep.uri.starts_with("unix://"));
        assert!(uds_ep.uri.contains("socket.sock"));

        let custom_ep = Endpoint::from_uri("http://example.com");
        assert_eq!(custom_ep.uri, "http://example.com");
    }

    #[test]
    fn test_endpoint_kind() {
        let http_ep = Endpoint::http("127.0.0.1", 8080);
        match http_ep.kind() {
            EndpointKind::Tcp(addr) => {
                assert_eq!(addr.ip().to_string(), "127.0.0.1");
                assert_eq!(addr.port(), 8080);
            }
            _ => panic!("Expected TCP endpoint for http"),
        }

        let https_ep = Endpoint::https("127.0.0.1", 8443);
        match https_ep.kind() {
            EndpointKind::Tcp(addr) => {
                assert_eq!(addr.ip().to_string(), "127.0.0.1");
                assert_eq!(addr.port(), 8443);
            }
            _ => panic!("Expected TCP endpoint for https"),
        }

        let uds_ep = Endpoint::uds("/tmp/test.sock");
        match uds_ep.kind() {
            EndpointKind::Uds(path) => {
                assert!(path.to_string_lossy().contains("test.sock"));
            }
            _ => panic!("Expected UDS endpoint"),
        }

        let other_ep = Endpoint::from_uri("grpc://example.com");
        match other_ep.kind() {
            EndpointKind::Other(uri) => {
                assert_eq!(uri, "grpc://example.com");
            }
            _ => panic!("Expected Other endpoint"),
        }
    }

    #[test]
    fn test_module_instance_builder() {
        let instance_id = Uuid::new_v4();
        let instance = ModuleInstance::new("test_module", instance_id)
            .with_control(Endpoint::http("localhost", 8080))
            .with_version("1.2.3")
            .with_grpc_service("service1", Endpoint::http("localhost", 8082))
            .with_grpc_service("service2", Endpoint::http("localhost", 8083));

        assert_eq!(instance.module, "test_module");
        assert_eq!(instance.instance_id, instance_id);
        assert!(instance.control.is_some());
        assert_eq!(instance.version, Some("1.2.3".to_string()));
        assert_eq!(instance.grpc_services.len(), 2);
        assert!(instance.grpc_services.contains_key("service1"));
        assert!(instance.grpc_services.contains_key("service2"));
        assert!(matches!(instance.state(), InstanceState::Registered));
    }

    #[test]
    fn test_quarantine_and_evict() {
        let ttl = Duration::from_millis(50);
        let grace = Duration::from_millis(50);
        let dir = ModuleManager::new().with_heartbeat_policy(ttl, grace);

        let now = Instant::now();
        let instance = ModuleInstance::new("test_module", Uuid::new_v4());
        // Set the last heartbeat to be stale
        instance.inner.write().last_heartbeat = now - ttl - Duration::from_millis(10);

        dir.register_instance(Arc::new(instance));

        dir.evict_stale(now);
        let instances = dir.instances_of("test_module");
        assert_eq!(instances.len(), 1);
        assert!(matches!(instances[0].state(), InstanceState::Quarantined));

        let later = now + grace + Duration::from_millis(10);
        dir.evict_stale(later);

        let instances_after = dir.instances_of("test_module");
        assert!(instances_after.is_empty());
    }

    #[test]
    fn test_instances_of_empty() {
        let dir = ModuleManager::new();
        let instances = dir.instances_of("nonexistent");
        assert!(instances.is_empty());
    }

    #[test]
    fn test_rr_prefers_healthy() {
        let dir = ModuleManager::new();

        // Create two instances: one healthy, one quarantined
        let healthy_id = Uuid::new_v4();
        let healthy = Arc::new(ModuleInstance::new("test_module", healthy_id));
        dir.register_instance(healthy.clone());
        dir.update_heartbeat("test_module", healthy_id, Instant::now());

        let quarantined_id = Uuid::new_v4();
        let quarantined = Arc::new(ModuleInstance::new("test_module", quarantined_id));
        dir.register_instance(quarantined.clone());
        dir.mark_quarantined("test_module", quarantined_id);

        // RR should only pick the healthy instance
        for _ in 0..5 {
            let picked = dir.pick_instance_round_robin("test_module").unwrap();
            assert_eq!(picked.instance_id, healthy_id);
        }
    }

    #[test]
    fn test_pick_service_round_robin() {
        let dir = ModuleManager::new();

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        // Register two instances providing the same service
        let inst1 = Arc::new(
            ModuleInstance::new("test_module", id1)
                .with_grpc_service("test.Service", Endpoint::http("127.0.0.1", 8001)),
        );
        let inst2 = Arc::new(
            ModuleInstance::new("test_module", id2)
                .with_grpc_service("test.Service", Endpoint::http("127.0.0.1", 8002)),
        );

        dir.register_instance(inst1);
        dir.register_instance(inst2);

        // Mark both as healthy
        dir.update_heartbeat("test_module", id1, Instant::now());
        dir.update_heartbeat("test_module", id2, Instant::now());

        // Pick should rotate between instances
        let pick1 = dir.pick_service_round_robin("test.Service");
        let pick2 = dir.pick_service_round_robin("test.Service");
        let pick3 = dir.pick_service_round_robin("test.Service");

        assert!(pick1.is_some());
        assert!(pick2.is_some());
        assert!(pick3.is_some());

        let (_, inst1, ep1) = pick1.unwrap();
        let (_, inst2, ep2) = pick2.unwrap();
        let (_, inst3, _) = pick3.unwrap();

        // First and third should be the same (round-robin)
        assert_eq!(inst1.instance_id, inst3.instance_id);
        // First and second should be different
        assert_ne!(inst1.instance_id, inst2.instance_id);
        // Endpoints should differ
        assert_ne!(ep1, ep2);
    }
}
