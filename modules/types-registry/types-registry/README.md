# Types Registry Module

GTS entity registration, storage, validation, and REST API endpoints for HyperSpot.

## Overview

The `types-registry` module provides:

- **Two-phase registration**: Configuration phase (no validation) → Production phase (full validation)
- **GTS entity storage**: In-memory storage using `gts-rust` for Phase 1.1
- **REST API**: Endpoints for registering, listing, and retrieving GTS entities
- **ClientHub integration**: Other modules access via `hub.get::<dyn TypesRegistryApi>()?`

## Architecture

```
types-registry/
├── src/
│   ├── api/rest/          # REST API layer (DTOs, handlers, routes)
│   ├── domain/            # Domain layer (error, repo trait, service)
│   ├── infra/storage/     # Infrastructure (in-memory repository)
│   ├── config.rs          # Module configuration
│   ├── local_client.rs    # TypesRegistryApi implementation
│   ├── module.rs          # Module declaration
│   └── lib.rs             # Crate root
└── Cargo.toml
```

## Usage

### Via ClientHub (Rust)

```rust
use types_registry_sdk::TypesRegistryApi;

// Get the client from ClientHub
let client = hub.get::<dyn TypesRegistryApi>()?;

// Register entities
let results = client.register(&ctx, entities).await?;

// List entities with filtering
let query = ListQuery::default().with_vendor("acme");
let entities = client.list(&ctx, query).await?;

// Get a single entity
let entity = client.get(&ctx, "gts.acme.core.events.user_created.v1~").await?;
```

### Via REST API

```bash
# Register entities
POST /types-registry/v1/entities
Content-Type: application/json

{
  "entities": [
    {
      "$id": "gts.acme.core.events.user_created.v1~",
      "type": "object",
      "properties": { "userId": { "type": "string" } }
    }
  ]
}

# List entities
GET /types-registry/v1/entities?vendor=acme&kind=type

# Get entity by ID
GET /types-registry/v1/entities/gts.acme.core.events.user_created.v1~
```

## Configuration

```yaml
types_registry:
  entity_id_fields:
    - "$id"
    - "gtsId"
    - "id"
  schema_id_fields:
    - "$schema"
    - "gtsTid"
    - "type"
```

## Two-Phase Registration

1. **Configuration Phase**: Entities are stored in temporary storage without full validation
2. **Production Phase**: Call `switch_to_production()` to validate all entities and move to persistent storage

```rust
// During module initialization (configuration phase)
registry.register(&ctx, entities).await?;

// When ready for production
module.switch_to_production()?;
```

## Dependencies

- `types-registry-sdk`: Public API trait, models, and errors
- `gts-rust`: Official GTS library for ID validation, parsing, and schema operations
- `modkit`: HyperSpot module framework

## Testing

```bash
cargo test -p types-registry
```
