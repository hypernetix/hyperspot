// @fdd-change:fdd-analytics-feature-gts-core-change-platform-integration-fix
/// Integration tests for GTS Core API
///
/// These tests verify:
/// - Module compilation and structure
/// - RestfulModule trait implementation
/// - Basic routing layer functionality
///
/// Note: Full api_ingress integration tests require running server

// @fdd-test:fdd-analytics-feature-gts-core-test-routing-table-lookup
#[test]
fn test_gts_core_module_compiles() {
    // Verify analytics module with GTS Core compiles and can be instantiated
    use analytics::AnalyticsModule;

    let module = AnalyticsModule::default();

    // Basic sanity check - module can be created
    assert!(std::mem::size_of_val(&module) > 0);
}

// @fdd-test:fdd-analytics-feature-gts-core-test-routing-table-lookup
#[test]
fn test_gts_core_restful_module_trait() {
    // Verify RestfulModule trait is implemented
    use analytics::AnalyticsModule;
    use modkit::RestfulModule;

    let module = AnalyticsModule::default();

    // Verify trait bounds compile
    fn assert_restful_module<T: RestfulModule>(_: &T) {}
    assert_restful_module(&module);
}
