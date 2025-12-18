# Tasks: Types Registry Module - Phase 1.1

## 1. SDK Crate Setup

- [ ] 1.1 Create `types-registry-sdk/Cargo.toml` with minimal dependencies
- [ ] 1.2 Define `GtsEntity` model using `GtsIdSegment` from gts-rust
- [ ] 1.4 Define `TypesRegistryError` enum
- [ ] 1.5 Define `TypesRegistryApi` trait with 3 methods:
  - `register(&SecurityCtx, Vec<serde_json::Value>) -> Result<Vec<GtsEntity>>` (batch registration)
  - `list(&SecurityCtx, ListQuery) -> Result<Page<GtsEntity>>`
  - `get(&SecurityCtx, &str) -> Result<GtsEntity>`
- [ ] 1.6 Define query/filter models for list operation

## 2. Module Crate Setup

- [ ] 2.1 Create `types-registry/Cargo.toml` with SDK and gts-rust dependencies
- [ ] 2.2 Create module structure following DDD-light pattern
- [ ] 2.3 Implement `#[modkit::module]` declaration

## 3. Domain Layer

- [ ] 3.1 Define domain error types
- [ ] 3.2 Define repository trait for GTS entities
- [ ] 3.3 Implement domain service with business logic
- [ ] 3.4 Use gts-rust built-in validations (no custom validation logic needed)

## 4. Infrastructure Layer

- [ ] 4.1 Implement two-phase storage (configuration + production) with `GtsOps`
- [ ] 4.2 Implement `switch_to_production()` validation and storage transition
- [ ] 4.3 Extract GTS ID using `GtsConfig.entity_id_fields`, determine kind from `~` suffix

## 5. Local Client

- [ ] 5.1 Implement `TypesRegistryApi` trait for local client
- [ ] 5.2 Wire local client to ClientHub

## 6. GTS Operations Integration (via gts-rust)

- [ ] 6.1 Integrate gts-rust library (`GtsOps` as primary API)
- [ ] 6.2 Use `validate_id()` for OP#1 - ID Validation
- [ ] 6.3 Use `extract_id()` for OP#2 - ID Extraction (via `GtsConfig.entity_id_fields`)
- [ ] 6.4 Use `parse_id()` for OP#3 - ID Parsing
- [ ] 6.5 Use `match_id_pattern()` for OP#4 - ID Pattern Matching
- [ ] 6.6 Use `uuid()` for OP#5 - ID to UUID Mapping
- [ ] 6.7 Use `validate_schema()` / `validate_instance()` for OP#6 - Schema Validation (includes x-gts-ref)
- [ ] 6.8 Use `resolve_relationships()` for OP#7 - Relationship Resolution (broken ref detection)
- [ ] 6.9 Use `compatibility()` for OP#8 - Compatibility Checking (backward, forward, full)
- [ ] 6.10 Use `cast()` for OP#9 - Version Casting
- [ ] 6.11 Use `query()` for OP#10 - Query Execution
- [ ] 6.12 Use `attr()` for OP#11 - Attribute Access

## 7. Testing (Target: 95% coverage)

- [ ] 7.1 Unit tests for SDK models
- [ ] 7.2 Unit tests for domain service
- [ ] 7.3 Unit tests for in-memory repository
- [ ] 7.4 Integration tests for GTS operations
- [ ] 7.5 Integration tests for full module flow
- [ ] 7.6 Verify 95% code coverage (critical component)

## 8. Documentation

- [ ] 8.1 Add rustdoc comments to public API
- [ ] 8.2 Update module README with usage examples
