#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Integration tests for list and query operations

mod common;

use axum::extract::Json;
use common::create_service;
use serde_json::json;
use types_registry::api::rest::dto::ListEntitiesQuery;
use types_registry_sdk::ListQuery;

// =============================================================================
// List and Query Tests
// =============================================================================

#[tokio::test]
async fn test_list_entities_with_vendor_filter() {
    let service = create_service();

    // Register entities from different vendors
    let entities = vec![
        json!({ "$id": "gts.acme.core.events.type1.v1~", "type": "object" }),
        json!({ "$id": "gts.acme.core.events.type2.v1~", "type": "object" }),
        json!({ "$id": "gts.globex.core.events.type3.v1~", "type": "object" }),
        json!({ "$id": "gts.initech.core.events.type4.v1~", "type": "object" }),
    ];

    let _ = service.register(entities);
    service.switch_to_ready().unwrap();

    // Filter by vendor "acme"
    let query = ListQuery::default().with_vendor("acme");
    let results = service.list(&query).unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|e| e.vendor() == Some("acme")));

    // Filter by vendor "globex"
    let query = ListQuery::default().with_vendor("globex");
    let results = service.list(&query).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].vendor(), Some("globex"));
}

#[tokio::test]
async fn test_list_entities_with_package_filter() {
    let service = create_service();

    let entities = vec![
        json!({ "$id": "gts.acme.core.events.type1.v1~", "type": "object" }),
        json!({ "$id": "gts.acme.billing.events.type2.v1~", "type": "object" }),
        json!({ "$id": "gts.acme.core.events.type3.v1~", "type": "object" }),
    ];

    let _ = service.register(entities);
    service.switch_to_ready().unwrap();

    let query = ListQuery::default().with_package("core");
    let results = service.list(&query).unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|e| e.package() == Some("core")));

    let query = ListQuery::default().with_package("billing");
    let results = service.list(&query).unwrap();
    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn test_list_entities_with_namespace_filter() {
    let service = create_service();

    let entities = vec![
        json!({ "$id": "gts.acme.core.events.type1.v1~", "type": "object" }),
        json!({ "$id": "gts.acme.core.commands.type2.v1~", "type": "object" }),
        json!({ "$id": "gts.acme.core.events.type3.v1~", "type": "object" }),
    ];

    let _ = service.register(entities);
    service.switch_to_ready().unwrap();

    let query = ListQuery::default().with_namespace("events");
    let results = service.list(&query).unwrap();
    assert_eq!(results.len(), 2);

    let query = ListQuery::default().with_namespace("commands");
    let results = service.list(&query).unwrap();
    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn test_list_entities_with_combined_filters() {
    let service = create_service();

    let entities = vec![
        json!({ "$id": "gts.acme.core.events.type1.v1~", "type": "object" }),
        json!({ "$id": "gts.acme.billing.events.type2.v1~", "type": "object" }),
        json!({ "$id": "gts.globex.core.events.type3.v1~", "type": "object" }),
    ];

    let _ = service.register(entities);
    service.switch_to_ready().unwrap();

    // Combined filter: vendor=acme AND package=core
    let query = ListQuery::default()
        .with_vendor("acme")
        .with_package("core");
    let results = service.list(&query).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].gts_id, "gts.acme.core.events.type1.v1~");
}

#[tokio::test]
async fn test_list_with_pattern_filter() {
    let service = create_service();

    let entities = vec![
        json!({ "$id": "gts.acme.core.events.user_created.v1~", "type": "object" }),
        json!({ "$id": "gts.acme.core.events.user_updated.v1~", "type": "object" }),
        json!({ "$id": "gts.acme.core.events.order_created.v1~", "type": "object" }),
        json!({ "$id": "gts.acme.core.commands.create_user.v1~", "type": "object" }),
    ];

    let _ = service.register(entities);
    service.switch_to_ready().unwrap();

    // Pattern matching for "user" in the name
    let query = ListQuery::default().with_pattern("user");
    let results = service.list(&query).unwrap();
    // Should match user_created, user_updated, create_user
    assert!(
        results.len() >= 2,
        "Pattern 'user' should match multiple entities"
    );
}

#[tokio::test]
async fn test_list_with_is_type_filter() {
    let service = create_service();

    // Register a type
    let type_schema = json!({
        "$id": "gts.acme.core.models.filter_test.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": { "name": { "type": "string" } }
    });

    let _ = service.register(vec![type_schema]);
    service.switch_to_ready().unwrap();

    // Register instances
    let instances = vec![
        json!({
            "id": "gts.acme.core.models.filter_test.v1~acme.core.instances.i1.v1",
            "name": "instance1"
        }),
        json!({
            "id": "gts.acme.core.models.filter_test.v1~acme.core.instances.i2.v1",
            "name": "instance2"
        }),
    ];

    let _ = service.register(instances);

    // Filter for types only
    let types = service
        .list(&ListQuery::default().with_is_type(true))
        .unwrap();
    assert_eq!(types.len(), 1);
    assert!(types[0].is_type());

    // Filter for instances only
    let instances = service
        .list(&ListQuery::default().with_is_type(false))
        .unwrap();
    assert_eq!(instances.len(), 2);
    assert!(instances.iter().all(types_registry::GtsEntity::is_instance));

    // No filter - get all
    let all = service.list(&ListQuery::default()).unwrap();
    assert_eq!(all.len(), 3);
}

