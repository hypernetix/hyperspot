# Types Registry Specification

## ADDED Requirements

### Requirement: GTS Entity Registration

The system SHALL provide an API to register GTS entities (types and instances) in the registry using a two-phase approach.

Entity identification is based on `GtsConfig.entity_id_fields`. When processing a JSON object:
1. Check each field in `entity_id_fields` order (e.g., `$id`, `gtsId`, `id`)
2. If a GTS ID is found → entity is registerable (Type or Instance)
3. If no GTS ID field exists → **return error** (entity cannot be registered without GTS ID)

The registry handles two categories of registerable entities:

**1. Types (Well-known schemas)** — GTS ID ends with `~`
- Define JSON Schema for validation
- Examples: `gts.x.core.events.type.v1~`, `gts.x.core.events.topic.v1~`

**2. Instances (Well-known objects)** — GTS ID does NOT end with `~`
- Conform to a type schema (referenced via chained ID)
- Have their own registered GTS ID
- Examples: `gts.x.core.events.topic.v1~x.commerce.orders.orders.v1.0`

**Anonymous Objects** — Objects without any `entity_id_fields` match cannot be registered. Registration returns an error.

A registered GTS entity consists of:
- **GTS ID**: A valid GTS identifier. Types end with `~` (e.g., `gts.vendor.package.namespace.type.v1~`). Instances must reference a type via chained ID (e.g., `gts.vendor.package.namespace.type.v1~vendor.app.instance.v1.0`) — no root-level instances.
- **Kind**: Either `Type` (schema) or `Instance` (object) — determined by whether GTS ID ends with `~`
- **Content**: JSON Schema for types, JSON object for instances
- **Description**: Optional description string

Registration operates in two phases:

**Phase 1: Configuration (before `switch_to_production`)**
- Entities accumulate in temporary storage
- Only basic GTS ID format validation (no reference validation)
- Entities arrive in random order, may reference each other
- Entities in temporary storage are NOT queryable via `list()`/`get()`

**Phase 2: Production (after `switch_to_production` succeeds)**
- All subsequent `register()` calls validate immediately
- Entities are added directly to persistent storage
- Full validation: references, schema, circular dependencies

Registration SHALL use gts-rust built-in validations:
- Extract GTS ID from each JSON object using configured `entity_id_fields` (from `GtsConfig`)
- Validate the GTS ID format using gts-rust OP#1 (`validate_id`)
- Extract and parse the GTS ID components using gts-rust OP#3 (`parse_id`)
- Generate a deterministic UUID from the GTS ID using gts-rust OP#5 (`uuid`)
- In production mode:
  - For types: validate schema using gts-rust OP#6 (`validate_schema`) — includes JSON Schema meta-schema + x-gts-ref validation
  - For instances: validate against schema using gts-rust OP#6 (`validate_instance`) — includes schema conformance + x-gts-ref validation
  - Validate GTS references using gts-rust OP#7 (`resolve_relationships`)
- Store the entity in appropriate storage (temporary or persistent)

Registration SHALL return `Vec<RegisterResult>` where each result indicates success (with `GtsEntity`) or failure (with error and optional GTS ID) for each input item. This allows batch operations to report per-item errors without failing the entire batch.

#### Scenario: Register entities during startup (configuration phase)

- **GIVEN** the service is in configuration mode (before `switch_to_production`)
- **AND** multiple entities with cross-references are registered in random order
- **WHEN** the user calls `register` with entity data
- **THEN** the system stores entities in temporary storage without reference validation
- **AND** returns `Vec<RegisterResult>` with success for each stored entity
- **AND** entities are not yet queryable

#### Scenario: Register a type entity in production mode

- **GIVEN** the service is in production mode (after `switch_to_production`)
- **AND** a valid GTS type identifier `gts.acme.core.events.user_created.v1~`
- **AND** a valid JSON Schema as content
- **WHEN** the user calls `register` with the entity data
- **THEN** the system validates immediately and returns `RegisterResult::Ok` with the registered entity
- **AND** the entity is stored in persistent storage

#### Scenario: Reject invalid GTS ID format

