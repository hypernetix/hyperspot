# Tasks: Types Registry Module

**Depends on**: `add-types-registry-sdk` must be completed first.

## 1. Module Crate Setup

- [ ] 1.1 Create `types-registry/Cargo.toml` with SDK and gts-rust dependencies
- [ ] 1.2 Create module structure following DDD-light pattern
- [ ] 1.3 Implement `#[modkit::module]` declaration with `capabilities = [system, rest]`
- [ ] 1.4 Implement `Module::init` with config loading and ClientHub registration

## 2. Domain Layer

- [ ] 2.1 Define domain error types
- [ ] 2.2 Define repository trait for GTS entities
- [ ] 2.3 Implement domain service with business logic
- [ ] 2.4 Use gts-rust built-in validations (no custom validation logic needed)

## 3. Infrastructure Layer

- [ ] 3.1 Implement two-phase storage (configuration + production) with `GtsOps`
- [ ] 3.2 Implement `switch_to_production()` validation and storage transition
- [ ] 3.3 Extract GTS ID using `GtsConfig.entity_id_fields`, determine kind from `~` suffix

## 4. Local Client

- [ ] 4.1 Implement `TypesRegistryApi` trait for local client
- [ ] 4.2 Wire local client to ClientHub

## 5. REST API Layer

- [ ] 5.1 Create REST DTOs (request/response models)
- [ ] 5.2 Implement REST handlers for `register`, `list`, `get`
- [ ] 5.3 Define REST routes and register with `api_ingress`
- [ ] 5.4 Implement error mapping (domain errors → HTTP responses)
- [ ] 5.5 Add OpenAPI documentation for endpoints

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

- [ ] 7.1 Unit tests for domain service
- [ ] 7.2 Unit tests for in-memory repository
- [ ] 7.3 Integration tests for GTS operations
- [ ] 7.4 Integration tests for full module flow
- [ ] 7.5 Integration tests for REST API endpoints
- [ ] 7.6 Verify 95% code coverage (critical component)

## 8. Documentation

- [ ] 8.1 Add rustdoc comments to public API
- [ ] 8.2 Update module README with usage examples
