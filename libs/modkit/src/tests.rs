#[cfg(test)]
mod module_tests {
    use crate::registry::ModuleRegistry;

    #[test]
    fn test_module_registry_builds() {
        let registry = ModuleRegistry::discover_and_build();
        assert!(registry.is_ok(), "Registry should build successfully");
    }

    // Note: Tests for REST phase, lifecycle phases (init/start/stop), and other
    // runtime orchestration have been moved to integration tests or are tested
    // through HostRuntime. ModuleRegistry is now purely a metadata + topo sort struct.
}