#[tokio::test]
async fn test_multiple_vendors_isolation() {
    let service = create_service();

    // Register entities from multiple vendors
    let entities = vec![
        json!({ "$id": "gts.vendor_a.pkg.ns.type1.v1~", "type": "object" }),
        json!({ "$id": "gts.vendor_a.pkg.ns.type2.v1~", "type": "object" }),
        json!({ "$id": "gts.vendor_b.pkg.ns.type1.v1~", "type": "object" }),
        json!({ "$id": "gts.vendor_c.pkg.ns.type1.v1~", "type": "object" }),
    ];

    let _ = service.register(entities);
    service.switch_to_ready().unwrap();

    // Each vendor filter should return correct count
    assert_eq!(
        service
            .list(&ListQuery::default().with_vendor("vendor_a"))
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        service
            .list(&ListQuery::default().with_vendor("vendor_b"))
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        service
            .list(&ListQuery::default().with_vendor("vendor_c"))
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        service
            .list(&ListQuery::default().with_vendor("vendor_d"))
            .unwrap()
            .len(),
        0
    );
}

#[tokio::test]
async fn test_combined_vendor_package_namespace_filter() {
    let service = create_service();

    let entities = vec![
        json!({ "$id": "gts.acme.billing.invoices.invoice.v1~", "type": "object" }),
        json!({ "$id": "gts.acme.billing.payments.payment.v1~", "type": "object" }),
        json!({ "$id": "gts.acme.core.events.event.v1~", "type": "object" }),
        json!({ "$id": "gts.globex.billing.invoices.invoice.v1~", "type": "object" }),
    ];

    let _ = service.register(entities);
    service.switch_to_ready().unwrap();

    // Triple filter: vendor + package + namespace
    let query = ListQuery::default()
        .with_vendor("acme")
        .with_package("billing")
        .with_namespace("invoices");
    let results = service.list(&query).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].gts_id, "gts.acme.billing.invoices.invoice.v1~");
}

// =============================================================================
// REST Handler List Tests
// =============================================================================

#[tokio::test]
async fn test_rest_list_handler_integration() {
    use axum::extract::{Extension, Query};
    use types_registry::api::rest::handlers::list_entities;

    let service = create_service();

    // Register entities via internal API (before ready)
    let _ = service.register(vec![
        json!({ "$id": "gts.acme.core.events.list_test1.v1~", "type": "object" }),
        json!({ "$id": "gts.acme.core.events.list_test2.v1~", "type": "object" }),
    ]);
    service.switch_to_ready().unwrap();

    // Test list handler (now service is ready)
    let query = ListEntitiesQuery {
        vendor: Some("acme".to_owned()),
        ..Default::default()
    };

    let result = list_entities(Extension(service), Query(query)).await;
    assert!(result.is_ok());

    let Json(response) = result.unwrap();
    assert_eq!(response.count, 2);
}

#[tokio::test]
async fn test_rest_list_empty_results() {
    use axum::extract::{Extension, Query};
    use types_registry::api::rest::handlers::list_entities;

    let service = create_service();
    service.switch_to_ready().unwrap();

    // Query with filter that matches nothing
    let query = ListEntitiesQuery {
        vendor: Some("nonexistent_vendor".to_owned()),
        ..Default::default()
    };

    let result = list_entities(Extension(service), Query(query)).await;
    assert!(result.is_ok());

    let Json(response) = result.unwrap();
    assert_eq!(response.count, 0);
    assert!(response.entities.is_empty());
}

// =============================================================================
// REST Handler Get Tests
// =============================================================================

#[tokio::test]
async fn test_rest_get_handler_integration() {
    use axum::extract::{Extension, Path};
    use types_registry::api::rest::handlers::get_entity;

    let service = create_service();

    // Register entity via internal API (before ready)
    let _ = service.register(vec![json!({
        "$id": "gts.acme.core.events.get_test.v1~",
        "type": "object",
        "description": "Test entity for GET handler"
    })]);
    service.switch_to_ready().unwrap();

    // Test get handler (now service is ready)
    let result = get_entity(
        Extension(service),
        Path("gts.acme.core.events.get_test.v1~".to_owned()),
    )
    .await;
    assert!(result.is_ok());

    let Json(entity) = result.unwrap();
    assert_eq!(entity.gts_id, "gts.acme.core.events.get_test.v1~");
    assert_eq!(
        entity.description,
        Some("Test entity for GET handler".to_owned())
    );
}

#[tokio::test]
async fn test_rest_get_handler_not_found() {
    use axum::extract::{Extension, Path};
    use types_registry::api::rest::handlers::get_entity;

    let service = create_service();
    service.switch_to_ready().unwrap();

    let result = get_entity(
        Extension(service),
        Path("gts.nonexistent.pkg.ns.type.v1~".to_owned()),
    )
    .await;

    assert!(result.is_err());
}
