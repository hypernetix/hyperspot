# Design: Types Registry Module

**Reference Implementation**: All modules in `modules/` folder (especially `file-parser`) and `examples/modkit/users_info` — follow these for SDK pattern, module layout, and ClientHub integration.

**Depends on**: `add-types-registry-sdk` must be implemented first.

## Context

Phase 1.1 focuses on establishing the foundational contracts, in-memory storage, and REST API, deferring database persistence (Phase 1.2) to later phases.

The Types Registry module implements the `TypesRegistryClient` trait from `types-registry-sdk` and provides:
- Two-phase registration (configuration → production)
- In-memory storage using gts-rust `GtsOps`
- REST API endpoints
- Full gts-rust integration (OP#1-OP#11)

**Stakeholders**: Platform developers, module authors, third-party integrators

## Goals

- Implement `TypesRegistryClient` trait
- Provide two-phase storage with validation
- Expose REST API endpoints
- Integrate all gts-rust operations
- **Achieve 95% unit test coverage** (critical component)

## Non-Goals (Phase 1.1)

- Database persistence (Phase 1.2)
- Tenant-level isolation (Phase 1.3)
- Dynamic provisioning via API (Phase 2)
- Event publishing on entity changes (Phase 2)

## Decisions

### 1. Use gts-rust as External Dependency

**Decision**: Integrate the official [gts-rust](https://github.com/GlobalTypeSystem/gts-rust) library for GTS operations as a git submodule.

**Setup**: `gts-rust` is added as a git submodule at `modules/types-registry/gts-rust`.

**Rationale**:
- Official reference implementation ensures spec compliance
- Provides all 11 GTS operations out of the box
- Maintained by the GTS specification authors
- Avoids reimplementing complex parsing/validation logic

### 2. Leverage gts-rust Built-in Validations

**Decision**: Use gts-rust's comprehensive validation capabilities instead of implementing custom validation logic.

**gts-rust provides the following validation operations:**

| Operation | Method | Description |
|-----------|--------|-------------|
| **OP#1 - ID Validation** | `validate_id(gts_id)` | Validates GTS ID syntax using regex patterns |
| **OP#6 - Schema Validation** | `validate_schema(gts_id)` | Validates schema against JSON Schema meta-schema + x-gts-ref constraints |
| **OP#6 - Instance Validation** | `validate_instance(gts_id)` | Validates instance against its schema + x-gts-ref constraints |
| **OP#7 - Reference Resolution** | `resolve_relationships(gts_id)` | Resolves all references, detects broken refs |
| **OP#8 - Compatibility** | `compatibility(old, new)` | Checks backward/forward/full compatibility between schema versions |

**Key gts-rust validation methods:**
```rust
impl GtsOps {
    pub fn validate_id(&self, gts_id: &str) -> GtsIdValidationResult;
    pub fn validate_schema(&mut self, gts_id: &str) -> GtsValidationResult;
    pub fn validate_instance(&mut self, gts_id: &str) -> GtsValidationResult;
    pub fn validate_entity(&mut self, gts_id: &str) -> GtsValidationResult;
    pub fn add_entity(&mut self, content: &Value, validate: bool) -> GtsAddEntityResult;
    pub fn compatibility(&mut self, old: &str, new: &str) -> GtsEntityCastResult;
}
```

### 3. Module Structure (DDD-light)

**Module Crate (`types-registry/`):**
```
types-registry/
├── Cargo.toml
└── src/
    ├── lib.rs              # Re-exports SDK + module
    ├── module.rs           # Module declaration
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

**Module Declaration (`module.rs`):**
```rust
#[modkit::module(
    name = "types_registry",
    capabilities = [system, rest]
)]
pub struct TypesRegistryModule {
    storage: arc_swap::ArcSwapOption<TypesRegistryStorage>,
}
```

- `system` — Core infrastructure module, initialized early in startup
- `rest` — Exposes REST API endpoints

**Module Init (Config + ClientHub Registration):**
```rust
#[async_trait]
impl Module for TypesRegistryModule {
    async fn init(&self, ctx: &ModuleCtx) -> Result<()> {
        // Load module configuration (GtsConfig for entity_id_fields, etc.)
        let cfg: TypesRegistryConfig = ctx.config()?;
        let gts_config = GtsConfig {
            entity_id_fields: cfg.entity_id_fields.clone(),
            // ... other GTS settings
        };
        
        // Create storage with GTS config
        let storage = Arc::new(TypesRegistryStorage::new(gts_config));
        self.storage.store(Some(storage.clone()));
        
        // Create local client and register in ClientHub
        let api: Arc<dyn TypesRegistryClient> = Arc::new(
            TypesRegistryLocalClient::new(storage)
        );
        
        // Register in ClientHub directly - consumers use hub.get::<dyn TypesRegistryClient>()?
        ctx.client_hub().register::<dyn TypesRegistryClient>(api);
        
        tracing::info!("Types registry module initialized");
        Ok(())
    }
}
```

**Reference modules for config pattern:**
- `file_parser/src/module.rs` — loads `FileParserConfig` via `ctx.config()?`
- `nodes_registry/src/module.rs` — ClientHub registration pattern

### 4. Use gts-rust In-Memory Cache

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
5. Phase 1.2 (DB): Populate `persistent` cache from database on startup

### 5. Two-Phase Registration Flow

**Decision**: Registration operates in two phases — configuration (pre-production) and production:

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
impl TypesRegistryClient for TypesRegistryLocalClient {
    async fn register(&self, ctx: &SecurityCtx, entities: Vec<Value>) -> Result<Vec<RegisterResult>, TypesRegistryError> {
        let mut results = Vec::with_capacity(entities.len());
        
        for entity in entities {
            let result = if self.storage.is_production.load(Ordering::SeqCst) {
                // Production: use gts-rust with validate=true
                match self.storage.persistent.add_entity(&entity, true) {
                    Ok(gts_entity) => RegisterResult::Ok(gts_entity),
                    Err(e) => RegisterResult::Err { 
                        gts_id: extract_gts_id(&entity), 
                        error: e.into() 
                    },
                }
            } else {
                // Configuration: use gts-rust with validate=false
                match self.storage.temporary.add_entity(&entity, false) {
                    Ok(gts_entity) => RegisterResult::Ok(gts_entity),
                    Err(e) => RegisterResult::Err { 
                        gts_id: extract_gts_id(&entity), 
                        error: e.into() 
                    },
                }
            };
            results.push(result);
        }
        
        Ok(results)
    }
}

impl TypesRegistryService {
    pub fn switch_to_production(&mut self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        for (gts_id, _) in self.storage.temporary.store.items() {
            let result = self.storage.temporary.validate_entity(&gts_id);
            if !result.ok {
                errors.push(ValidationError::new(&gts_id, &result.error));
            }
        }
        
        if !errors.is_empty() {
            return Err(errors);
        }
        
        self.storage.is_production.store(true, Ordering::SeqCst);
        Ok(())
    }
}
```

