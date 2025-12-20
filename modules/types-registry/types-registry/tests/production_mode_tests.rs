#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Integration tests for production mode behavior and immediate validation

mod common;

use common::create_service;
use serde_json::json;
use types_registry_sdk::ListQuery;

// =============================================================================
// Production Mode Immediate Validation Tests
// =============================================================================

#[tokio::test]
async fn test_production_mode_validates_immediately_with_correct_order() {
    let service = create_service();

    // First, switch to production mode
    let _ = service.switch_to_production();

    // Register parent type FIRST, then instances in a single call
    // In production mode, validation happens immediately so order matters
    let entities = vec![
        // Parent type - must be first for instances to validate against it
        json!({
            "$id": "gts.acme.core.models.person.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer" }
            },
            "required": ["name", "age"],
            "description": "Person type"
        }),
        // Instance 1 - valid, conforms to schema
        json!({
            "$id": "gts.acme.core.models.person.v1~acme.core.instances.person1.v1",
            "type": "gts.acme.core.models.person.v1~",
            "name": "Alice",
            "age": 30
        }),
        // Instance 2 - valid
        json!({
            "$id": "gts.acme.core.models.person.v1~acme.core.instances.person2.v1",
            "type": "gts.acme.core.models.person.v1~",
            "name": "Bob",
            "age": 25
        }),
        // Instance 3 - valid
        json!({
            "$id": "gts.acme.core.models.person.v1~acme.core.instances.person3.v1",
            "type": "gts.acme.core.models.person.v1~",
            "name": "Charlie",
            "age": 35
        }),
    ];

    let results = service.register(entities);

    // All should succeed when parent type is registered first
    assert_eq!(results.len(), 4);
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Entity {i} should succeed: {result:?}");
    }

    // Verify all entities are immediately available (production mode)
    let all = service.list(&ListQuery::default()).unwrap();
    assert_eq!(all.len(), 4, "All 4 entities should be registered");

    // Verify we have 1 type and 3 instances
    let types = service
        .list(&ListQuery::default().with_is_type(true))
        .unwrap();
    assert_eq!(types.len(), 1);

    let instances = service
        .list(&ListQuery::default().with_is_type(false))
        .unwrap();
    assert_eq!(instances.len(), 3);
}

#[tokio::test]
async fn test_production_mode_fails_when_instance_before_parent() {
    let service = create_service();

    // Switch to production mode
    let _ = service.switch_to_production();

    // Try to register instance BEFORE parent type - should fail
    // In production mode, validation is immediate so parent must exist
    let entities = vec![
        // Instance first - will fail because parent doesn't exist yet
        json!({
            "$id": "gts.acme.core.models.widget.v1~acme.core.instances.widget1.v1",
            "type": "gts.acme.core.models.widget.v1~",
            "widgetId": "w-001",
            "color": "red"
        }),
        // Parent type - registered after instance
        json!({
            "$id": "gts.acme.core.models.widget.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "widgetId": { "type": "string" },
                "color": { "type": "string" }
            },
            "required": ["widgetId", "color"]
        }),
    ];

    let results = service.register(entities);

    // Instance should fail - parent type not found
    assert!(
        results[0].is_err(),
        "Instance before parent should fail: {:?}",
        results[0]
    );

    // Parent type should succeed
    assert!(
        results[1].is_ok(),
        "Parent type should succeed: {:?}",
        results[1]
    );
}

