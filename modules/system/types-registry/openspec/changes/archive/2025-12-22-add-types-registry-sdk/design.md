# Design: Types Registry SDK

**Reference Implementation**: All modules in `modules/` folder (especially `file-parser`) and `examples/modkit/users_info` — follow these for SDK pattern, module layout, and ClientHub integration.

## Context

The Types Registry SDK provides the public API contracts that other modules depend on. Following the HyperSpot SDK pattern, this crate is lightweight and contains only:
- Trait definitions
- Data models
- Error types

No implementation logic — that lives in the module crate.

## Goals

- Define `TypesRegistryClient` trait for inter-module communication
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

**Decision**: Use `#[async_trait]` and ensure `TypesRegistryClient` is object-safe for `Arc<dyn TypesRegistryClient>`.

```rust
#[async_trait]
pub trait TypesRegistryClient: Send + Sync {
    async fn register(&self, ctx: &SecurityCtx, entities: Vec<Value>) -> Result<Vec<RegisterResult>, TypesRegistryError>;
    async fn list(&self, ctx: &SecurityCtx, query: ListQuery) -> Result<Vec<GtsEntity>, TypesRegistryError>;
    async fn get(&self, ctx: &SecurityCtx, gts_id: &str) -> Result<GtsEntity, TypesRegistryError>;
}
```

**Rationale**:
- Enables ClientHub registration: `hub.register::<dyn TypesRegistryClient>(api)`
- Consistent with other HyperSpot SDK traits
- `register` returns `Vec<RegisterResult>` for per-item error reporting in batch operations

### 3. GTS Entity Model

**Decision**: Define a generic `GtsEntity<C>` model and reuse `GtsIdSegment` from gts-rust:

```rust
use gts::GtsIdSegment;  // From gts-rust

/// Generic GTS entity - content type is pluggable
pub struct GtsEntity<C = serde_json::Value> {
    pub id: Uuid,                       // Deterministic UUID from GTS ID (UUID v5 with GTS namespace)
    pub gts_id: String,                 // Full GTS identifier string
    pub segments: Vec<GtsIdSegment>,    // All parsed segments (chained IDs have multiple)
    pub kind: GtsEntityKind,            // Type or Instance
    pub content: C,                     // Generic content: Value or concrete type
    pub description: Option<String>,
}

/// Type alias for dynamic entities (default)
pub type DynGtsEntity = GtsEntity<serde_json::Value>;
pub type GtsTypeEntity = GtsEntity<TypeSchema>;
pub type GtsInstanceEntity = GtsEntity<InstanceObject>;

pub enum GtsEntityKind {
    Type,      // GTS ID ends with ~
    Instance,  // GTS ID does not end with ~
}
```

**Rationale**:
- **Reuse gts-rust**: `GtsIdSegment` already has all parsed components
- **No trait bounds on `C`**: Keeps the struct simple and flexible
- **Flexibility**: Use `serde_json::Value` when you don't care about the concrete type
- **Type safety**: Use `TypeSchema` or `InstanceObject` for semantic clarity

### 3.1 Content Wrapper Types

**Decision**: Provide newtype wrappers for semantic clarity:

```rust
/// Wrapper for JSON Schema content in type definitions
pub struct TypeSchema(pub serde_json::Value);

/// Wrapper for instance object content
pub struct InstanceObject(pub serde_json::Value);
```

Both implement `Deref`, `AsRef`, and `From`/`Into` conversions for ergonomic use.

**Rationale**:
- Semantic clarity when working with different entity types
- Type aliases (`GtsTypeEntity`, `GtsInstanceEntity`) provide compile-time distinction
- Transparent access via `Deref` means no API friction

### 3.2 RegisterResult for Batch Operations

**Decision**: Use a custom `RegisterResult<C>` enum instead of `Result<GtsEntity<C>, Error>`:

```rust
pub enum RegisterResult<C = serde_json::Value> {
    Ok(GtsEntity<C>),
    Err { gts_id: Option<String>, error: TypesRegistryError },
}
```

**Rationale**:
- Batch registration should not fail entirely if one item is invalid
- Per-item error reporting with the attempted GTS ID for debugging
- `RegisterSummary::from_results()` provides aggregate counts

### 4. ListQuery with SegmentMatchScope

**Decision**: Defer pagination (limit, cursor) to Phase 1.2. Add `segment_scope` for controlling filter matching on chained GTS IDs.

```rust
pub struct ListQuery {
    pub pattern: Option<String>,
    pub is_type: Option<bool>,
    pub vendor: Option<String>,
    pub package: Option<String>,
    pub namespace: Option<String>,
    pub segment_scope: SegmentMatchScope,  // defaults to Any
}

pub enum SegmentMatchScope {
    Primary,  // Match only the first segment
    Any,      // Match any segment in the chain (default)
}
```

**Rationale**:
- Phase 1.1 focuses on core functionality
- In-memory storage doesn't need pagination initially
- `segment_scope` provides flexibility for chained GTS ID filtering
- Default `Any` is most intuitive for users

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
    ├── api.rs          # TypesRegistryClient trait
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
