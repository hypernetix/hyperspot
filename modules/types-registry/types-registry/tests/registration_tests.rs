#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Integration tests for entity registration flows

mod common;

use axum::http::StatusCode;
use common::create_service;
use serde_json::json;
use types_registry::api::rest::dto::RegisterEntitiesRequest;
use types_registry_sdk::ListQuery;

// =============================================================================
// Anonymous Entity Rejection Tests
// =============================================================================

#[tokio::test]
async fn test_anonymous_entity_rejected_immediately() {
    let service = create_service();

    // Entity without any ID field - should be rejected immediately
    let anonymous_entity = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" }
        }
    });

    let results = service.register(vec![anonymous_entity]);
    assert_eq!(results.len(), 1);
    assert!(results[0].is_err(), "Anonymous entity should be rejected");

    // Access the error from RegisterResult::Err variant
    if let types_registry_sdk::RegisterResult::Err { error, .. } = &results[0] {
        assert!(
            error.to_string().contains("GTS ID") || error.to_string().contains("id"),
            "Error should mention missing GTS ID: {error}"
        );
    } else {
        panic!("Expected RegisterResult::Err");
    }
}

#[tokio::test]
async fn test_batch_with_anonymous_entities_rejected_immediately() {
    let service = create_service();

    // Mix of valid and anonymous entities
    let valid_entity = json!({
        "$id": "gts://gts.acme.core.models.valid.v1~",
        "type": "object",
        "properties": {
            "name": { "type": "string" }
        }
    });

    let anonymous_entity1 = json!({
        "type": "object",
        "description": "No ID field"
    });

    let anonymous_entity2 = json!({
        "properties": {
            "value": { "type": "integer" }
        }
    });

    let results = service.register(vec![valid_entity, anonymous_entity1, anonymous_entity2]);
    assert_eq!(results.len(), 3);

    // Valid entity should succeed
    assert!(results[0].is_ok(), "Valid entity should be accepted");

    // Anonymous entities should fail immediately
    assert!(results[1].is_err(), "Anonymous entity 1 should be rejected");
    assert!(results[2].is_err(), "Anonymous entity 2 should be rejected");
}

#[tokio::test]
async fn test_anonymous_entity_rejected_in_ready_mode() {
    let service = create_service();

    // Register a valid entity first and switch to ready
    let valid_entity = json!({
        "$id": "gts://gts.acme.core.models.setup.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object"
    });
    let _ = service.register(vec![valid_entity]);
    service.switch_to_ready().unwrap();
    assert!(service.is_ready());

    // Try to register anonymous entity in ready mode
    let anonymous_entity = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" }
        }
    });

    let results = service.register(vec![anonymous_entity]);
    assert_eq!(results.len(), 1);
    assert!(
        results[0].is_err(),
        "Anonymous entity should be rejected in ready mode"
    );
}

// =============================================================================
// Full Registration Flow Tests
// =============================================================================

#[tokio::test]
async fn test_full_registration_flow_configuration_to_ready() {
    let service = create_service();

    // Phase 1: Configuration mode - register entities without validation
    assert!(!service.is_ready());

    let type_schema = json!({
        "$id": "gts://gts.acme.core.events.user_created.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "userId": { "type": "string" },
            "email": { "type": "string", "format": "email" },
            "createdAt": { "type": "string", "format": "date-time" }
        },
        "required": ["userId", "email", "createdAt"],
        "description": "Event emitted when a new user is created"
    });

    let results = service.register(vec![type_schema]);
    assert_eq!(results.len(), 1);
    assert!(results[0].is_ok());

    // Phase 2: Switch to ready mode
    let switch_result = service.switch_to_ready();
    assert!(switch_result.is_ok());
    assert!(service.is_ready());

    // Phase 3: Verify entity is accessible
    let entity = service
        .get("gts.acme.core.events.user_created.v1~")
        .unwrap();
    assert_eq!(entity.gts_id, "gts.acme.core.events.user_created.v1~");
    assert!(entity.is_type());
    assert_eq!(entity.vendor(), Some("acme"));
    assert_eq!(entity.package(), Some("core"));
    assert_eq!(entity.namespace(), Some("events"));
}

