#[cfg(test)]
mod module_tests {
    use axum::Router;
    use std::sync::{Arc, Mutex};
    use tokio_util::sync::CancellationToken;

    use crate::{
        context::{ConfigProvider, ModuleCtx},
        contracts::{Module, OpenApiRegistry, RestHostModule, RestfulModule},
        registry::{ModuleRegistry, RegistryBuilder, RegistryError},
    };

    // Helper to create a test ModuleCtx
    struct EmptyConfigProvider;
    impl ConfigProvider for EmptyConfigProvider {
        fn get_module_config(&self, _module_name: &str) -> Option<&serde_json::Value> {
            None
        }
    }

    fn test_module_ctx(cancel: CancellationToken) -> ModuleCtx {
        ModuleCtx::new(
            "test",
            Arc::new(EmptyConfigProvider),
            Arc::new(crate::client_hub::ClientHub::default()),
            cancel,
            None,
        )
    }

    // Minimal OpenAPI mock for REST phase
    struct MockOpenApi;
    impl OpenApiRegistry for MockOpenApi {
        fn register_operation(&self, _op: &crate::api::OperationSpec) {}
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

    // Test module implementations
    #[derive(Default)]
    struct TestModule {
        name: String,
    }

    impl TestModule {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    #[async_trait::async_trait]
    impl Module for TestModule {
        async fn init(&self, _ctx: &crate::context::ModuleCtx) -> anyhow::Result<()> {
            Ok(())
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    impl RestfulModule for TestModule {
        fn register_rest(
            &self,
            _ctx: &crate::context::ModuleCtx,
            router: Router,
            _openapi: &dyn OpenApiRegistry,
        ) -> anyhow::Result<Router> {
            // Just add a test route to verify it was called
            use axum::routing::get;
            let route_path = format!("/{}", self.name);
            Ok(router.route(&route_path, get(|| async { "test" })))
        }
    }

    // Track calls to rest_prepare and rest_finalize
    type CallTracker = Arc<Mutex<Vec<String>>>;

    #[derive(Clone)]
    struct TestRestHost {
        #[allow(dead_code)]
        name: String,
        calls: CallTracker,
    }

    impl TestRestHost {
        fn new(name: &str, calls: CallTracker) -> Self {
            Self {
                name: name.to_string(),
                calls,
            }
        }
    }

    #[async_trait::async_trait]
    impl Module for TestRestHost {
        async fn init(&self, _ctx: &crate::context::ModuleCtx) -> anyhow::Result<()> {
            Ok(())
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    impl RestHostModule for TestRestHost {
        fn as_registry(&self) -> &dyn crate::contracts::OpenApiRegistry {
            static REG: MockOpenApi = MockOpenApi;
            &REG
        }
        fn rest_prepare(
            &self,
            _ctx: &crate::context::ModuleCtx,
            router: Router,
        ) -> anyhow::Result<Router> {
            self.calls.lock().unwrap().push("prepare".to_string());
            use axum::routing::get;
            Ok(router.route("/healthz", get(|| async { "ok" })))
        }

        fn rest_finalize(
            &self,
            _ctx: &crate::context::ModuleCtx,
            router: Router,
        ) -> anyhow::Result<Router> {
            self.calls.lock().unwrap().push("finalize".to_string());
            use axum::routing::get;
            Ok(router.route("/openapi.json", get(|| async { "{}" })))
        }
    }

    #[test]
    fn test_module_registry_builds() {
        let registry = ModuleRegistry::discover_and_build();
        assert!(registry.is_ok(), "Registry should build successfully");
    }

    #[tokio::test]
    async fn test_empty_lifecycle_phases() {
        // Build empty registry (no inventory modules in this unit test)
        let registry = ModuleRegistry::discover_and_build().expect("registry builds");

        // Build scoped context
        let cancel = CancellationToken::new();
        let ctx = test_module_ctx(cancel.clone());

        // init
        registry
            .run_init_phase(&ctx)
            .await
            .expect("init should succeed on empty registry");

        // (optional) REST phase: should be a no-op and return the same router
        let router = Router::new();
        let _router = registry
            .run_rest_phase(&ctx, router)
            .expect("rest registration should succeed on empty registry");

        // start / stop
        registry
            .run_start_phase(cancel.clone())
            .await
            .expect("start should succeed");
        registry
            .run_stop_phase(cancel)
            .await
            .expect("stop should succeed");
    }

    #[test]
    fn test_rest_host_no_host_with_rest_modules_fails() {
        let mut builder = RegistryBuilder::default();

        // Register a REST module without a host
        let rest_module = Arc::new(TestModule::new("test_rest"));
        builder.register_core_with_meta("test_rest", &[], rest_module.clone() as Arc<dyn Module>);
        builder.register_rest_with_meta("test_rest", rest_module.clone() as Arc<dyn RestfulModule>);

        let registry = builder.build_topo_sorted().expect("registry should build");

        let cancel = CancellationToken::new();
        let ctx = test_module_ctx(cancel);

        let router = Router::new();
        // Should fail with specific error type
        let result = registry.run_rest_phase(&ctx, router);
        assert!(matches!(result, Err(RegistryError::RestRequiresHost)));
    }

    #[test]
    fn test_rest_host_with_one_host_and_rest_modules_succeeds() {
        let mut builder = RegistryBuilder::default();
        let call_tracker = Arc::new(Mutex::new(Vec::new()));

        // Register a REST host
        let host_module = Arc::new(TestRestHost::new("test_host", call_tracker.clone()));
        builder.register_core_with_meta("test_host", &[], host_module.clone() as Arc<dyn Module>);
        builder.register_rest_host_with_meta(
            "test_host",
            host_module.clone() as Arc<dyn RestHostModule>,
        );

        // Register a REST module
        let rest_module = Arc::new(TestModule::new("test_rest"));
        builder.register_core_with_meta("test_rest", &[], rest_module.clone() as Arc<dyn Module>);
        builder.register_rest_with_meta("test_rest", rest_module.clone() as Arc<dyn RestfulModule>);

        let registry = builder.build_topo_sorted().expect("registry should build");

        let cancel = CancellationToken::new();
        let ctx = test_module_ctx(cancel);

        let router = Router::new();
        // Should succeed
        let result = registry.run_rest_phase(&ctx, router);
        assert!(result.is_ok());

        // Verify the correct call sequence: prepare -> finalize
        let calls = call_tracker.lock().unwrap();
        assert_eq!(*calls, vec!["prepare", "finalize"]);
    }

    #[test]
    fn test_multiple_rest_hosts_fails_at_registration() {
        let mut builder = RegistryBuilder::default();
        let call_tracker1 = Arc::new(Mutex::new(Vec::new()));
        let call_tracker2 = Arc::new(Mutex::new(Vec::new()));

        // Register first REST host
        let host1 = Arc::new(TestRestHost::new("host1", call_tracker1));
        builder.register_core_with_meta("host1", &[], host1.clone() as Arc<dyn Module>);
        builder.register_rest_host_with_meta("host1", host1.clone() as Arc<dyn RestHostModule>);

        // Register second REST host - should panic
        let host2 = Arc::new(TestRestHost::new("host2", call_tracker2));
        builder.register_core_with_meta("host2", &[], host2.clone() as Arc<dyn Module>);

        // Registering a second host should be reported as a configuration error at build time
        builder.register_rest_host_with_meta("host2", host2.clone() as Arc<dyn RestHostModule>);
        let err = builder.build_topo_sorted().unwrap_err();
        match err {
            RegistryError::InvalidRegistryConfiguration { errors } => {
                assert!(
                    errors
                        .iter()
                        .any(|e| e.contains("Multiple REST host modules detected")),
                    "expected multiple host configuration error, got {errors:?}"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn test_rest_host_without_core_module_fails_validation() {
        let mut builder = RegistryBuilder::default();
        let call_tracker = Arc::new(Mutex::new(Vec::new()));

        // Register a REST host capability WITHOUT registering the core module
        let host_module = Arc::new(TestRestHost::new("test_host", call_tracker));
        // Skip: builder.register_core_with_meta("test_host", &[], host_module.clone() as Arc<dyn Module>);
        builder.register_rest_host_with_meta(
            "test_host",
            host_module.clone() as Arc<dyn RestHostModule>,
        );

        // Should fail validation during build
        let result = builder.build_topo_sorted();
        match result {
            Ok(_) => panic!("Expected build to fail, but it succeeded"),
            Err(e) => match e {
                RegistryError::UnknownModule(name) => assert_eq!(name, "test_host"),
                other => panic!("unexpected error: {other:?}"),
            },
        }
    }

    #[test]
    fn test_rest_host_no_host_no_rest_modules_succeeds() {
        let mut builder = RegistryBuilder::default();

        // Register a module that's neither REST nor REST host (e.g., just core)
        let core_module = Arc::new(TestModule::new("core_only"));
        builder.register_core_with_meta("core_only", &[], core_module.clone() as Arc<dyn Module>);

        let registry = builder.build_topo_sorted().expect("registry should build");

        let cancel = CancellationToken::new();
        let ctx = test_module_ctx(cancel);

        let router = Router::new();
        // Should succeed and return router unchanged
        let result = registry.run_rest_phase(&ctx, router);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rest_host_prepare_finalize_call_order() {
        let mut builder = RegistryBuilder::default();
        let call_tracker = Arc::new(Mutex::new(Vec::new()));

        // Register a REST host
        let host_module = Arc::new(TestRestHost::new("test_host", call_tracker.clone()));
        builder.register_core_with_meta("test_host", &[], host_module.clone() as Arc<dyn Module>);
        builder.register_rest_host_with_meta(
            "test_host",
            host_module.clone() as Arc<dyn RestHostModule>,
        );

        // Register multiple REST modules to ensure they're all processed between prepare and finalize
        let names = ["rest_1", "rest_2", "rest_3"];
        for &name in &names {
            let rest_module = Arc::new(TestModule::new(name));
            builder.register_core_with_meta(name, &[], rest_module.clone() as Arc<dyn Module>);
            builder.register_rest_with_meta(name, rest_module.clone() as Arc<dyn RestfulModule>);
        }

        let registry = builder.build_topo_sorted().expect("registry should build");

        let cancel = CancellationToken::new();
        let ctx = test_module_ctx(cancel);

        let router = Router::new();
        // Run REST phase
        let result = registry.run_rest_phase(&ctx, router);
        assert!(result.is_ok());

        // Verify the call order: prepare first, then finalize last
        let calls = call_tracker.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0], "prepare");
        assert_eq!(calls[1], "finalize");
    }
}
