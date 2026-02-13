#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Integration tests for type-instance validation

mod common;

use common::create_service;
use serde_json::json;
use types_registry_sdk::ListQuery;

// =============================================================================
// Type-Instance Validation Tests
// =============================================================================

#[tokio::test]
async fn test_type_with_valid_instances() {
    let service = create_service();

    // Register a base type schema (User type)
    // Types end with ~ and define the schema
    let user_type = json!({
        "$id": "gts://gts.acme.core.models.user.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "userId": { "type": "string" },
            "email": { "type": "string" },
            "age": { "type": "integer" },
            "isActive": { "type": "boolean" }
        },
        "required": ["userId", "email"],
        "description": "User entity type definition"
    });

    let type_result = service.register(vec![user_type]);
    assert!(
        type_result[0].is_ok(),
        "Type registration should succeed: {:?}",
        type_result[0]
    );

    // Switch to ready to enable validation
    service.switch_to_ready().unwrap();

    // Register valid instances that conform to the schema
    // Instances have format: parent~instance (at least 2 segments)
    // Note: "type" field is not needed - schema ID is derived from $id
    let valid_instance1 = json!({
        "id": "gts.acme.core.models.user.v1~acme.core.instances.user1.v1",
        "userId": "user-001",
        "email": "alice@example.com",
        "age": 30,
        "isActive": true
    });

    let valid_instance2 = json!({
        "id": "gts.acme.core.models.user.v1~acme.core.instances.user2.v1",
        "userId": "user-002",
        "email": "bob@example.com"
        // age and isActive are optional
    });

    let instance_results = service.register(vec![valid_instance1, valid_instance2]);

    // Both instances should be registered successfully
    assert!(
        instance_results[0].is_ok(),
        "First valid instance should succeed: {:?}",
        instance_results[0]
    );
    assert!(
        instance_results[1].is_ok(),
        "Second valid instance should succeed: {:?}",
        instance_results[1]
    );

    // Verify instances are retrievable
    let i1 = service.get("gts.acme.core.models.user.v1~acme.core.instances.user1.v1");
    assert!(i1.is_ok());
    assert!(i1.unwrap().is_instance());

    let i2 = service.get("gts.acme.core.models.user.v1~acme.core.instances.user2.v1");
    assert!(i2.is_ok());
}

#[tokio::test]
async fn test_type_with_invalid_instance_missing_required_field() {
    let service = create_service();

    // Register a type with required fields
    let order_type = json!({
        "$id": "gts://gts.acme.core.models.order.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "orderId": { "type": "string" },
            "customerId": { "type": "string" },
            "total": { "type": "number" }
        },
        "required": ["orderId", "customerId", "total"],
        "description": "Order entity type"
    });

    _ = service.register(vec![order_type]);
    service.switch_to_ready().unwrap();

    // Try to register an instance missing required "total" field
    // Note: "type" field is not needed - schema ID is derived from $id
    let invalid_instance = json!({
        "id": "gts.acme.core.models.order.v1~acme.core.instances.order1.v1",
        "orderId": "order-001",
        "customerId": "cust-001"
        // Missing required "total" field
    });

    let result = service.register(vec![invalid_instance]);

    // Instance should fail validation due to missing required field
    assert!(
        result[0].is_err(),
        "Instance missing required field should fail: {:?}",
        result[0]
    );
}

#[tokio::test]
async fn test_type_with_invalid_instance_wrong_field_type() {
    let service = create_service();

    // Register a type with specific field types
    let product_type = json!({
        "$id": "gts://gts.acme.core.models.product.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "productId": { "type": "string" },
            "name": { "type": "string" },
            "price": { "type": "number" },
            "quantity": { "type": "integer" }
        },
        "required": ["productId", "name", "price"],
        "description": "Product entity type"
    });

    _ = service.register(vec![product_type]);
    service.switch_to_ready().unwrap();

    // Try to register an instance with wrong type for "price" (string instead of number)
    // Note: "type" field is not needed - schema ID is derived from $id
    let invalid_instance = json!({
        "id": "gts.acme.core.models.product.v1~acme.core.instances.prod1.v1",
        "productId": "prod-001",
        "name": "Widget",
        "price": "not-a-number",  // Should be a number
        "quantity": 10
    });

    let result = service.register(vec![invalid_instance]);

    // Instance should fail validation due to type mismatch
    assert!(
        result[0].is_err(),
        "Instance with wrong field type should fail: {:?}",
        result[0]
    );
}

