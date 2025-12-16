# Types Registry Specification

## ADDED Requirements

### Requirement: GTS Entity Registration

The system SHALL provide an API to register GTS entities (types and instances) in the registry.

A GTS entity consists of:
- **GTS ID**: A valid GTS identifier (e.g., `gts.vendor.package.namespace.type.v1~` for types, `gts.vendor.package.namespace.type.v1.0` for instances)
- **Kind**: Either `Type` (schema) or `Instance` (object)
- **Content**: JSON Schema for types, JSON object for instances
- **Metadata**: Optional metadata (description, tags, created_at, updated_at)

Registration SHALL:
- Validate the GTS ID format using OP#1 (ID Validation)
- Extract and parse the GTS ID components using OP#3 (ID Parsing)
- Optionally validate GTS references within the content using OP#7 (Relationship Resolution)
- Generate a deterministic UUID from the GTS ID using OP#5 (ID to UUID Mapping)
- Store the entity in the registry

#### Scenario: Register a type entity successfully

- **GIVEN** a valid GTS type identifier `gts.acme.core.events.user_created.v1~`
- **AND** a valid JSON Schema as content
- **WHEN** the user calls `register` with the entity data
- **THEN** the system returns the registered entity with generated UUID
- **AND** the entity is stored in the registry

#### Scenario: Register an instance entity successfully

- **GIVEN** a valid GTS instance identifier `gts.acme.core.events.user_created.v1.0`
- **AND** a valid JSON object as content
- **WHEN** the user calls `register` with the entity data
- **THEN** the system returns the registered entity with generated UUID
- **AND** the entity is stored in the registry

#### Scenario: Reject invalid GTS ID format

- **GIVEN** an invalid GTS identifier `invalid-gts-id`
- **WHEN** the user calls `register` with the entity data
- **THEN** the system returns a validation error with details about the invalid format

#### Scenario: Reject duplicate entity registration

- **GIVEN** an entity with GTS ID `gts.acme.core.events.user_created.v1~` already exists
- **WHEN** the user calls `register` with the same GTS ID
- **THEN** the system returns a conflict error

---

### Requirement: GTS Entity Listing

The system SHALL provide an API to list GTS entities with filtering and pagination support.

Listing SHALL support:
- Filtering by GTS ID pattern using OP#4 (ID Pattern Matching) with wildcards
- Filtering by entity kind (Type, Instance, or both)
- Filtering by vendor, package, namespace, or type components
- Cursor-based pagination with configurable limit
- Query execution using OP#10 (Query Execution)

#### Scenario: List all entities with pagination

- **GIVEN** the registry contains 100 GTS entities
- **WHEN** the user calls `list` with `limit=25`
- **THEN** the system returns the first 25 entities
- **AND** the response includes a cursor for the next page

#### Scenario: Filter entities by wildcard pattern

- **GIVEN** the registry contains entities from vendors `acme` and `globex`
- **WHEN** the user calls `list` with pattern `gts.acme.*`
- **THEN** the system returns only entities from vendor `acme`

#### Scenario: Filter entities by kind

- **GIVEN** the registry contains both type and instance entities
- **WHEN** the user calls `list` with `kind=Type`
- **THEN** the system returns only type entities (ending with `~`)

#### Scenario: Return empty list when no matches

- **GIVEN** the registry contains no entities matching pattern `gts.unknown.*`
- **WHEN** the user calls `list` with that pattern
- **THEN** the system returns an empty list with no error

---

### Requirement: GTS Entity Retrieval

The system SHALL provide an API to retrieve a single GTS entity by its identifier.

Retrieval SHALL:
- Accept a valid GTS ID
- Return the full entity including content and metadata
- Support attribute access using OP#11 (Attribute Access) with `@` selector

#### Scenario: Get entity by exact ID

- **GIVEN** an entity with GTS ID `gts.acme.core.events.user_created.v1~` exists
- **WHEN** the user calls `get` with that ID
- **THEN** the system returns the complete entity with content and metadata

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

- **GIVEN** the identifier `gts.acme.core.events.user_created.v1.0`
- **WHEN** the system validates the identifier
- **THEN** validation succeeds
- **AND** the identifier is recognized as an instance (object)

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

The system SHALL match GTS identifiers against patterns containing wildcards.

Pattern matching SHALL support:
- `*` wildcard matching any single segment
- `**` wildcard matching zero or more segments
- Exact segment matching

#### Scenario: Match with single wildcard

- **GIVEN** the pattern `gts.acme.*.events.*`
- **AND** the identifier `gts.acme.core.events.user_created.v1~`
- **WHEN** the system matches the identifier against the pattern
- **THEN** the match succeeds

#### Scenario: No match with different vendor

- **GIVEN** the pattern `gts.acme.*`
- **AND** the identifier `gts.globex.core.events.order.v1~`
- **WHEN** the system matches the identifier against the pattern
- **THEN** the match fails

---

### Requirement: GTS ID to UUID Mapping (OP#5)

The system SHALL generate deterministic UUIDs from GTS identifiers.

UUID generation SHALL:
- Produce the same UUID for the same GTS ID (deterministic)
- Use UUID v5 with a GTS-specific namespace
- Support scoping by major version (same UUID for v1.0 and v1.5)

#### Scenario: Generate deterministic UUID

- **GIVEN** the identifier `gts.acme.core.events.user_created.v1~`
- **WHEN** the system generates a UUID twice
- **THEN** both UUIDs are identical

#### Scenario: Same major version produces same UUID

- **GIVEN** the identifiers `gts.acme.core.events.user_created.v1.0` and `gts.acme.core.events.user_created.v1.5`
- **WHEN** the system generates UUIDs with major scope
- **THEN** both UUIDs are identical

---

### Requirement: Schema Validation (OP#6)

The system SHALL validate JSON object instances against their corresponding JSON Schema types.

#### Scenario: Validate instance against schema

- **GIVEN** a type `gts.acme.core.events.user_created.v1~` with JSON Schema requiring `userId` field
- **AND** an instance `gts.acme.core.events.user_created.v1.0` with `userId` field
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

#### Scenario: Query with attribute filter

- **GIVEN** entities with various `status` values
- **WHEN** the user queries `gts.acme.core.*[status=active]`
- **THEN** only entities with `status=active` are returned

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
