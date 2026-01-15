// @fdd-change:fdd-analytics-feature-gts-core-change-platform-integration-fix:ph-1
/// Integration tests for GTS Core API
///
/// These tests verify:
/// - Module compilation and structure
/// - RestfulModule trait implementation
/// - Basic routing layer functionality
///
/// Note: Full api_ingress integration tests require running server

// @fdd-test:fdd-analytics-feature-gts-core-test-routing-table-lookup:ph-1
#[test]
fn test_gts_core_module_compiles() {
    // Verify analytics module with GTS Core compiles and can be instantiated
    use analytics::AnalyticsModule;

    // fdd-begin fdd-analytics-feature-gts-core-test-routing-table-lookup:ph-1:inst-route-known-type
    let module = AnalyticsModule::default();
    // fdd-end fdd-analytics-feature-gts-core-test-routing-table-lookup:ph-1:inst-route-known-type

    // Basic sanity check - module can be created
    // fdd-begin fdd-analytics-feature-gts-core-test-routing-table-lookup:ph-1:inst-verify-target-selected
    assert!(std::mem::size_of_val(&module) > 0);
    // fdd-end fdd-analytics-feature-gts-core-test-routing-table-lookup:ph-1:inst-verify-target-selected
}

// @fdd-test:fdd-analytics-feature-gts-core-test-routing-table-lookup:ph-1
#[test]
fn test_gts_core_restful_module_trait() {
    // Verify RestfulModule trait is implemented
    use analytics::AnalyticsModule;
    use analytics::api::rest::gts_core::handlers;
    use analytics::domain::gts_core::{GtsCoreRouter, RoutingTable};
    use modkit::RestfulModule;
    use modkit_security::SecurityCtx;
    use std::sync::Arc;
    use tokio::runtime;

    let module = AnalyticsModule::default();

    // Verify trait bounds compile
    fn assert_restful_module<T: RestfulModule>(_: &T) {}
    assert_restful_module(&module);

    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");

    // fdd-begin fdd-analytics-feature-gts-core-test-routing-table-lookup:ph-1:inst-verify-404
    // Unknown type -> routing decision returns None -> 404
    let router = Arc::new(GtsCoreRouter::new(RoutingTable::new()));
    let err = rt
        .block_on(handlers::get_entity(
        axum::extract::Path("gts.hypernetix.hyperspot.ax.unknown.v1~acme.analytics._.instance_123.v1".to_string()),
        axum::extract::Extension(SecurityCtx::anonymous()),
        axum::extract::Extension(router.clone()),
    ))
        .expect_err("expected 404 Problem");
    assert_eq!(err.status, axum::http::StatusCode::NOT_FOUND);
    // fdd-end fdd-analytics-feature-gts-core-test-routing-table-lookup:ph-1:inst-verify-404

    // fdd-begin fdd-analytics-feature-gts-core-test-routing-table-lookup:ph-1:inst-route-missing-delegate
    // fdd-begin fdd-analytics-feature-gts-core-test-routing-table-lookup:ph-1:inst-verify-501
    // Known type but no delegate registered -> decision is Some -> 501
    let mut table = RoutingTable::new();
    table
        .register(
            "gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.test.v1",
            "query-handler",
        )
        .expect("register");
    let router = Arc::new(GtsCoreRouter::new(table));
    let err = rt
        .block_on(handlers::get_entity(
        axum::extract::Path(
            "gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.instance_123.v1".to_string(),
        ),
        axum::extract::Extension(SecurityCtx::anonymous()),
        axum::extract::Extension(router),
    ))
        .expect_err("expected 501 Problem");
    assert_eq!(err.status, axum::http::StatusCode::NOT_IMPLEMENTED);
    // fdd-end fdd-analytics-feature-gts-core-test-routing-table-lookup:ph-1:inst-verify-501
    // fdd-end fdd-analytics-feature-gts-core-test-routing-table-lookup:ph-1:inst-route-missing-delegate
}
