#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Integration tests for edge cases, error handling, and entity content verification

mod common;

use common::create_service;
use serde_json::json;

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
async fn test_get_nonexistent_entity() {
    let service = create_service();
    service.switch_to_ready().unwrap();

    let result = service.get("gts.nonexistent.pkg.ns.type.v1~");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_invalid_gts_id_formats() {
    let service = create_service();

    let invalid_entities = vec![
        json!({ "$id": "not-a-gts-id", "type": "object" }),
        json!({ "$id": "gts", "type": "object" }),
        json!({ "$id": "gts.vendor", "type": "object" }),
        json!({ "$id": "", "type": "object" }),
    ];

    let results = service.register(invalid_entities);

    // All should fail due to invalid GTS ID format
    for result in results {
        assert!(result.is_err());
    }
}

// =============================================================================
// GTS ID Extraction Tests
// =============================================================================

#[tokio::test]
async fn test_gts_id_extraction_priority() {
    let service = create_service();

    // Test that $id field is used when present (highest priority)
    // The repository extracts GTS ID from configured fields in order
    let entity = json!({
        "$id": "gts.acme.core.events.from_dollar_id.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object"
    });

    let results = service.register(vec![entity]);
    assert!(results[0].is_ok());

    service.switch_to_ready().unwrap();

    // Verify the entity was registered with the $id value
    let retrieved = service.get("gts.acme.core.events.from_dollar_id.v1~");
    assert!(retrieved.is_ok());
    assert_eq!(
        retrieved.unwrap().gts_id,
        "gts.acme.core.events.from_dollar_id.v1~"
    );
}

// =============================================================================
// Entity Content Verification Tests
// =============================================================================

#[tokio::test]
async fn test_entity_content_preserved() {
    let service = create_service();

    let original_content = json!({
        "$id": "gts.acme.core.events.content_test.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "customField": { "type": "string" },
            "nestedObject": {
                "type": "object",
                "properties": {
                    "innerField": { "type": "number" }
                }
            }
        },
        "customMetadata": {
            "author": "test",
            "version": "1.0.0"
        },
        "description": "Test entity with custom content"
    });

    let _ = service.register(vec![original_content]);
    service.switch_to_ready().unwrap();

    let retrieved = service
        .get("gts.acme.core.events.content_test.v1~")
        .unwrap();

    // Verify description is extracted
    assert_eq!(
        retrieved.description,
        Some("Test entity with custom content".to_owned())
    );

    // Verify content contains original fields
    assert!(retrieved.content.get("properties").is_some());
    assert!(retrieved.content.get("customMetadata").is_some());
}

#[tokio::test]
async fn test_entity_segments_parsed_correctly() {
    let service = create_service();

    let entity = json!({
        "$id": "gts.myvendor.mypackage.mynamespace.mytype.v2~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object"
    });

    let _ = service.register(vec![entity]);
    service.switch_to_ready().unwrap();

    let retrieved = service
        .get("gts.myvendor.mypackage.mynamespace.mytype.v2~")
        .unwrap();

    assert_eq!(retrieved.vendor(), Some("myvendor"));
    assert_eq!(retrieved.package(), Some("mypackage"));
    assert_eq!(retrieved.namespace(), Some("mynamespace"));
    assert!(retrieved.is_type());
}

// =============================================================================
// Edge Cases Tests
// =============================================================================

#[tokio::test]
async fn test_gts_id_with_special_segments() {
    let service = create_service();

    // GTS IDs with underscores and numbers
    let entities = vec![
        json!({
            "$id": "gts.acme_corp.core_v2.events_ns.my_type_123.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        }),
        json!({
            "$id": "gts.vendor123.pkg456.ns789.type000.v99~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        }),
    ];

    let results = service.register(entities);
    assert!(
        results[0].is_ok(),
        "Underscores should be valid: {:?}",
        results[0]
    );
    assert!(
        results[1].is_ok(),
        "Numbers should be valid: {:?}",
        results[1]
    );

    service.switch_to_ready().unwrap();

    let e1 = service.get("gts.acme_corp.core_v2.events_ns.my_type_123.v1~");
    assert!(e1.is_ok());
}
