# Design: Types Registry Module - Phase 1.1

## Context

HyperSpot requires a centralized GTS (Global Type System) registry to manage type definitions and instances across the platform. This enables:
- Cross-vendor type interoperability
- Schema validation for API payloads
- Fine-grained access control based on type identifiers
- Extensible plugin architectures

Phase 1.1 focuses on establishing the foundational contracts and in-memory storage, deferring REST API (Phase 1.2) and database persistence (Phase 1.3) to later phases.

**Reference Implementation**: `examples/modkit/users_info` — follow this structure for SDK pattern, module layout, and ClientHub integration.

**Stakeholders**: Platform developers, module authors, third-party integrators

## Goals / Non-Goals

### Goals
- Define SDK API trait for inter-module communication
- Implement in-memory storage for GTS entities
- Integrate gts-rust library for standard GTS operations (OP#1-OP#11)
- Follow HyperSpot module conventions (DDD-light, SDK pattern)
- Ensure thread-safe concurrent access
- **Achieve 95% unit test coverage** (critical component)

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

### 1a. Leverage gts-rust Built-in Validations

**Decision**: Use gts-rust's comprehensive validation capabilities instead of implementing custom validation logic.

**gts-rust provides the following validation operations:**

| Operation | Method | Description |
|-----------|--------|-------------|
| **OP#1 - ID Validation** | `validate_id(gts_id)` | Validates GTS ID syntax using regex patterns |
| **OP#6 - Schema Validation** | `validate_schema(gts_id)` | Validates schema against JSON Schema meta-schema + x-gts-ref constraints |
| **OP#6 - Instance Validation** | `validate_instance(gts_id)` | Validates instance against its schema + x-gts-ref constraints |
| **OP#7 - Reference Resolution** | `resolve_relationships(gts_id)` | Resolves all references, detects broken refs |
| **OP#8 - Compatibility** | `compatibility(old, new)` | Checks backward/forward/full compatibility between schema versions |

**x-gts-ref Validation** (built into gts-rust):
- Validates `x-gts-ref` annotations in schemas (GTS-specific reference constraints)
- Validates instance values against `x-gts-ref` patterns
- Supports both absolute GTS patterns (`gts.vendor.*`) and relative JSON pointers (`/$id`)
- Automatically strips `x-gts-ref` before JSON Schema compilation (jsonschema crate doesn't understand it)

**Key gts-rust validation methods:**
```rust
// GtsOps API - high-level operations
impl GtsOps {
    /// Validates GTS ID syntax (OP#1)
    pub fn validate_id(&self, gts_id: &str) -> GtsIdValidationResult;
    
    /// Validates schema against JSON Schema meta-schema + x-gts-ref (OP#6)
    pub fn validate_schema(&mut self, gts_id: &str) -> GtsValidationResult;
    
    /// Validates instance against its schema + x-gts-ref (OP#6)
    pub fn validate_instance(&mut self, gts_id: &str) -> GtsValidationResult;
    
    /// Validates entity (auto-detects schema vs instance by ~ suffix)
    pub fn validate_entity(&mut self, gts_id: &str) -> GtsValidationResult;
    
    /// Adds entity with optional validation
    pub fn add_entity(&mut self, content: &Value, validate: bool) -> GtsAddEntityResult;
    
    /// Checks schema compatibility (OP#8)
    pub fn compatibility(&mut self, old: &str, new: &str) -> GtsEntityCastResult;
}

// GtsStore - lower-level storage with validation
impl GtsStore {
    /// Registers entity (no validation)
    pub fn register(&mut self, entity: GtsEntity) -> Result<(), StoreError>;
    
    /// Validates schema (JSON Schema + x-gts-ref)
    pub fn validate_schema(&mut self, gts_id: &str) -> Result<(), StoreError>;
    
    /// Validates instance against schema + x-gts-ref
    pub fn validate_instance(&mut self, gts_id: &str) -> Result<(), StoreError>;
}
```

**Validation flow in gts-rust:**
1. **Schema validation** (`validate_schema`):
   - Validates x-gts-ref constraints first (catches invalid GTS patterns)
   - Removes x-gts-ref fields (jsonschema crate doesn't understand them)
   - Compiles schema with jsonschema crate to verify JSON Schema validity

2. **Instance validation** (`validate_instance`):
   - Resolves all `$ref` references by inlining them
   - Compiles resolved schema
   - Validates instance content against schema
   - Validates x-gts-ref constraints on instance values

**Integration with two-phase registration:**

The `TypesRegistryApi::register` method internally uses gts-rust's `add_entity(content, validate)` with different validation flags based on the current phase:

```rust
impl TypesRegistryApi for TypesRegistryLocalClient {
    /// Register entities - behavior depends on current phase
    async fn register(&self, ctx: &SecurityCtx, entities: Vec<Value>) -> Result<Vec<GtsEntity>> {
        if self.storage.is_production.load(Ordering::SeqCst) {
            // Production: use gts-rust with validate=true (immediate full validation)
            for entity in entities {
                self.storage.persistent.add_entity(&entity, true)?;
            }
        } else {
            // Configuration: use gts-rust with validate=false (only GTS ID format check)
            for entity in entities {
                self.storage.temporary.add_entity(&entity, false)?;
            }
        }
        // ...
    }
}

impl TypesRegistryService {
    /// Validate all staged entities and move to persistent storage
    pub fn switch_to_production(&mut self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        for (gts_id, _) in self.storage.temporary.store.items() {
            // Use gts-rust's validate_entity (auto-detects schema vs instance)
            let result = self.storage.temporary.validate_entity(&gts_id);
            if !result.ok {
                errors.push(ValidationError::new(&gts_id, &result.error));
            }
        }
        
        if !errors.is_empty() {
            return Err(errors);
        }
        
        // Move all entities from temporary to persistent
        self.storage.is_production.store(true, Ordering::SeqCst);
        Ok(())
    }
}
```

**Rationale**:
- Avoids reimplementing complex validation logic
- Ensures spec compliance with official implementation
- x-gts-ref validation is non-trivial (pattern matching, JSON pointer resolution)
- JSON Schema validation is handled by battle-tested jsonschema crate
- Consistent error messages with gts-rust ecosystem

### 2. SDK Pattern with Separate Crates

**Decision**: Follow the standard HyperSpot SDK pattern with two crates (mirroring `examples/modkit/users_info`):

**SDK Crate (`types-registry-sdk/`):**
```
types-registry-sdk/
├── Cargo.toml
└── src/
    ├── lib.rs          # Re-exports: TypesRegistryApi, models, errors
    ├── api.rs          # TypesRegistryApi trait definition
    ├── models.rs       # GtsEntity, NewGtsEntity, ListQuery, etc.
    └── errors.rs       # TypesRegistryError enum
```

**Module Crate (`types-registry/`):**
```
types-registry/
├── Cargo.toml
└── src/
    ├── lib.rs              # Re-exports SDK + module
    ├── module.rs           # #[modkit::module] declaration
    ├── local_client.rs     # TypesRegistryLocalClient implements SDK trait
    ├── config.rs           # TypesRegistryConfig
    ├── domain/
    │   ├── mod.rs
    │   ├── service.rs      # Domain service with business logic
    │   ├── error.rs        # DomainError
    │   ├── repo.rs         # GtsRepository trait (port)
    │   └── ports/          # Output ports (EventPublisher, etc.)
    └── infra/
        └── storage/
            └── in_memory_repo.rs  # In-memory repository implementation
```

**Rationale**:
- Consistent with `users_info` example and other HyperSpot modules
- Consumers only depend on lightweight SDK crate
- Clear separation of public API and implementation
- DDD-light structure with ports/adapters

### 3. Use gts-rust In-Memory Cache

**Decision**: Use `gts-rust`'s built-in `GtsOps` cache as the storage layer, with two instances for two-phase registration.

**Architecture**:
```rust
pub struct TypesRegistryStorage {
    temporary: GtsOps,       // Temporary storage during configuration phase
    persistent: GtsOps,      // Persistent storage after validation
    is_production: AtomicBool,  // Flag indicating production mode
}
```

**Flow**:
1. On `register` (configuration phase): Store entity in `temporary` cache (no validation)
2. On `switch_to_production`: Validate all entities, move from `temporary` → `persistent`
3. On `register` (production phase): Validate immediately, store in `persistent`
4. On `get`/`list`: Query `persistent` storage only
5. Phase 1.3 (DB): Populate `persistent` cache from database on startup

**Rationale**:
- Reuse gts-rust's optimized cache and operations
- gts-rust handles GTS ID parsing, validation, pattern matching internally
- No separate metadata — description is part of `GtsEntity` content
- Two-phase approach allows flexible startup with deferred validation
- Easy migration to DB in Phase 1.3 — just change where cache is populated from

**Alternatives considered**:
- Custom `DashMap<String, GtsEntity>` → Duplicates gts-rust functionality
- Wrap gts-rust entities entirely → Loses access to gts-rust operations
- Single storage with validation flag → More complex state management

### 4. GTS Entity Model

**Decision**: Define a generic `GtsEntity<C>` model using standard serde traits and reuse `GtsIdSegment` from gts-rust:

```rust
use serde::{Serialize, de::DeserializeOwned};
use gts::GtsIdSegment;  // From gts-rust OP#3

/// Generic GTS entity - content type is pluggable
/// Uses standard serde traits - no custom trait needed!
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

// GtsIdSegment from gts-rust provides:
// - vendor: String
// - package: String  
// - namespace: String
// - type_name: String
// - ver_major: Option<u32>
// - ver_minor: Option<u32>
// - is_type: bool  ← true for types (schemas), false for instances

// Chained IDs (e.g., gts.a.b.c.d.v1~globex.app.x.y.v1) have multiple segments.
// The last segment determines is_type for the entity.
```

**Usage examples**:

```rust
// Dynamic use (default) - content is serde_json::Value
let entity: DynGtsEntity = registry.get(&ctx, "gts.x.core.events.topic.v1~").await?;
let name = entity.content.get("name").and_then(|v| v.as_str());

// Type-safe use - just derive Serialize/Deserialize (standard serde)
#[derive(Clone, Serialize, Deserialize)]
struct EventTopic {
    name: String,
    retention: String,
    ordering: String,
}

// Get with concrete type - no custom trait impl needed!
let entity: GtsEntity<EventTopic> = registry.get_typed(&ctx, "gts.x.core.events.topic.v1~").await?;
println!("Topic name: {}", entity.content.name);  // Type-safe access!
```

**Rationale**:
- **Reuse gts-rust**: `GtsIdSegment` already has all parsed components — no custom struct needed
- **Standard serde**: No custom traits — just use `#[derive(Serialize, Deserialize)]`
- **Flexibility**: Use `serde_json::Value` when you don't care about the concrete type
- **Type safety**: Use concrete structs when you want compile-time guarantees
- **Zero-cost abstraction**: Generic is monomorphized at compile time
- `segment.is_type` indicates type vs instance

### 4a. GTS Entity Categories

Entity identification is based on `GtsConfig.entity_id_fields`. When processing a JSON object:
1. Check each field in `entity_id_fields` order (e.g., `$id`, `gtsId`, `id`)
2. If a GTS ID is found → entity is registerable (Type or Instance)
3. If no GTS ID field exists → **return error** (entity cannot be registered without GTS ID)

The registry handles two categories of registerable entities:

**1. Types (Well-known schemas)** — GTS ID ends with `~`
- Define JSON Schema for validation
- Examples:
  - `gts.x.core.events.type.v1~` — base event type schema
  - `gts.x.core.events.topic.v1~` — topic schema
  - `gts.x.core.events.type.v1~x.commerce.orders.order_placed.v1.0~` — chained type (event type extending base)

**2. Instances (Well-known objects)** — GTS ID does NOT end with `~`
- Conform to a type schema (referenced via chained ID)
- Have their own registered GTS ID
- Examples:
  - `gts.x.core.events.topic.v1~x.commerce.orders.orders.v1.0` — topic instance
  - `gts.x.core.modules.capability.v1~x.core.api.has_ws.v1` — capability instance

**Anonymous Objects** — No GTS ID field present
- Objects without any `entity_id_fields` match cannot be registered
- Registration **returns error** — caller must ensure all entities have GTS IDs
- Example: event payloads that reference a type via `type` field but have no own GTS ID

**Detection logic**:
```rust
/// Extracts GTS ID from a JSON entity and determines its category.
///
/// # Arguments
/// * `entity` - JSON object to extract GTS ID from
/// * `config` - GtsConfig containing field names to check for GTS ID
///
/// # Returns
/// * `Ok((gts_id, category))` - The extracted GTS ID and its category (Type or Instance)
/// * `Err(MissingGtsId)` - If no GTS ID field is found in the entity
fn extract_and_categorize(entity: &serde_json::Value, config: &GtsConfig) -> Result<(String, EntityCategory), TypesRegistryError> {
    // Try each field in order until we find a GTS ID
    let gts_id = config.entity_id_fields.iter()
        .find_map(|field| entity.get(field).and_then(|v| v.as_str()))
        .ok_or(TypesRegistryError::MissingGtsId)?;  // Error if no GTS ID found
    
    let category = if gts_id.ends_with('~') {
        EntityCategory::Type
    } else {
        EntityCategory::Instance
    };
    
    Ok((gts_id.to_string(), category))
}
```

**Note**: The registry only stores Types and Instances. Attempting to register an object without a GTS ID returns an error.

### 5. Two-Phase Registration Flow

**Decision**: Registration operates in two phases — configuration (pre-production) and production:

```rust
pub struct TypesRegistryStorage {
    temporary: GtsOps,  // Temporary storage during configuration phase (no validation)
    persistent: GtsOps,  // Persistent storage after validation
    is_production: AtomicBool,  // Flag indicating production mode
}
```

**Phase 1: Configuration (during service startup)**
- `register()` accumulates entities in temporary storage
- No reference validation (entities arrive in random order)
- Only basic GTS ID format validation
- Entities not yet queryable via `list()`/`get()`

**Phase 2: Production (after `switch_to_production()` succeeds)**
- `switch_to_production()` validates ALL staged entities:
  - Reference validation (x-gts-ref)
  - Schema validation for instances
  - Circular dependency detection
- On success: moves all entities from temporary → persistent
- On failure: returns list of all validation errors, service doesn't start
- After commit: `register()` validates immediately on each call

```rust
#[async_trait]
pub trait TypesRegistryApi: Send + Sync {
    /// Register multiple GTS entities (batch registration)
    /// - Before production: accumulates in temporary storage (no validation)
    /// - After production: validates immediately, adds to persistent storage
    /// GTS ID extracted from each JSON object using GtsConfig.entity_id_fields
    /// Returns error if any entity is missing GTS ID
    async fn register(
        &self,
        ctx: &SecurityCtx,
        entities: Vec<serde_json::Value>,  // JSON array of entity objects
    ) -> Result<Vec<GtsEntity>, TypesRegistryError>;

    /// List GTS entities with filtering and pagination.
    ///
    /// # Arguments
    /// * `ctx` - Security context for authorization
    /// * `query` - Query parameters (pattern, filters, pagination)
    ///
    /// # Returns
    /// Paginated list of entities matching the query from persistent storage
    async fn list(
        &self,
        ctx: &SecurityCtx,
        query: ListQuery,
    ) -> Result<Page<GtsEntity>, TypesRegistryError>;

    /// Retrieve a single GTS entity by its identifier.
    ///
    /// # Arguments
    /// * `ctx` - Security context for authorization
    /// * `gts_id` - Full GTS identifier string
    ///
    /// # Returns
    /// * `Ok(entity)` - The requested entity from persistent storage
    /// * `Err(NotFound)` - If no entity with the given ID exists
    async fn get(
        &self,
        ctx: &SecurityCtx,
        gts_id: &str,
    ) -> Result<GtsEntity, TypesRegistryError>;
}

impl TypesRegistryService {
    /// Validate all staged entities and move to persistent storage
    /// Called by modkit master process during startup
    /// Returns list of validation errors if any entity is invalid
    /// After success, all subsequent register() calls validate immediately
    pub fn switch_to_production(&self) -> Result<(), Vec<ValidationError>>;
}
```

**Rationale**:
- Entities registered during startup may reference each other in any order
- Deferred validation allows flexible module initialization
- Production mode ensures all references are valid before service starts
- Post-production registration validates immediately (trusted state)

### 6. API Method Signatures

**Rationale**:
- Consistent with HyperSpot conventions
- Enables future tenant isolation (Phase 1.3)
- Supports audit logging

### 7. Module Configuration with GtsConfig

**Decision**: Use `GtsConfig` from gts-rust as part of the module's configuration to identify GTS ID and schema fields from JSON objects:

```rust
/// From gts-rust - configures how to extract GTS IDs from JSON objects
pub struct GtsConfig {
    /// Field names to look for entity GTS ID (checked in order)
    /// Default: ["$id", "gtsId", "gtsIid", "gtsOid", "gtsI", "gts_id", "gts_oid", "gts_iid", "id"]
    pub entity_id_fields: Vec<String>,
    
    /// Field names to look for schema/type reference (checked in order)
    /// Default: ["$schema", "gtsTid", "gtsType", "gtsT", "gts_t", "gts_tid", "gts_type", "type", "schema"]
    pub schema_id_fields: Vec<String>,
}

pub struct TypesRegistryConfig {
    pub gts_config: GtsConfig,  // From gts-rust
    // ... other module-specific config
}
```

**Usage in register()**:
```rust
/// Registers multiple GTS entities from JSON objects.
///
/// Extracts GTS ID from each entity using configured field names,
/// validates format, and stores in appropriate storage based on phase.
///
/// # Arguments
/// * `ctx` - Security context for authorization
/// * `entities` - JSON objects to register
///
/// # Returns
/// * `Ok(entities)` - Successfully registered entities
/// * `Err(MissingGtsId)` - If any entity lacks a GTS ID field
async fn register(&self, ctx: &SecurityCtx, entities: Vec<serde_json::Value>) -> Result<Vec<GtsEntity>> {
    for entity in entities {
        // Use GtsConfig to extract GTS ID from the JSON object
        let gts_id = self.config.gts_config.entity_id_fields.iter()
            .find_map(|field| entity.get(field).and_then(|v| v.as_str()))
            .ok_or(TypesRegistryError::MissingGtsId)?;
        
        // ... register entity
    }
}
```

**Rationale**:
- Reuses gts-rust's standard configuration format
- Supports multiple field naming conventions (`$id`, `gtsId`, `id`, etc.)
- Consistent with gts.config.json used by gts-rust CLI
- Allows customization per deployment if needed

## Data Model

### Storage Structure

Storage uses two-phase architecture (see Decision #5 above):

```rust
pub struct TypesRegistryStorage {
    temporary: GtsOps,         // Temporary storage during configuration phase
    persistent: GtsOps,        // Persistent storage after validation
    is_production: AtomicBool, // Flag indicating production mode
}
```

### Query Flow

Queries only read from persistent storage (temporary storage is not queryable):

```rust
/// Lists GTS entities from persistent storage with filtering.
///
/// Queries persistent storage using gts-rust OP#10 for pattern matching,
/// then applies additional filters (kind, vendor) on the results.
///
/// # Arguments
/// * `query` - Query parameters including pattern, filters, and pagination
///
/// # Returns
/// Vector of entities matching all filter criteria
fn list(&self, query: &ListQuery) -> Vec<GtsEntity> {
    // Use gts-rust OP#10 for pattern-based queries (persistent only)
    let results = self.persistent.query(&query.pattern.unwrap_or("*".into()), query.limit);
    
    results.results
        .into_iter()
        .filter_map(|gts_entity| {
            // Parse GTS ID to get segments
            let parsed = self.persistent.parse_id(&gts_entity.gts_id);
            
            // Apply kind filter using last segment's is_type
            if let Some(is_type) = query.is_type {
                if parsed.segments.last().map_or(false, |s| s.is_type != is_type) {
                    return None;
                }
            }
            
            // Apply vendor filter - matches ANY segment (for chained IDs)
            if let Some(ref vendor) = query.vendor {
                if !parsed.segments.iter().any(|s| &s.vendor == vendor) {
                    return None;
                }
            }
            
            Some(self.to_gts_entity(gts_entity, parsed))
        })
        .collect()
}

/// Converts a gts-rust entity to our GtsEntity model.
///
/// # Arguments
/// * `gts_entity` - Raw entity from gts-rust storage
/// * `parsed` - Parsed GTS ID with segments
///
/// # Returns
/// GtsEntity with all fields populated
fn to_gts_entity(&self, gts_entity: GtsRustEntity, parsed: ParsedGtsId) -> GtsEntity {
    // ... conversion logic
}
```

**Why linear scan is acceptable for Phase 1.1:**
- In-memory storage is temporary (Phase 1.3 adds database with proper indexes)
- Expected entity count <10K for initial use cases
- HashMap iteration is efficient for small datasets
- Pattern matching via gts-rust is CPU-efficient (regex-based)
- Filters on pre-parsed metadata (vendor, kind) avoid re-parsing GTS IDs

### Query Model
```rust
pub struct ListQuery {
    pub pattern: Option<String>,      // Wildcard pattern (OP#4) - linear scan
    pub is_type: Option<bool>,        // Filter by Type (true) / Instance (false)
    pub vendor: Option<String>,       // Filter by vendor - O(1) per entity
    pub package: Option<String>,      // Filter by package - O(1) per entity
    pub namespace: Option<String>,    // Filter by namespace - O(1) per entity
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
