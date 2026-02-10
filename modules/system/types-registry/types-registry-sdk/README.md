# Types Registry SDK

SDK crate for the Types Registry module, providing the public API contracts for GTS (Global Type System) entity management.

## Overview

This crate defines the transport-agnostic interface for the Types Registry module:

- **`TypesRegistryClient`** - Async trait for inter-module communication
- **`GtsEntity`** - Model representing registered GTS entities (types and instances)
- **`ListQuery`** - Query builder for filtering entity listings
- **`TypesRegistryError`** - Error types for all operations

## Usage

### Getting the Client

Consumers obtain the client from `ClientHub`:

```rust
use types_registry_sdk::{TypesRegistryClient, ListQuery};

// Get the client from ClientHub
let client = hub.get::<dyn TypesRegistryClient>()?;
```

### Registering Entities

```rust
use serde_json::json;

// Register a type schema
let schemas = vec![
    json!({
        "$id": "gts://gts.acme.core.events.user_created.v1~",
        "type": "object",
        "properties": {
            "user_id": { "type": "string" },
            "email": { "type": "string" }
        }
    })
];

let entities = client.register(&ctx, schemas).await?;
```

### Listing Entities

```rust
use types_registry_sdk::ListQuery;

// List all entities
let all = client.list(&ctx, ListQuery::default()).await?;

// List only types from vendor "acme"
let query = ListQuery::new()
    .with_is_type(true)
    .with_vendor("acme");
let acme_types = client.list(&ctx, query).await?;

// List entities matching a pattern
let query = ListQuery::new()
    .with_pattern("gts.acme.core.*");
let matched = client.list(&ctx, query).await?;
```

### Getting a Single Entity

```rust
let entity = client.get(&ctx, "gts.acme.core.events.user_created.v1~").await?;

println!("GTS ID: {}", entity.gts_id);
println!("Kind: {:?}", entity.kind);
println!("Vendor: {:?}", entity.vendor());
```

## Models

### GtsEntity

Represents a registered GTS entity:

```rust
pub struct GtsEntity<C = serde_json::Value> {
    pub id: Uuid,                    // Deterministic UUID from GTS ID
    pub gts_id: String,              // Full GTS identifier
    pub segments: Vec<GtsIdSegment>, // Parsed segments
    pub kind: GtsEntityKind,         // Type or Instance
    pub content: C,                  // Schema or object content
    pub description: Option<String>, // Optional description
}
```

### GtsIdSegment

Re-exported from `gts-rust`. Represents a parsed segment of a GTS identifier:

```rust
pub struct GtsIdSegment {
    pub num: usize,              // Segment number in chain
    pub offset: usize,           // Character offset in original string
    pub segment: String,         // Original segment string
    pub vendor: String,          // e.g., "acme"
    pub package: String,         // e.g., "core"
    pub namespace: String,       // e.g., "events"
    pub type_name: String,       // e.g., "user_created"
    pub ver_major: u32,          // e.g., 1
    pub ver_minor: Option<u32>,  // e.g., Some(0) for v1.0
    pub is_type: bool,           // true if segment ends with ~
    pub is_wildcard: bool,       // true if segment contains *
}
```

### GtsEntityKind

```rust
pub enum GtsEntityKind {
    Type,     // GTS ID ends with ~
    Instance, // GTS ID does not end with ~
}
```

### ListQuery

Builder for filtering entity listings:

```rust
let query = ListQuery::new()
    .with_pattern("gts.acme.*")
    .with_is_type(true)
    .with_vendor("acme")
    .with_package("core")
    .with_namespace("events");
```

## Error Handling

All API methods return `Result<T, TypesRegistryError>`:

```rust
match client.get(&ctx, gts_id).await {
    Ok(entity) => println!("Found: {}", entity.gts_id),
    Err(TypesRegistryError::NotFound(id)) => println!("Not found: {}", id),
    Err(TypesRegistryError::InvalidGtsId(msg)) => println!("Invalid ID: {}", msg),
    Err(e) => println!("Error: {}", e),
}
```

## License

Apache-2.0
