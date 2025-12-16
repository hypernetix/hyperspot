# Design: Types Registry Module - Phase 1.1

## Context

HyperSpot requires a centralized GTS (Global Type System) registry to manage type definitions and instances across the platform. This enables:
- Cross-vendor type interoperability
- Schema validation for API payloads
- Fine-grained access control based on type identifiers
- Extensible plugin architectures

Phase 1.1 focuses on establishing the foundational contracts and in-memory storage, deferring REST API (Phase 1.2) and database persistence (Phase 1.3) to later phases.

**Stakeholders**: Platform developers, module authors, third-party integrators

## Goals / Non-Goals

### Goals
- Define SDK API trait for inter-module communication
- Implement in-memory storage for GTS entities
- Integrate gts-rust library for standard GTS operations (OP#1-OP#11)
- Follow HyperSpot module conventions (DDD-light, SDK pattern)
- Ensure thread-safe concurrent access

### Non-Goals (Phase 1.1)
- REST API exposure (Phase 1.2)
- Database persistence (Phase 1.3)
- Tenant-level isolation (Phase 1.3)
- Dynamic provisioning via API (Phase 2)
- Event publishing on entity changes (Phase 2)

## Decisions

### 1. Use gts-rust as External Dependency

**Decision**: Integrate the official [gts-rust](https://github.com/GlobalTypeSystem/gts-rust) library for GTS operations.

**Rationale**:
- Official reference implementation ensures spec compliance
- Provides all 11 GTS operations out of the box
- Maintained by the GTS specification authors
- Avoids reimplementing complex parsing/validation logic

**Alternatives considered**:
- Reimplement GTS operations in-house → Higher effort, risk of spec drift
- Use gts-go via FFI → Complexity, performance overhead

### 2. SDK Pattern with Separate Crates

**Decision**: Follow the standard HyperSpot SDK pattern with two crates:
- `types-registry-sdk`: Public API trait, models, errors
- `types-registry`: Module implementation

**Rationale**:
- Consistent with other HyperSpot modules
- Consumers only depend on lightweight SDK crate
- Clear separation of public API and implementation

### 3. In-Memory Storage with DashMap

**Decision**: Use `DashMap` for thread-safe in-memory storage.

**Rationale**:
- Lock-free concurrent reads (common case)
- Fine-grained locking for writes
- Already used in HyperSpot (modkit patterns)
- Simple API, no async complexity

**Alternatives considered**:
- `RwLock<HashMap>` → Coarse-grained locking, potential contention
- `tokio::sync::RwLock` → Async overhead for simple operations
- SQLite in-memory → Overkill for Phase 1.1

### 4. GTS Entity Model

**Decision**: Define a unified `GtsEntity` model:

```rust
pub struct GtsEntity {
    pub id: Uuid,              // Deterministic UUID from GTS ID (OP#5)
    pub gts_id: String,        // Full GTS identifier string
    pub kind: GtsEntityKind,   // Type or Instance
    pub content: serde_json::Value,  // JSON Schema or JSON object
    pub metadata: GtsEntityMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum GtsEntityKind {
    Type,      // Schema definition (ends with ~)
    Instance,  // Object instance (no ~ suffix)
}

pub struct GtsEntityMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub vendor: String,
    pub package: String,
    pub namespace: String,
    pub type_name: String,
    pub version_major: u32,
    pub version_minor: Option<u32>,
}
```

**Rationale**:
- Captures all GTS identifier components for efficient querying
- Stores content as `serde_json::Value` for flexibility
- Metadata extracted from GTS ID for filtering without parsing

### 5. API Method Signatures

**Decision**: All SDK API methods accept `&SecurityCtx` as first parameter:

```rust
#[async_trait]
pub trait TypesRegistryApi: Send + Sync {
    async fn register(
        &self,
        ctx: &SecurityCtx,
        entity: NewGtsEntity,
    ) -> Result<GtsEntity, TypesRegistryError>;

    async fn list(
        &self,
        ctx: &SecurityCtx,
        query: ListQuery,
    ) -> Result<Page<GtsEntity>, TypesRegistryError>;

    async fn get(
        &self,
        ctx: &SecurityCtx,
        gts_id: &str,
    ) -> Result<GtsEntity, TypesRegistryError>;
}
```

**Rationale**:
- Consistent with HyperSpot conventions
- Enables future tenant isolation (Phase 1.3)
- Supports audit logging

### 6. Optional Validation on Registration

**Decision**: Support optional validation flag on registration:

```rust
pub struct NewGtsEntity {
    pub gts_id: String,
    pub content: serde_json::Value,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub validate_references: bool,  // If true, validate x-gts-ref references
}
```

**Rationale**:
- Allows fast registration without validation for trusted sources
- Enables strict validation for untrusted input
- Matches gts-rust registry behavior

## Data Model

### Storage Structure

```
InMemoryStorage {
    entities: DashMap<String, GtsEntity>,  // Key: GTS ID string
    by_uuid: DashMap<Uuid, String>,        // UUID → GTS ID lookup
    by_vendor: DashMap<String, HashSet<String>>,  // Vendor → GTS IDs index
}
```

### Query Model

```rust
pub struct ListEntitiesQuery {
    pub pattern: Option<String>,      // Wildcard pattern (OP#4)
    pub kind: Option<GtsEntityKind>,  // Filter by Type/Instance
    pub vendor: Option<String>,       // Filter by vendor
    pub package: Option<String>,      // Filter by package
    pub namespace: Option<String>,    // Filter by namespace
    pub limit: u32,                   // Page size (default: 25, max: 200)
    pub cursor: Option<String>,       // Pagination cursor
}
```

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| gts-rust API changes | Pin to specific version, monitor releases |
| In-memory data loss on restart | Acceptable for Phase 1.1; Phase 1.3 adds persistence |
| Large number of entities | Add pagination limits; Phase 1.3 uses database |
| Pattern matching performance | Index by vendor for common queries |

## Migration Plan

Phase 1.1 is greenfield — no migration needed.

Future phases:
- **Phase 1.2**: Add REST API layer on top of SDK
- **Phase 1.3**: Replace in-memory storage with database, add tenant isolation

## Open Questions

1. **gts-rust version**: Which version to pin? → Use latest stable (check crates.io)
2. **Validation strictness**: Should invalid GTS references block registration or just warn? → Block by default when `validate_references=true`
3. **Entity size limits**: Should we limit content size? → Defer to Phase 1.2 (REST layer)