#[tokio::test]
async fn test_batch_registration_with_mixed_results() {
    let service = create_service();

    let entities = vec![
        json!({
            "$id": "gts://gts.acme.core.events.valid_type.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        }),
        json!({
            "$id": "invalid-gts-id",
            "type": "object"
        }),
        json!({
            "$id": "gts://gts.globex.core.events.another_valid.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        }),
        json!({
            "no_id_field": true
        }),
    ];

    let results = service.register(entities);
    assert_eq!(results.len(), 4);

    // First entity should succeed
    assert!(results[0].is_ok());

    // Second entity should fail (invalid GTS ID format)
    assert!(results[1].is_err());

    // Third entity should succeed
    assert!(results[2].is_ok());

    // Fourth entity should fail (no GTS ID field)
    assert!(results[3].is_err());
}

#[tokio::test]
async fn test_duplicate_registration_fails() {
    let service = create_service();

    let entity = json!({
        "$id": "gts://gts.acme.core.events.user_created.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object"
    });

    let entity_modified = json!({
        "$id": "gts://gts.acme.core.events.user_created.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "description": "modified"
    });

    // First registration should succeed
    let results1 = service.register(vec![entity]);
    assert!(results1[0].is_ok());

    // Second registration with different content should fail
    let results2 = service.register(vec![entity_modified]);
    assert!(results2[0].is_err());
}

#[tokio::test]
async fn test_registration_in_ready_mode() {
    let service = create_service();

    // Switch to ready first
    service.switch_to_ready().unwrap();
    assert!(service.is_ready());

    // Register in ready mode (with validation)
    let entity = json!({
        "$id": "gts://gts.acme.core.events.ready_type.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "data": { "type": "string" }
        }
    });

    let results = service.register(vec![entity]);
    assert!(results[0].is_ok());

    // Entity should be immediately accessible
    let retrieved = service.get("gts.acme.core.events.ready_type.v1~");
    assert!(retrieved.is_ok());
}

#[tokio::test]
async fn test_empty_batch_registration() {
    let service = create_service();

    let results = service.register(vec![]);
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_large_batch_registration() {
    let service = create_service();

    // Register 100 entities in a single batch
    let entities: Vec<_> = (0..100)
        .map(|i| {
            json!({
                "$id": format!("gts://gts.acme.core.events.batch_{i}.v1~"),
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object"
            })
        })
        .collect();

    let results = service.register(entities);
    assert_eq!(results.len(), 100);
    assert!(results.iter().all(types_registry::RegisterResult::is_ok));

    service.switch_to_ready().unwrap();

    let all = service.list(&ListQuery::default()).unwrap();
    assert_eq!(all.len(), 100);
}

// =============================================================================
// REST Handler Registration Tests
// =============================================================================

#[tokio::test]
async fn test_rest_register_handler_integration() {
    use axum::extract::{Extension, Json};
    use types_registry::api::rest::handlers::register_entities;

    let service = create_service();
    // Switch to ready first so REST API works
    service.switch_to_ready().unwrap();

    let request = RegisterEntitiesRequest {
        entities: vec![json!({
            "$id": "gts://gts.acme.core.events.rest_test.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "description": "Test type for REST handler"
        })],
    };

    let result = register_entities(Extension(service), Json(request)).await;
    assert!(result.is_ok());

    let (status, Json(response)) = result.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response.summary.total, 1);
    assert_eq!(response.summary.succeeded, 1);
    assert_eq!(response.summary.failed, 0);
}

#[tokio::test]
async fn test_rest_register_empty_request() {
    use axum::extract::{Extension, Json};
    use types_registry::api::rest::handlers::register_entities;

    let service = create_service();
    // Switch to ready first so REST API works
    service.switch_to_ready().unwrap();

    let request = RegisterEntitiesRequest { entities: vec![] };

    let result = register_entities(Extension(service), Json(request)).await;
    assert!(result.is_ok());

    let (status, Json(response)) = result.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response.summary.total, 0);
    assert_eq!(response.summary.succeeded, 0);
    assert_eq!(response.summary.failed, 0);
}

#[tokio::test]
async fn test_rest_register_with_invalid_entities() {
    use axum::extract::{Extension, Json};
    use types_registry::api::rest::handlers::register_entities;

    let service = create_service();
    // Switch to ready first so REST API works
    service.switch_to_ready().unwrap();

    let request = RegisterEntitiesRequest {
        entities: vec![
            json!({ "$id": "invalid-id", "type": "object" }),
            json!({ "no_id": true }),
        ],
    };

    let result = register_entities(Extension(service), Json(request)).await;
    assert!(result.is_ok());

    let (status, Json(response)) = result.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response.summary.total, 2);
    assert_eq!(response.summary.succeeded, 0);
    assert_eq!(response.summary.failed, 2);
}
