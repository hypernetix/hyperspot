# Types Registry SDK Specification

## ADDED Requirements

### Requirement: TypesRegistryApi Trait

The SDK SHALL provide a `TypesRegistryApi` trait defining the public contract for types registry operations.

The trait SHALL define:
- `register` — Register GTS entities (types or instances) in batch
- `list` — List GTS entities with filtering support
- `get` — Retrieve a single GTS entity by ID

#### Scenario: Trait is object-safe

- **GIVEN** the `TypesRegistryApi` trait
- **WHEN** used as `dyn TypesRegistryApi`
- **THEN** the trait compiles and can be used with `Arc<dyn TypesRegistryApi>`

#### Scenario: Trait methods are async

- **GIVEN** the `TypesRegistryApi` trait
- **WHEN** implementing the trait
- **THEN** all methods are async and return `Result` types

---

### Requirement: GtsEntity Model

The SDK SHALL provide a `GtsEntity` struct representing a registered GTS entity.

The struct SHALL contain:
- `gts_id` — The full GTS identifier string
- `segments` — Parsed `GtsIdSegment` components from gts-rust
- `kind` — Either `Type` or `Instance`
- `content` — The JSON content (schema or object)
- `description` — Optional description string
- `uuid` — Deterministic UUID generated from GTS ID

#### Scenario: Entity from type schema

- **GIVEN** a JSON Schema with GTS ID `gts.acme.core.events.user_created.v1~`
- **WHEN** creating a `GtsEntity`
- **THEN** `kind` is `Type` and `segments` contains parsed components

#### Scenario: Entity from instance object

- **GIVEN** a JSON object with GTS ID `gts.acme.core.events.user_created.v1~acme.app.events.user_created.v1.0`
- **WHEN** creating a `GtsEntity`
- **THEN** `kind` is `Instance` and `segments` contains all chained segments

---

### Requirement: ListQuery Model

The SDK SHALL provide a `ListQuery` struct for filtering entity listings.

The struct SHALL support:
- `pattern` — Optional wildcard pattern for GTS ID matching
- `is_type` — Optional filter for Type (true) or Instance (false)
- `vendor` — Optional vendor filter (matches any segment)
- `package` — Optional package filter
- `namespace` — Optional namespace filter

Note: Pagination (limit, cursor) deferred to Phase 1.2.

#### Scenario: Query with no filters

- **GIVEN** a `ListQuery` with all fields `None`
- **WHEN** used in a list operation
- **THEN** all entities are returned

#### Scenario: Query with pattern filter

- **GIVEN** a `ListQuery` with `pattern = Some("gts.acme.*")`
- **WHEN** used in a list operation
- **THEN** only entities matching the pattern are returned

---

### Requirement: TypesRegistryError Enum

The SDK SHALL provide a `TypesRegistryError` enum for error handling.

The enum SHALL include variants:
- `InvalidGtsId` — GTS ID format validation failed
- `NotFound` — Entity not found
- `AlreadyExists` — Entity with same GTS ID already registered
- `ValidationFailed` — Schema or reference validation failed
- `NotInProductionMode` — Operation requires production mode
- `Internal` — Internal error with message

#### Scenario: Error implements std::error::Error

- **GIVEN** a `TypesRegistryError` variant
- **WHEN** used with `?` operator
- **THEN** the error propagates correctly with Display and Debug

