# types-registry-sdk Specification

## Purpose

The Types Registry SDK provides the public API contracts for the Types Registry module. It defines the `TypesRegistryApi` trait, data models (`GtsEntity`, `ListQuery`, `RegisterResult`), and error types that enable other HyperSpot modules to interact with the Types Registry without depending on the full implementation.
## Requirements
### Requirement: TypesRegistryApi Trait

The SDK SHALL provide a `TypesRegistryApi` trait defining the public contract for types registry operations.

The trait SHALL define:
- `register` — Register GTS entities (types or instances) in batch, returning per-item results
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

#### Scenario: Register returns per-item results

- **GIVEN** a batch of entities to register
- **WHEN** calling `register`
- **THEN** returns `Vec<RegisterResult>` with success/error for each input item

---

### Requirement: GtsEntity Model

The SDK SHALL provide a generic `GtsEntity<C>` struct representing a registered GTS entity.

The struct SHALL contain:
- `id` — Deterministic UUID generated from GTS ID (UUID v5 with GTS namespace)
- `gts_id` — The full GTS identifier string
- `segments` — Parsed `GtsIdSegment` components from gts-rust
- `kind` — Either `Type` or `Instance` (via `GtsEntityKind` enum)
- `content` — Generic content type `C` (defaults to `serde_json::Value`)
- `description` — Optional description string

The SDK SHALL provide type aliases:
- `DynGtsEntity` — `GtsEntity<serde_json::Value>` for dynamic content
- `GtsTypeEntity` — `GtsEntity<TypeSchema>` for type definitions
- `GtsInstanceEntity` — `GtsEntity<InstanceObject>` for instances

#### Scenario: Entity from type schema

- **GIVEN** a JSON Schema with GTS ID `gts.acme.core.events.user_created.v1~`
- **WHEN** creating a `GtsEntity`
- **THEN** `kind` is `Type` and `segments` contains parsed components

#### Scenario: Entity from instance object

- **GIVEN** a JSON object with GTS ID `gts.acme.core.events.user_created.v1~acme.app.events.user_created.v1.0`
- **WHEN** creating a `GtsEntity`
- **THEN** `kind` is `Instance` and `segments` contains all chained segments

---

### Requirement: Content Wrapper Types

The SDK SHALL provide newtype wrappers for semantic clarity:
- `TypeSchema` — Wrapper for JSON Schema content in type definitions
- `InstanceObject` — Wrapper for instance object content

Both wrappers SHALL implement:
- `Deref` to `serde_json::Value` for transparent access
- `AsRef<serde_json::Value>` for reference access
- `From<serde_json::Value>` and `Into<serde_json::Value>` for conversions
- `new()` constructor and `into_inner()` consumer

#### Scenario: TypeSchema wraps JSON Schema

- **GIVEN** a JSON Schema value
- **WHEN** wrapped in `TypeSchema`
- **THEN** the inner value is accessible via `Deref` and can be converted back

#### Scenario: InstanceObject wraps instance data

- **GIVEN** a JSON object representing instance data
- **WHEN** wrapped in `InstanceObject`
- **THEN** the inner value is accessible via `Deref` and can be converted back

---

### Requirement: RegisterResult Enum

The SDK SHALL provide a `RegisterResult<C>` enum for per-item batch registration results.

The enum SHALL have variants:
- `Ok(GtsEntity<C>)` — Successfully registered entity
- `Err { gts_id: Option<String>, error: TypesRegistryError }` — Failed registration with optional GTS ID

The enum SHALL provide methods:
- `is_ok()` / `is_err()` — Check result status
- `as_result()` / `into_result()` — Convert to standard `Result`
- `ok()` / `err()` — Extract success or error value

The SDK SHALL provide `DynRegisterResult` type alias for `RegisterResult<serde_json::Value>`.

#### Scenario: Successful registration

- **GIVEN** a valid GTS entity
- **WHEN** registration succeeds
- **THEN** `RegisterResult::Ok` contains the registered entity

#### Scenario: Failed registration with GTS ID

- **GIVEN** an invalid GTS entity with extractable GTS ID
- **WHEN** registration fails
- **THEN** `RegisterResult::Err` contains the error and the attempted GTS ID

---

### Requirement: RegisterSummary Struct

The SDK SHALL provide a `RegisterSummary` struct for aggregate batch operation counts.

The struct SHALL contain:
- `succeeded` — Number of successfully registered entities
- `failed` — Number of failed registrations

The struct SHALL provide:
- `from_results()` — Create summary from a slice of `RegisterResult`
- `all_succeeded()` / `all_failed()` — Check if all items succeeded/failed
- `total()` — Total number of items processed

#### Scenario: Summary from mixed results

- **GIVEN** a batch with 3 successes and 2 failures
- **WHEN** creating `RegisterSummary::from_results()`
- **THEN** `succeeded` is 3, `failed` is 2, `total()` is 5

---

### Requirement: ListQuery Model

The SDK SHALL provide a `ListQuery` struct for filtering entity listings.

The struct SHALL support:
- `pattern` — Optional wildcard pattern for GTS ID matching
- `is_type` — Optional filter for Type (true) or Instance (false)
- `vendor` — Optional vendor filter
- `package` — Optional package filter
- `namespace` — Optional namespace filter
- `segment_scope` — Controls which segments filters match against (defaults to `Any`)

The struct SHALL provide a builder pattern with `with_*` methods.

Note: Pagination (limit, cursor) deferred to Phase 1.2.

#### Scenario: Query with no filters

- **GIVEN** a `ListQuery` with all fields `None`
- **WHEN** used in a list operation
- **THEN** all entities are returned

#### Scenario: Query with pattern filter

- **GIVEN** a `ListQuery` with `pattern = Some("gts.acme.*")`
- **WHEN** used in a list operation
- **THEN** only entities matching the pattern are returned

#### Scenario: Query with segment scope

- **GIVEN** a `ListQuery` with `vendor = "acme"` and `segment_scope = Primary`
- **WHEN** used in a list operation
- **THEN** only entities where the primary segment has vendor "acme" are returned

---

### Requirement: SegmentMatchScope Enum

The SDK SHALL provide a `SegmentMatchScope` enum controlling filter matching for chained GTS IDs.

The enum SHALL have variants:
- `Primary` — Match filters against only the primary (first) segment
- `Any` — Match filters against any segment in the chain (default)

#### Scenario: Primary scope matching

- **GIVEN** a chained GTS ID `gts.acme.core.events.order.v1~billing.invoices.line_item.v1`
- **WHEN** filtering with `vendor = "billing"` and `segment_scope = Primary`
- **THEN** the entity does NOT match (primary segment vendor is "acme")

#### Scenario: Any scope matching

- **GIVEN** a chained GTS ID `gts.acme.core.events.order.v1~billing.invoices.line_item.v1`
- **WHEN** filtering with `vendor = "billing"` and `segment_scope = Any`
- **THEN** the entity matches (second segment vendor is "billing")

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

The enum SHALL provide:
- Constructor methods: `invalid_gts_id()`, `not_found()`, `already_exists()`, `validation_failed()`, `not_in_production_mode()`, `internal()`
- Predicate methods: `is_not_found()`, `is_already_exists()`, `is_validation_failed()`, `is_invalid_gts_id()`

#### Scenario: Error implements std::error::Error

- **GIVEN** a `TypesRegistryError` variant
- **WHEN** used with `?` operator
- **THEN** the error propagates correctly with Display and Debug