#[tokio::test]
async fn test_production_mode_validates_invalid_instance_immediately() {
    let service = create_service();

    // Switch to production mode first
    let _ = service.switch_to_production();

    // Register parent type and an INVALID instance in one call
    // The instance is missing required "age" field
    let entities = vec![
        // Parent type
        json!({
            "$id": "gts.acme.core.models.employee.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "employeeId": { "type": "string" },
                "department": { "type": "string" },
                "salary": { "type": "number" }
            },
            "required": ["employeeId", "department", "salary"],
            "description": "Employee type"
        }),
        // Invalid instance - missing required "salary" field
        json!({
            "$id": "gts.acme.core.models.employee.v1~acme.core.instances.emp1.v1",
            "type": "gts.acme.core.models.employee.v1~",
            "employeeId": "emp-001",
            "department": "Engineering"
            // Missing required "salary" field
        }),
    ];

    let results = service.register(entities);

    // Parent type should succeed
    assert!(
        results[0].is_ok(),
        "Parent type should succeed: {:?}",
        results[0]
    );

    // Invalid instance should fail validation immediately in production mode
    assert!(
        results[1].is_err(),
        "Invalid instance should fail immediately in production: {:?}",
        results[1]
    );
}

#[tokio::test]
async fn test_production_mode_batch_with_valid_and_invalid_instances() {
    let service = create_service();

    // Switch to production mode
    let _ = service.switch_to_production();

    // Register type with mix of valid and invalid instances
    let entities = vec![
        // Parent type
        json!({
            "$id": "gts.acme.core.models.item.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "itemId": { "type": "string" },
                "price": { "type": "number" }
            },
            "required": ["itemId", "price"],
            "description": "Item type"
        }),
        // Valid instance
        json!({
            "$id": "gts.acme.core.models.item.v1~acme.core.instances.item1.v1",
            "type": "gts.acme.core.models.item.v1~",
            "itemId": "item-001",
            "price": 29.99
        }),
        // Invalid instance - wrong type for price
        json!({
            "$id": "gts.acme.core.models.item.v1~acme.core.instances.item2.v1",
            "type": "gts.acme.core.models.item.v1~",
            "itemId": "item-002",
            "price": "not-a-number"  // Should be number
        }),
        // Another valid instance
        json!({
            "$id": "gts.acme.core.models.item.v1~acme.core.instances.item3.v1",
            "type": "gts.acme.core.models.item.v1~",
            "itemId": "item-003",
            "price": 49.99
        }),
    ];

    let results = service.register(entities);

    assert_eq!(results.len(), 4);

    // Type should succeed
    assert!(results[0].is_ok(), "Type should succeed");

    // First instance should succeed
    assert!(results[1].is_ok(), "Valid instance 1 should succeed");

    // Second instance should fail (wrong type)
    assert!(results[2].is_err(), "Invalid instance should fail");

    // Third instance should succeed
    assert!(results[3].is_ok(), "Valid instance 3 should succeed");
}

// =============================================================================
// Configuration Mode Behavior Tests
// =============================================================================

#[tokio::test]
async fn test_configuration_mode_defers_validation() {
    let service = create_service();

    // In configuration mode, entities are stored without validation
    assert!(!service.is_production());

    // Register a type schema
    let type_schema = json!({
        "$id": "gts.acme.core.models.config_test.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "requiredField": { "type": "string" }
        },
        "required": ["requiredField"]
    });

    let _ = service.register(vec![type_schema]);

    // Register an instance that would fail validation (missing required field)
    // In configuration mode, this should succeed (deferred validation)
    let invalid_instance = json!({
        "$id": "gts.acme.core.models.config_test.v1~acme.core.instances.test1.v1",
        "type": "gts.acme.core.models.config_test.v1~"
        // Missing "requiredField"
    });

    let result = service.register(vec![invalid_instance]);
    // In configuration mode, registration succeeds (validation deferred)
    assert!(
        result[0].is_ok(),
        "Config mode should defer validation: {:?}",
        result[0]
    );
}

