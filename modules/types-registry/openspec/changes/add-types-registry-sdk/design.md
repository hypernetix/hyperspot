# Design: Types Registry SDK

**Reference Implementation**: All modules in `modules/` folder (especially `file-parser`) and `examples/modkit/users_info` — follow these for SDK pattern, module layout, and ClientHub integration.

## Context

The Types Registry SDK provides the public API contracts that other modules depend on. Following the HyperSpot SDK pattern, this crate is lightweight and contains only:
- Trait definitions
- Data models
- Error types

No implementation logic — that lives in the module crate.

## Goals

- Define `TypesRegistryApi` trait for inter-module communication
- Provide `GtsEntity` model using gts-rust types
- Define `ListQuery` for filtering operations
- Define `TypesRegistryError` for error handling

## Non-Goals

- Implementation logic (deferred to `add-types-registry-module`)
- REST API (deferred to `add-types-registry-module`)
- Storage (deferred to `add-types-registry-module`)

## Decisions

### 1. Use gts-rust Types Directly

**Decision**: Re-export `GtsIdSegment` from gts-rust rather than creating wrapper types.

**Rationale**:
- Avoids duplication and conversion overhead
- Consumers already familiar with gts-rust types
- Consistent with GTS ecosystem

### 2. Async Trait with Object Safety

**Decision**: Use `#[async_trait]` and ensure `TypesRegistryApi` is object-safe for `Arc<dyn TypesRegistryApi>`.

```rust
#[async_trait]
pub trait TypesRegistryApi: Send + Sync {
    async fn register(&self, ctx: &SecurityCtx, entities: Vec<Value>) -> Result<Vec<GtsEntity>, TypesRegistryError>;
    async fn list(&self, ctx: &SecurityCtx, query: ListQuery) -> Result<Vec<GtsEntity>, TypesRegistryError>;
    async fn get(&self, ctx: &SecurityCtx, gts_id: &str) -> Result<GtsEntity, TypesRegistryError>;
}
```

**Rationale**:
- Enables ClientHub registration: `hub.register::<dyn TypesRegistryApi>(api)`
- Consistent with other HyperSpot SDK traits

### 3. GTS Entity Model

**Decision**: Define a generic `GtsEntity<C>` model using standard serde traits and reuse `GtsIdSegment` from gts-rust:

```rust
use serde::{Serialize, de::DeserializeOwned};
use gts::GtsIdSegment;  // From gts-rust OP#3

/// Generic GTS entity - content type is pluggable
pub struct GtsEntity<C = serde_json::Value>
where
    C: Serialize + DeserializeOwned + Clone,
{
    pub id: Uuid,                       // Deterministic UUID from GTS ID (OP#5)
    pub gts_id: String,                 // Full GTS identifier string
    pub segments: Vec<GtsIdSegment>,    // All parsed segments (chained IDs have multiple)
    pub content: C,                     // Generic content: Value or concrete type
    pub description: Option<String>,
}

/// Type alias for dynamic entities (default)
pub type DynGtsEntity = GtsEntity<serde_json::Value>;

pub enum GtsEntityKind {
    Type,      // GTS ID ends with ~
    Instance,  // GTS ID does not end with ~
}
```

**Rationale**:
- **Reuse gts-rust**: `GtsIdSegment` already has all parsed components
- **Standard serde**: No custom traits — just use `#[derive(Serialize, Deserialize)]`
- **Flexibility**: Use `serde_json::Value` when you don't care about the concrete type
- **Type safety**: Use concrete structs when you want compile-time guarantees

### 4. ListQuery Without Pagination

**Decision**: Defer pagination (limit, cursor) to Phase 1.2.

```rust
pub struct ListQuery {
    pub pattern: Option<String>,
    pub is_type: Option<bool>,
    pub vendor: Option<String>,
    pub package: Option<String>,
    pub namespace: Option<String>,
}
```

**Rationale**:
- Phase 1.1 focuses on core functionality
- In-memory storage doesn't need pagination initially
- Simplifies initial implementation

### 5. Error Types

**Decision**: Define `TypesRegistryError` enum with variants for all error cases:

```rust
#[derive(Debug, thiserror::Error)]
pub enum TypesRegistryError {
    #[error("Invalid GTS ID: {0}")]
    InvalidGtsId(String),
    
    #[error("Entity not found: {0}")]
    NotFound(String),
    
    #[error("Entity already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Not in production mode")]
    NotInProductionMode,
    
    #[error("Internal error: {0}")]
    Internal(String),
}
```

## Crate Structure

```
types-registry-sdk/
├── Cargo.toml
└── src/
    ├── lib.rs          # Re-exports
    ├── api.rs          # TypesRegistryApi trait
    ├── models.rs       # GtsEntity, GtsEntityKind, ListQuery
    └── error.rs        # TypesRegistryError
```

## Dependencies

**Note**: `gts-rust` is added as a git submodule in the `types-registry` module directory.

```toml
[dependencies]
gts-rust = { path = "gts-rust" }  # Git submodule at modules/types-registry/gts-rust
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v5"] }
thiserror = "1"
```