- **GIVEN** an invalid GTS identifier `invalid-gts-id`
- **WHEN** the user calls `register` with the entity data
- **THEN** the system returns `RegisterResult::Err` with `TypesRegistryError::InvalidGtsId`

#### Scenario: Reject invalid reference in production mode

- **GIVEN** the service is in production mode
- **AND** an entity references a non-existent type via `x-gts-ref`
- **WHEN** the user calls `register` with the entity data
- **THEN** the system returns `RegisterResult::Err` with `TypesRegistryError::ValidationFailed`

#### Scenario: Reject duplicate entity registration

- **GIVEN** an entity with GTS ID `gts.acme.core.events.user_created.v1~` already exists
- **WHEN** the user calls `register` with the same GTS ID
- **THEN** the system returns `RegisterResult::Err` with `TypesRegistryError::AlreadyExists`

#### Scenario: Batch registration with mixed results

- **GIVEN** the service is in production mode
- **AND** a batch of 3 entities: 2 valid and 1 with invalid GTS ID
- **WHEN** the user calls `register` with all entities
- **THEN** the system returns `Vec<RegisterResult>` with 2 `Ok` and 1 `Err`
- **AND** the valid entities are stored in persistent storage
- **AND** `RegisterSummary::from_results()` shows `succeeded=2, failed=1`

---

### Requirement: Production Commit

The system SHALL provide a method to validate all entities in temporary storage and transition to production mode.

`switch_to_production` SHALL:
- Validate ALL entities in temporary storage comprehensively:
  - Reference validation (x-gts-ref points to existing entities)
  - Schema validation (instances conform to their type schemas)
  - Circular dependency detection
- On success: move all entities from temporary to persistent storage
- On failure: return a list of ALL validation errors (not just the first)
- After successful commit: switch to production mode

This method is called by the modkit master process during service startup.

#### Scenario: Successful commit to production

- **GIVEN** the service is in configuration mode
- **AND** all entities in temporary storage have valid references and schemas
- **WHEN** `switch_to_production` is called
- **THEN** all entities are moved to persistent storage
- **AND** the service enters production mode
- **AND** entities become queryable via `list()`/`get()`

#### Scenario: Commit fails with validation errors

- **GIVEN** the service is in configuration mode
- **AND** some entities in temporary storage have invalid references
- **WHEN** `switch_to_production` is called
- **THEN** the system returns a list of ALL validation errors
- **AND** entities remain in temporary storage (not moved to persistent)
- **AND** the service does NOT start

#### Scenario: Commit fails on circular dependency

- **GIVEN** the service is in configuration mode
- **AND** entity A references entity B which references entity A
- **WHEN** `switch_to_production` is called
- **THEN** the system returns a circular dependency error
- **AND** the service does NOT start

---

### Requirement: GTS Entity Listing

The system SHALL provide an API to list GTS entities with filtering support.

Listing SHALL support:
- Filtering by GTS ID pattern using OP#4 (ID Pattern Matching) with wildcards
- Filtering by entity kind (Type, Instance, or both)
- Filtering by vendor, package, namespace, or type components
- `segment_scope` control for chained GTS IDs:
  - `Primary` — match filters against only the first segment
  - `Any` — match filters against any segment in the chain (default)
- Query execution using OP#10 (Query Execution)

Note: Pagination (limit, cursor) deferred to Phase 1.2.

#### Scenario: List all entities

- **GIVEN** the registry contains multiple GTS entities
- **WHEN** the user calls `list` without filters
- **THEN** the system returns all registered entities

#### Scenario: Filter entities by wildcard pattern

- **GIVEN** the registry contains entities from vendors `acme` and `globex`
- **WHEN** the user calls `list` with pattern `gts.acme.*`
- **THEN** the system returns only entities from vendor `acme`

#### Scenario: Filter entities by kind

- **GIVEN** the registry contains both type and instance entities
- **WHEN** the user calls `list` with `kind=Type`
- **THEN** the system returns only type entities (ending with `~`)

#### Scenario: Filter chained entities by vendor with Any scope