#[tokio::test]
async fn test_switch_to_production_validates_all_entities() {
    let service = create_service();

    // Register valid entities in configuration mode
    let entities = vec![
        json!({
            "$id": "gts.acme.core.events.valid1.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        }),
        json!({
            "$id": "gts.acme.core.events.valid2.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        }),
    ];

    let _ = service.register(entities);

    // Switch to production should succeed with valid entities
    let result = service.switch_to_production();
    assert!(
        result.is_ok(),
        "Switch to production should succeed with valid entities"
    );
    assert!(service.is_production());
}

// =============================================================================
// State Transition Tests
// =============================================================================

#[tokio::test]
async fn test_switch_to_production_is_idempotent() {
    let service = create_service();

    // Register something first
    let _ = service.register(vec![json!({
        "$id": "gts.acme.core.events.state_test.v1~",
        "type": "object"
    })]);

    // First switch should succeed
    let first_switch = service.switch_to_production();
    assert!(first_switch.is_ok());
    assert!(service.is_production());

    // Second switch is idempotent (already in production, no-op)
    let second_switch = service.switch_to_production();
    assert!(second_switch.is_ok(), "Second switch should be idempotent");
    assert!(service.is_production());
}

#[tokio::test]
async fn test_list_before_production_returns_empty() {
    let service = create_service();

    // Register entities in configuration mode
    let _ = service.register(vec![json!({
        "$id": "gts.acme.core.events.not_visible.v1~",
        "type": "object"
    })]);

    // List should return empty before switching to production
    let results = service.list(&ListQuery::default()).unwrap();
    assert!(
        results.is_empty(),
        "List should be empty before production mode"
    );
}

#[tokio::test]
async fn test_get_before_production_fails() {
    let service = create_service();

    // Register entity in configuration mode
    let _ = service.register(vec![json!({
        "$id": "gts.acme.core.events.not_accessible.v1~",
        "type": "object"
    })]);

    // Get should fail before switching to production
    let result = service.get("gts.acme.core.events.not_accessible.v1~");
    assert!(result.is_err(), "Get should fail before production mode");
}

// =============================================================================
// Switch to Production Validation Error Reporting Tests
// =============================================================================

#[tokio::test]
async fn test_switch_to_production_returns_errors_as_list() {
    let service = create_service();

    // Register a parent type schema
    let parent_type = json!({
        "$id": "gts.acme.core.models.product.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "productId": { "type": "string" },
            "name": { "type": "string" },
            "price": { "type": "number" }
        },
        "required": ["productId", "name", "price"]
    });

    let _ = service.register(vec![parent_type]);

    // Register multiple invalid child instances in configuration mode
    // Child 1: Missing all required fields
    let invalid_child1 = json!({
        "$id": "gts.acme.core.models.product.v1~acme.core.instances.product1.v1",
        "type": "gts.acme.core.models.product.v1~"
        // Missing productId, name, price
    });

    // Child 2: Missing price field
    let invalid_child2 = json!({
        "$id": "gts.acme.core.models.product.v1~acme.core.instances.product2.v1",
        "type": "gts.acme.core.models.product.v1~",
        "productId": "prod-002",
        "name": "Widget"
        // Missing price
    });

    // Child 3: Wrong type for price (string instead of number)
    let invalid_child3 = json!({
        "$id": "gts.acme.core.models.product.v1~acme.core.instances.product3.v1",
        "type": "gts.acme.core.models.product.v1~",
        "productId": "prod-003",
        "name": "Gadget",
        "price": "not-a-number"  // Should be number
    });

    // All should succeed in configuration mode (validation deferred)
    let results = service.register(vec![invalid_child1, invalid_child2, invalid_child3]);
    assert!(
        results[0].is_ok(),
        "Config mode should accept invalid child 1"
    );
    assert!(
        results[1].is_ok(),
        "Config mode should accept invalid child 2"
    );
    assert!(
        results[2].is_ok(),
        "Config mode should accept invalid child 3"
    );

    // Switch to production should fail
    let switch_result = service.switch_to_production();
    assert!(
        switch_result.is_err(),
        "Switch to production should fail with invalid children"
    );

    let error = switch_result.unwrap_err();

    // Access the errors as a list
    let validation_errors = error.validation_errors();
    assert!(validation_errors.is_some(), "Should have validation errors");

    let errors = validation_errors.unwrap();

    // Should have at least 2 errors (one for each invalid child)
    assert!(
        errors.len() >= 2,
        "Should have at least 2 validation errors, got {}: {:?}",
        errors.len(),
        errors
    );

    // Each error should have the GTS ID of the failing entity
    for err in errors {
        assert!(
            err.gts_id.contains("gts.acme.core.models.product.v1~"),
            "Error should contain GTS ID: {err:?}"
        );
    }

    // Verify we can access typed fields
    println!("Validation errors returned as typed structs:");
    for (i, err) in errors.iter().enumerate() {
        println!(
            "  Error {}: gts_id={}, message={}",
            i + 1,
            err.gts_id,
            err.message
        );
    }
}

#[tokio::test]
async fn test_switch_to_production_error_contains_gts_ids() {
    let service = create_service();

    // Register a type schema
    let type_schema = json!({
        "$id": "gts.acme.core.models.error_test.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "name": { "type": "string" }
        },
        "required": ["name"]
    });

    let _ = service.register(vec![type_schema]);

    // Register an invalid instance
    let invalid_instance = json!({
        "$id": "gts.acme.core.models.error_test.v1~acme.core.instances.missing_name.v1",
        "type": "gts.acme.core.models.error_test.v1~"
        // Missing required "name" field
    });

    let _ = service.register(vec![invalid_instance]);

    // Switch to production should fail
    let switch_result = service.switch_to_production();
    assert!(switch_result.is_err());

    let error = switch_result.unwrap_err();

    // Access typed validation errors
    let validation_errors = error.validation_errors();
    assert!(validation_errors.is_some(), "Should have validation errors");

    let errors = validation_errors.unwrap();
    assert_eq!(errors.len(), 1, "Should have exactly 1 validation error");

    // The error should contain the GTS ID of the failing entity
    let err = &errors[0];
    assert!(
        err.gts_id
            .contains("gts.acme.core.models.error_test.v1~acme.core.instances.missing_name.v1"),
        "Error gts_id should reference the failing entity: {err:?}"
    );
    assert!(
        err.message.contains("name") || err.message.contains("required"),
        "Error message should mention the missing field: {err:?}"
    );
}

#[tokio::test]
async fn test_switch_to_production_success_with_valid_types_only() {
    let service = create_service();

    // Register only valid type schemas (no instances that need validation)
    let type_schema1 = json!({
        "$id": "gts.acme.core.models.fixable1.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "value": { "type": "string" }
        }
    });

    let type_schema2 = json!({
        "$id": "gts.acme.core.models.fixable2.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "count": { "type": "integer" }
        }
    });

    let _ = service.register(vec![type_schema1, type_schema2]);

    // Switch to production should succeed with valid type schemas
    let switch_result = service.switch_to_production();
    assert!(
        switch_result.is_ok(),
        "Switch should succeed with valid types: {switch_result:?}"
    );
    assert!(service.is_production());

    // Verify entities are accessible
    let all = service.list(&ListQuery::default()).unwrap();
    assert_eq!(all.len(), 2);
}

// =============================================================================
// Concurrent Access Tests
// =============================================================================

#[tokio::test]
async fn test_concurrent_registrations() {
    let service = create_service();

    // Spawn multiple concurrent registration tasks
    let mut handles = vec![];

    for i in 0..10 {
        let svc = service.clone();
        let handle = tokio::spawn(async move {
            let entity = json!({
                "$id": format!("gts.acme.core.events.concurrent_{i}.v1~"),
                "type": "object"
            });
            svc.register(vec![entity])
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok());
        let register_results = result.unwrap();
        assert!(register_results[0].is_ok());
    }

    // Switch to production and verify all entities
    let _ = service.switch_to_production();

    let all = service.list(&ListQuery::default()).unwrap();
    assert_eq!(all.len(), 10);
}