### 6. GTS Entity Categories

Entity identification is based on `GtsConfig.entity_id_fields`. When processing a JSON object:
1. Check each field in `entity_id_fields` order (e.g., `$id`, `gtsId`, `id`)
2. If a GTS ID is found → entity is registerable (Type or Instance)
3. If no GTS ID field exists → **return error**

**1. Types (Well-known schemas)** — GTS ID ends with `~`
**2. Instances (Well-known objects)** — GTS ID does NOT end with `~`

### 7. Module Configuration with GtsConfig

**Decision**: Use `GtsConfig` from gts-rust as part of the module's configuration:

```rust
pub struct GtsConfig {
    pub entity_id_fields: Vec<String>,  // ["$id", "gtsId", "id"]
    pub schema_id_fields: Vec<String>,  // ["$schema", "gtsTid", "type"]
}

pub struct TypesRegistryConfig {
    pub gts_config: GtsConfig,
    // ... other module-specific config
}
```

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| gts-rust API changes | Pin to specific version, monitor releases |
| In-memory data loss on restart | Acceptable for Phase 1.1; Phase 1.2 adds persistence |
| Large number of entities | Add pagination limits; Phase 1.2 uses database |
| Pattern matching performance | Index by vendor for common queries |

## Migration Plan

Phase 1.1 is greenfield — no migration needed.

Future phases:
- **Phase 1.2**: Replace in-memory storage with database, add tenant isolation
