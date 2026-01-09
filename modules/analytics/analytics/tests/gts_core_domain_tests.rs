// @fdd-change:fdd-analytics-feature-gts-core-change-quality-assurance
use analytics::api::rest::gts_core::GtsCoreError;
use analytics::domain::gts_core::{GtsCoreRouter, GtsTypeIdentifier, RoutingTable};

// @fdd-test:fdd-analytics-feature-gts-core-test-gts-identifier-parsing
#[test]
fn parses_gts_type_identifier_from_instance_id() {
    let id = "gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.sales.v1";
    let type_id = GtsTypeIdentifier::parse(id).expect("parse");
    assert_eq!(type_id.as_str(), "gts.hypernetix.hyperspot.ax.query.v1~");
}

// @fdd-test:fdd-analytics-feature-gts-core-test-routing-table-lookup
#[test]
fn routing_table_register_and_lookup() {
    let mut table = RoutingTable::new();
    table
        .register(
            "gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.test.v1",
            "query-handler",
        )
        .expect("register");

    let handler = table
        .lookup("gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.instance_123.v1")
        .expect("lookup");
    assert_eq!(handler.map(|h| h.as_str()), Some("query-handler"));
}

// @fdd-test:fdd-analytics-feature-gts-core-test-routing-table-lookup
#[test]
fn router_routes_or_returns_none() {
    let mut table = RoutingTable::new();
    table
        .register(
            "gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.test.v1",
            "query-handler",
        )
        .expect("register");

    let router = GtsCoreRouter::new(table);

    let hit = router
        .route("gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.instance_123.v1")
        .expect("route");
    assert_eq!(hit, Some("query-handler"));

    let miss = router
        .route("gts.hypernetix.hyperspot.ax.unknown.v1~acme.analytics._.instance_123.v1")
        .expect("route");
    assert_eq!(miss, None);
}

// @fdd-test:fdd-analytics-feature-gts-core-test-tolerant-reader-pattern
#[test]
fn gts_core_error_maps_to_expected_problem_statuses() {
    let e404 = GtsCoreError::UnknownGtsType {
        gts_type: "gts.unknown.v1~".to_string(),
        instance: "/api/analytics/v1/gts/test".to_string(),
    };
    assert_eq!(e404.to_problem_details().status, 404);

    let e400 = GtsCoreError::InvalidIdentifier {
        detail: "bad id".to_string(),
        instance: "/api/analytics/v1/gts/test".to_string(),
    };
    assert_eq!(e400.to_problem_details().status, 400);

    let e503 = GtsCoreError::DomainFeatureUnavailable {
        gts_type: "gts.any.v1~".to_string(),
        instance: "/api/analytics/v1/gts/test".to_string(),
    };
    assert_eq!(e503.to_problem_details().status, 503);
}
