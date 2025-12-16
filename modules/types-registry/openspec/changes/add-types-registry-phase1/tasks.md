# Tasks: Types Registry Module - Phase 1.1

## 1. SDK Crate Setup

- [ ] 1.1 Create `types-registry-sdk/Cargo.toml` with minimal dependencies
- [ ] 1.2 Define `GtsEntity` model (id, kind, schema/data, metadata)
- [ ] 1.3 Define `GtsEntityKind` enum (Type, Instance)
- [ ] 1.4 Define `TypesRegistryError` enum
- [ ] 1.5 Define `TypesRegistryApi` trait with 3 methods:
  - `register(&SecurityCtx, NewGtsEntity) -> Result<GtsEntity>`
  - `list(&SecurityCtx, ListQuery) -> Result<Page<GtsEntity>>`
  - `get(&SecurityCtx, GtsId) -> Result<GtsEntity>`
- [ ] 1.6 Define query/filter models for list operation

## 2. Module Crate Setup

- [ ] 2.1 Create `types-registry/Cargo.toml` with SDK and gts-rust dependencies
- [ ] 2.2 Create module structure following DDD-light pattern
- [ ] 2.3 Implement `#[modkit::module]` declaration

## 3. Domain Layer

- [ ] 3.1 Define domain error types
- [ ] 3.2 Define repository trait for GTS entities
- [ ] 3.3 Implement domain service with business logic
- [ ] 3.4 Add optional validation on registration (validate GTS references)

## 4. Infrastructure Layer

- [ ] 4.1 Implement in-memory repository
- [ ] 4.2 Add thread-safe storage with `DashMap` or `RwLock<HashMap>`

## 5. Local Client

- [ ] 5.1 Implement `TypesRegistryApi` trait for local client
- [ ] 5.2 Wire local client to ClientHub

## 6. GTS Operations Integration

- [ ] 6.1 Integrate gts-rust library
- [ ] 6.2 Expose OP#1 - ID Validation
- [ ] 6.3 Expose OP#2 - ID Extraction
- [ ] 6.4 Expose OP#3 - ID Parsing
- [ ] 6.5 Expose OP#4 - ID Pattern Matching
- [ ] 6.6 Expose OP#5 - ID to UUID Mapping
- [ ] 6.7 Expose OP#6 - Schema Validation
- [ ] 6.8 Expose OP#7 - Relationship Resolution
- [ ] 6.9 Expose OP#8 - Compatibility Checking (backward, forward, full)
- [ ] 6.10 Expose OP#9 - Version Casting
- [ ] 6.11 Expose OP#10 - Query Execution
- [ ] 6.12 Expose OP#11 - Attribute Access

## 7. Testing

- [ ] 7.1 Unit tests for SDK models
- [ ] 7.2 Unit tests for domain service
- [ ] 7.3 Unit tests for in-memory repository
- [ ] 7.4 Integration tests for GTS operations
- [ ] 7.5 Integration tests for full module flow

## 8. Documentation

- [ ] 8.1 Add rustdoc comments to public API
- [ ] 8.2 Update module README with usage examples
