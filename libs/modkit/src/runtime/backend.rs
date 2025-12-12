//! Backend configuration types for out-of-process module management

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

/// The kind of backend used to spawn and manage module instances
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    LocalProcess,
    K8s,
    Static,
    Mock,
}

/// Configuration for an out-of-process module
pub struct OopModuleConfig {
    pub name: crate::runtime::ModuleName,
    pub binary: Option<PathBuf>,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub backend: BackendKind,
    pub version: Option<String>,
}

impl OopModuleConfig {
    pub fn new(name: crate::runtime::ModuleName, backend: BackendKind) -> Self {
        Self {
            name,
            binary: None,
            args: Vec::new(),
            env: HashMap::new(),
            backend,
            version: None,
        }
    }
}

/// A handle to a running module instance
#[derive(Clone)]
pub struct InstanceHandle {
    pub module: crate::runtime::ModuleName,
    pub instance_id: String,
    pub backend: BackendKind,
    pub pid: Option<u32>,
    pub created_at: Instant,
}

impl std::fmt::Debug for InstanceHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstanceHandle")
            .field("module", &self.module)
            .field("instance_id", &self.instance_id)
            .field("backend", &self.backend)
            .field("pid", &self.pid)
            .field("created_at", &self.created_at)
            .finish()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_oop_module_config_builder() {
        let mut cfg = OopModuleConfig::new("my_module", BackendKind::LocalProcess);
        cfg.binary = Some(PathBuf::from("/usr/bin/myapp"));
        cfg.args = vec!["--port".to_string(), "8080".to_string()];
        cfg.env.insert("LOG_LEVEL".to_string(), "debug".to_string());
        cfg.version = Some("1.0.0".to_string());

        assert_eq!(cfg.name, "my_module");
        assert_eq!(cfg.backend, BackendKind::LocalProcess);
        assert_eq!(cfg.binary, Some(PathBuf::from("/usr/bin/myapp")));
        assert_eq!(cfg.args.len(), 2);
        assert_eq!(cfg.env.len(), 1);
        assert_eq!(cfg.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_backend_kind_equality() {
        assert_eq!(BackendKind::LocalProcess, BackendKind::LocalProcess);
        assert_ne!(BackendKind::LocalProcess, BackendKind::K8s);
        assert_ne!(BackendKind::K8s, BackendKind::Static);
        assert_ne!(BackendKind::Static, BackendKind::Mock);
    }

    #[test]
    fn test_instance_handle_debug() {
        let handle = InstanceHandle {
            module: "test_module",
            instance_id: "test-123".to_string(),
            backend: BackendKind::LocalProcess,
            pid: Some(12345),
            created_at: Instant::now(),
        };

        let debug_str = format!("{:?}", handle);
        assert!(debug_str.contains("test_module"));
        assert!(debug_str.contains("test-123"));
        assert!(debug_str.contains("LocalProcess"));
        assert!(debug_str.contains("12345"));
    }
}