#[tokio::test]
async fn test_multiple_instances_of_same_type() {
    let service = create_service();

    // Register an event type
    let event_type = json!({
        "$id": "gts://gts.acme.core.events.user_action.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "eventId": { "type": "string" },
            "userId": { "type": "string" },
            "action": { "type": "string" },
            "timestamp": { "type": "string" }
        },
        "required": ["eventId", "userId", "action", "timestamp"],
        "description": "User action event"
    });

    _ = service.register(vec![event_type]);
    service.switch_to_ready().unwrap();

    // Register multiple instances of the same type (parent~instance format)
    // Note: "type" field is not needed - schema ID is derived from $id
    let instances = vec![
        json!({
            "id": "gts.acme.core.events.user_action.v1~acme.core.instances.event1.v1",
            "eventId": "evt-001",
            "userId": "user-001",
            "action": "login",
            "timestamp": "2024-01-15T10:30:00Z"
        }),
        json!({
            "id": "gts.acme.core.events.user_action.v1~acme.core.instances.event2.v1",
            "eventId": "evt-002",
            "userId": "user-001",
            "action": "purchase",
            "timestamp": "2024-01-15T11:00:00Z"
        }),
        json!({
            "id": "gts.acme.core.events.user_action.v1~acme.core.instances.event3.v1",
            "eventId": "evt-003",
            "userId": "user-002",
            "action": "logout",
            "timestamp": "2024-01-15T12:00:00Z"
        }),
    ];

    let results = service.register(instances);

    // All instances should succeed
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Instance {i} should succeed: {result:?}");
    }

    // Verify we can list all entities (1 type + 3 instances)
    let all = service.list(&ListQuery::default()).unwrap();
    assert_eq!(all.len(), 4);

    // Filter to get only instances (not types)
    let instances_only = service
        .list(&ListQuery::default().with_is_type(false))
        .unwrap();
    assert_eq!(instances_only.len(), 3);

    // Filter to get only the type
    let types_only = service
        .list(&ListQuery::default().with_is_type(true))
        .unwrap();
    assert_eq!(types_only.len(), 1);
}

#[tokio::test]
async fn test_nested_object_type_with_instance() {
    let service = create_service();

    // Register a complex type with nested objects
    let customer_type = json!({
        "$id": "gts://gts.acme.core.models.customer.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "customerId": { "type": "string" },
            "name": { "type": "string" },
            "billingAddress": {
                "type": "object",
                "properties": {
                    "street": { "type": "string" },
                    "city": { "type": "string" },
                    "country": { "type": "string" }
                },
                "required": ["street", "city", "country"]
            }
        },
        "required": ["customerId", "name", "billingAddress"],
        "description": "Customer with nested address"
    });

    _ = service.register(vec![customer_type]);
    service.switch_to_ready().unwrap();

    // Register a valid customer instance with nested address
    // Note: "type" field is not needed - schema ID is derived from $id
    let valid_customer = json!({
        "id": "gts.acme.core.models.customer.v1~acme.core.instances.cust1.v1",
        "customerId": "cust-001",
        "name": "Acme Corp",
        "billingAddress": {
            "street": "123 Main St",
            "city": "New York",
            "country": "USA"
        }
    });

    let result = service.register(vec![valid_customer]);
    assert!(
        result[0].is_ok(),
        "Customer with nested address should succeed: {:?}",
        result[0]
    );

    // Verify the instance
    let customer = service.get("gts.acme.core.models.customer.v1~acme.core.instances.cust1.v1");
    assert!(customer.is_ok());
    assert!(customer.unwrap().is_instance());
}

#[tokio::test]
async fn test_array_type_with_instance() {
    let service = create_service();

    // Register a type with array properties
    let cart_type = json!({
        "$id": "gts://gts.acme.core.models.cart.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "cartId": { "type": "string" },
            "items": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "productId": { "type": "string" },
                        "quantity": { "type": "integer" }
                    },
                    "required": ["productId", "quantity"]
                }
            }
        },
        "required": ["cartId", "items"],
        "description": "Shopping cart with array of items"
    });

    _ = service.register(vec![cart_type]);
    service.switch_to_ready().unwrap();

    // Register a valid cart instance with array items
    // Note: "type" field is not needed - schema ID is derived from $id
    let valid_cart = json!({
        "id": "gts.acme.core.models.cart.v1~acme.core.instances.cart1.v1",
        "cartId": "cart-001",
        "items": [
            { "productId": "prod-001", "quantity": 2 },
            { "productId": "prod-002", "quantity": 1 }
        ]
    });

    let result = service.register(vec![valid_cart]);
    assert!(
        result[0].is_ok(),
        "Cart with array items should succeed: {:?}",
        result[0]
    );
}

#[tokio::test]
async fn test_instance_with_mismatched_type_field_is_ignored_for_well_known_instances() {
    let service = create_service();

    // Register two different type schemas
    let user_type = json!({
        "$id": "gts://gts.acme.core.models.user.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "userId": { "type": "string" },
            "name": { "type": "string" }
        },
        "required": ["userId", "name"],
        "description": "User type"
    });

    let product_type = json!({
        "$id": "gts://gts.acme.core.models.product.v1~",
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "productId": { "type": "string" },
            "price": { "type": "number" }
        },
        "required": ["productId", "price"],
        "description": "Product type"
    });

    _ = service.register(vec![user_type, product_type]);
    service.switch_to_ready().unwrap();

    // Register an instance where:
    // - Instance ID indicates parent is "user" type (gts.acme.core.models.user.v1~)
    // - "type" field explicitly claims parent is "product" type (gts.acme.core.models.product.v1~)
    // Chained GTS ID ALWAYS takes priority over explicit type field, so validation uses user schema
    let mismatched_instance = json!({
        "id": "gts.acme.core.models.user.v1~acme.core.instances.user1.v1",
        "type": "gts.acme.core.models.product.v1~",
        "userId": "user-001",
        "name": "Alice"
    });

    let result = service.register(vec![mismatched_instance]);

    // The chained GTS ID takes priority over the explicit "type" field.
    // Since the instance has user fields and is validated against user schema (from chain),
    // it should SUCCEED. The explicit "type" field is ignored.
    assert!(
        result[0].is_ok(),
        "Instance should succeed validation using schema from chained ID: {:?}",
        result[0]
    );
}
