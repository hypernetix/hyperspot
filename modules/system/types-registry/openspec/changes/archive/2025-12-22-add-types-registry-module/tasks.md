# Tasks: Types Registry Module

**Depends on**: `add-types-registry-sdk` must be completed first.

## 1. Module Crate Setup

- [x] 1.1 Create `types-registry/Cargo.toml` with SDK and gts-rust dependencies
- [x] 1.2 Create module structure following DDD-light pattern
- [x] 1.3 Implement `#[modkit::module]` declaration with `capabilities = [system, rest]`
- [x] 1.4 Implement `Module::init` with config loading and ClientHub registration

## 2. Domain Layer

- [x] 2.1 Define domain error types
- [x] 2.2 Define repository trait for GTS entities
- [x] 2.3 Implement domain service with business logic
- [x] 2.4 Use gts-rust built-in validations (no custom validation logic needed)

## 3. Infrastructure Layer

- [x] 3.1 Implement two-phase storage (configuration + production) with `GtsOps`
- [x] 3.2 Implement `switch_to_production()` validation and storage transition
- [x] 3.3 Extract GTS ID using `GtsConfig.entity_id_fields`, determine kind from `~` suffix

## 4. Local Client

- [x] 4.1 Implement `TypesRegistryClient` trait for local client
- [x] 4.2 Wire local client to ClientHub

## 5. REST API Layer

- [x] 5.1 Create REST DTOs (request/response models)
- [x] 5.2 Implement REST handlers for `register`, `list`, `get`
- [x] 5.3 Define REST routes and register with `api-gateway`
- [x] 5.4 Implement error mapping (domain errors → HTTP responses)
- [x] 5.5 Add OpenAPI documentation for endpoints

## 6. GTS Operations Integration (via gts-rust)

- [x] 6.1 Integrate gts-rust library (`GtsOps` as primary API)
- [x] 6.2 Use `validate_id()` for OP#1 - ID Validation
- [x] 6.3 Use `extract_id()` for OP#2 - ID Extraction (via `GtsConfig.entity_id_fields`)
- [x] 6.4 Use `parse_id()` for OP#3 - ID Parsing
- [x] 6.5 Use `match_id_pattern()` for OP#4 - ID Pattern Matching
- [x] 6.6 Use `uuid()` for OP#5 - ID to UUID Mapping
- [x] 6.7 Use `validate_schema()` / `validate_instance()` for OP#6 - Schema Validation (includes x-gts-ref)
- [x] 6.8 Use `resolve_relationships()` for OP#7 - Relationship Resolution (broken ref detection)
- [x] 6.9 Use `compatibility()` for OP#8 - Compatibility Checking (backward, forward, full)
- [x] 6.10 Use `cast()` for OP#9 - Version Casting
- [x] 6.11 Use `query()` for OP#10 - Query Execution
- [x] 6.12 Use `attr()` for OP#11 - Attribute Access

## 7. Testing (Target: 95% coverage)

- [x] 7.1 Unit tests for domain service (10 tests)
- [x] 7.2 Unit tests for in-memory repository (17 tests)
- [x] 7.3 Integration tests split into 5 files:
  - `registration_tests.rs` - 9 tests (registration flows, batch, REST handlers)
  - `query_tests.rs` - 12 tests (list filters, REST list/get handlers)
  - `type_instance_tests.rs` - 6 tests (type-instance validation)
  - `production_mode_tests.rs` - 10 tests (immediate validation, state transitions, concurrent)
  - `edge_cases_tests.rs` - 6 tests (error handling, GTS ID extraction, content verification)
- [x] 7.4 Common test utilities in `tests/common/mod.rs`
- [x] 7.5 Verify 95%+ code coverage (achieved: config 100%, local_client 100%, dto 99%, handlers 99%, service 99%, repo 97%, error 96%)
- [x] 7.6 Total: 103 tests (60 unit + 43 integration)

## 8. Documentation

- [x] 8.1 Add rustdoc comments to public API
- [x] 8.2 Update module README with usage examples