- **GIVEN** the registry contains chained GTS IDs:
  - `gts.a.b.c.d.v1~globex.app.x.y.v1`
  - `gts.k.l.m.n.v1~globex.app.a.b.v1`
  - `gts.acme.x.y.z.v1~acme.a.b.c.v1~globex.app.a.b.v1`
- **WHEN** the user calls `list` with `vendor=globex` and `segment_scope=Any` (default)
- **THEN** the system returns all three entities (vendor matches ANY segment)
- **AND** this differs from wildcard `gts.globex.*` which only matches the first segment

#### Scenario: Filter chained entities by vendor with Primary scope

- **GIVEN** the registry contains chained GTS IDs:
  - `gts.a.b.c.d.v1~globex.app.x.y.v1`
  - `gts.globex.core.events.order.v1~acme.app.orders.v1`
- **WHEN** the user calls `list` with `vendor=globex` and `segment_scope=Primary`
- **THEN** the system returns only `gts.globex.core.events.order.v1~acme.app.orders.v1`
- **AND** the first entity is excluded because its primary segment vendor is `a`, not `globex`

#### Scenario: Return empty list when no matches

- **GIVEN** the registry contains no entities matching pattern `gts.unknown.*`
- **WHEN** the user calls `list` with that pattern
- **THEN** the system returns an empty list with no error

---

### Requirement: GTS Entity Retrieval

The system SHALL provide an API to retrieve a single GTS entity by its identifier.

Retrieval SHALL:
- Accept a valid GTS ID
- Return the full entity including content and description
- Support attribute access using OP#11 (Attribute Access) with `@` selector

#### Scenario: Get entity by exact ID

- **GIVEN** an entity with GTS ID `gts.acme.core.events.user_created.v1~` exists
- **WHEN** the user calls `get` with that ID
- **THEN** the system returns the complete entity with content and description

#### Scenario: Get entity attribute

- **GIVEN** an entity with GTS ID `gts.acme.core.events.user_created.v1~` exists
- **AND** the entity has a property `name` with value `"UserCreated"`
- **WHEN** the user calls `get` with ID `gts.acme.core.events.user_created.v1~@name`
- **THEN** the system returns the value `"UserCreated"`

#### Scenario: Return not found for non-existent entity

- **GIVEN** no entity with GTS ID `gts.unknown.pkg.ns.type.v1~` exists
- **WHEN** the user calls `get` with that ID
- **THEN** the system returns a not found error

---

### Requirement: GTS ID Validation (OP#1)

The system SHALL validate GTS identifier syntax according to the GTS specification.

Valid GTS identifiers MUST:
- Start with `gts.` prefix
- Contain segments: `vendor.package.namespace.type`
- Include version: `v<MAJOR>` or `v<MAJOR>.<MINOR>`
- End with `~` for type identifiers, no suffix for instance identifiers
- Use only lowercase ASCII letters, digits, and underscores in segments
- Have segments starting with a letter or underscore

#### Scenario: Validate correct type identifier

- **GIVEN** the identifier `gts.acme.core.events.user_created.v1~`
- **WHEN** the system validates the identifier
- **THEN** validation succeeds
- **AND** the identifier is recognized as a type (schema)

#### Scenario: Validate correct instance identifier

- **GIVEN** the identifier `gts.acme.core.events.user_created.v1~acme.app.events.user_created.v1.0`
- **WHEN** the system validates the identifier
- **THEN** validation succeeds
- **AND** the identifier is recognized as an instance (object) based on last segment

#### Scenario: Reject identifier with invalid prefix

- **GIVEN** the identifier `invalid.acme.core.events.type.v1~`
- **WHEN** the system validates the identifier
- **THEN** validation fails with error "Invalid GTS prefix"

#### Scenario: Reject identifier with uppercase characters

- **GIVEN** the identifier `gts.Acme.core.events.type.v1~`
- **WHEN** the system validates the identifier
- **THEN** validation fails with error "Segments must be lowercase"

---

### Requirement: GTS ID Parsing (OP#3)

The system SHALL parse GTS identifiers into their constituent components.

Parsing SHALL extract:
- `vendor`: The vendor/organization identifier
- `package`: The package/module identifier
- `namespace`: The namespace category (or `_` placeholder)
- `type_name`: The type name
- `version_major`: The major version number
- `version_minor`: The optional minor version number
- `is_type`: Whether the identifier represents a type (true) or instance (false)
- `segments`: For chained identifiers, all segments in order

#### Scenario: Parse simple type identifier

- **GIVEN** the identifier `gts.acme.core.events.user_created.v1.2~`
- **WHEN** the system parses the identifier
- **THEN** the result contains:
  - `vendor = "acme"`
  - `package = "core"`
  - `namespace = "events"`
  - `type_name = "user_created"`
  - `version_major = 1`
  - `version_minor = Some(2)`
  - `is_type = true`

#### Scenario: Parse chained identifier

- **GIVEN** the identifier `gts.x.core.events.event.v1~acme.app._.custom_event.v2~`
- **WHEN** the system parses the identifier
- **THEN** the result contains 2 segments
- **AND** the first segment has vendor `x` and is a base type
- **AND** the second segment has vendor `acme` and is a derived type

---

### Requirement: GTS ID Pattern Matching (OP#4)

The system SHALL match GTS identifiers against patterns containing wildcards per GTS spec §10.

Pattern matching rules (per GTS spec):
- Wildcard (`*`) must be used only **once**
- Wildcard must appear at the **end** of the pattern
- Wildcard is greedy — matches any sequence including `~` chain separator
- Wildcard must not be used with attribute selector (`@`) or query (`[]`)
- Pattern must start at the beginning of a valid segment

#### Scenario: Match with trailing wildcard

- **GIVEN** the pattern `gts.acme.core.events.*`
- **AND** the identifier `gts.acme.core.events.user_created.v1~`
- **WHEN** the system matches the identifier against the pattern
- **THEN** the match succeeds

#### Scenario: Match chained IDs with wildcard

- **GIVEN** the pattern `gts.acme.core.*`
- **AND** the identifier `gts.acme.core.events.user_created.v1~vendor.app.custom.v1`
- **WHEN** the system matches the identifier against the pattern
- **THEN** the match succeeds (wildcard is greedy, matches through `~`)

#### Scenario: No match with different vendor

- **GIVEN** the pattern `gts.acme.*`
- **AND** the identifier `gts.globex.core.events.order.v1~`
- **WHEN** the system matches the identifier against the pattern
- **THEN** the match fails

---

### Requirement: GTS ID to UUID Mapping (OP#5)

The system SHALL invoke gts-rust's `uuid()` method for UUID generation — do not re-invent semantics.

#### Scenario: Generate deterministic UUID

- **GIVEN** the identifier `gts.acme.core.events.user_created.v1~`
- **WHEN** the system generates a UUID twice
- **THEN** both UUIDs are identical

#### Scenario: Different versions produce different UUIDs

- **GIVEN** the identifiers `gts.acme.core.events.user_created.v1~acme.app.instance.v1.0` and `gts.acme.core.events.user_created.v1~acme.app.instance.v1.5`
- **WHEN** the system generates UUIDs for each
- **THEN** the UUIDs are different (full identifier is used, not just major version)

---

### Requirement: Schema Validation (OP#6)

The system SHALL validate JSON object instances against their corresponding JSON Schema types.

#### Scenario: Validate instance against schema

- **GIVEN** a type `gts.acme.core.events.user_created.v1~` with JSON Schema requiring `userId` field
- **AND** an instance `gts.acme.core.events.user_created.v1~acme.app.events.user_created.v1.0` with `userId` field
- **WHEN** the system validates the instance
- **THEN** validation succeeds

#### Scenario: Reject invalid instance

- **GIVEN** a type `gts.acme.core.events.user_created.v1~` with JSON Schema requiring `userId` field
- **AND** an instance missing the `userId` field
- **WHEN** the system validates the instance
- **THEN** validation fails with details about the missing field

---

### Requirement: Relationship Resolution (OP#7)

The system SHALL resolve inter-dependencies between GTS entities and detect broken references.

#### Scenario: Resolve valid references

- **GIVEN** a type that references another registered type via `x-gts-ref`
- **WHEN** the system resolves relationships
- **THEN** all references are resolved successfully

#### Scenario: Detect broken reference

- **GIVEN** a type that references a non-existent type `gts.unknown.pkg.ns.type.v1~`
- **WHEN** the system resolves relationships
- **THEN** the system reports the broken reference

---

### Requirement: Compatibility Checking (OP#8)

The system SHALL verify schema compatibility between different MINOR versions.

Compatibility modes:
- **Backward**: Old instances work with new schema
- **Forward**: New instances work with old schema
- **Full**: Both backward and forward compatible

#### Scenario: Check backward compatibility

- **GIVEN** schema v1.0 and schema v1.1 where v1.1 adds an optional field
- **WHEN** the system checks backward compatibility
- **THEN** the schemas are backward compatible

#### Scenario: Detect backward incompatibility

- **GIVEN** schema v1.0 and schema v1.1 where v1.1 removes a required field
- **WHEN** the system checks backward compatibility
- **THEN** the schemas are NOT backward compatible
- **AND** the system reports the incompatibility reason

---

### Requirement: Version Casting (OP#9)

The system SHALL transform instances between compatible MINOR versions.

#### Scenario: Cast instance to newer version

- **GIVEN** an instance conforming to v1.0
- **AND** v1.1 adds an optional field with default value
- **WHEN** the system casts the instance to v1.1
- **THEN** the casted instance includes the new field with default value

---

### Requirement: Query Execution (OP#10)

The system SHALL filter identifier collections using the GTS query language.

Note: Per GTS spec §10, wildcard (`*`) cannot be combined with query (`[]`). Use exact ID with query.

#### Scenario: Query with attribute filter

- **GIVEN** entities with various `status` values
- **WHEN** the user queries `gts.acme.core.events.order.v1~[status=active]`
- **THEN** only entities matching the exact type with `status=active` are returned

---

### Requirement: Attribute Access (OP#11)

The system SHALL retrieve property values using the attribute selector (`@`).

#### Scenario: Access top-level attribute

- **GIVEN** an entity with property `name = "UserCreated"`
- **WHEN** the user accesses `entity@name`
- **THEN** the value `"UserCreated"` is returned

#### Scenario: Access nested attribute

- **GIVEN** an entity with nested property `metadata.version = "1.0"`
- **WHEN** the user accesses `entity@metadata.version`
- **THEN** the value `"1.0"` is returned

---

### Requirement: In-Memory Storage

The system SHALL provide thread-safe in-memory storage for GTS entities in Phase 1.1.

Storage SHALL:
- Support concurrent read/write access
- Maintain entity ordering by registration time
- Support efficient lookup by GTS ID
- Support efficient pattern-based queries

#### Scenario: Concurrent entity registration

- **GIVEN** multiple concurrent registration requests
- **WHEN** all requests complete
- **THEN** all entities are stored correctly without data corruption

#### Scenario: Concurrent read during write

- **GIVEN** a read operation and a write operation occurring concurrently
- **WHEN** both operations complete
- **THEN** the read returns consistent data (either before or after the write)

---

### Requirement: REST API Endpoints

The system SHALL expose REST API endpoints for types registry operations.

Endpoints SHALL include:
- `POST /api/v1/types-registry/entities` — Register entities (batch)
- `GET /api/v1/types-registry/entities` — List entities with query params
- `GET /api/v1/types-registry/entities/{gts_id}` — Get entity by ID

#### Scenario: Register entities via REST

- **GIVEN** a valid JSON payload with entity data
- **WHEN** POST request is sent to `/api/v1/types-registry/entities`
- **THEN** entities are registered and response contains registered entities

#### Scenario: List entities via REST

- **GIVEN** registered entities in the registry
- **WHEN** GET request is sent to `/api/v1/types-registry/entities?pattern=gts.acme.*`
- **THEN** response contains filtered entities matching the pattern

#### Scenario: Get entity via REST

- **GIVEN** an entity with GTS ID `gts.acme.core.events.user_created.v1~`
- **WHEN** GET request is sent to `/api/v1/types-registry/entities/gts.acme.core.events.user_created.v1~`
- **THEN** response contains the complete entity

