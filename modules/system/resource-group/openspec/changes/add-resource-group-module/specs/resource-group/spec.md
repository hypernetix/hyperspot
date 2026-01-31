## ADDED Requirements

### Requirement: Resource Group Type Management

The system SHALL provide an API to manage resource group types that define the schema and allowed parent-child relationships for resource groups.

A resource group type consists of:
- **Code**: Unique identifier (1-63 chars, no whitespace, case-insensitive)
- **Parents**: Array of type codes that can be parents of this type
- **Owner ID**: UUID of the owner
- **Owner Type**: Type of the owner (e.g., "app", "user")

Type management SHALL support:
- Creating new types with validation
- Listing all types
- Retrieving a specific type by code
- Updating type properties (parents, owner info)
- Deleting types (only if no entities of this type exist)

#### Scenario: Create a new resource group type

- **GIVEN** a valid type code `DEPARTMENT` and allowed parent types `["ORGANIZATION", "DIVISION"]`
- **AND** an authenticated application with ID `app-uuid`
- **WHEN** the user calls `create_type` with type data
- **THEN** the system creates the type with the application as owner
- **AND** returns the created type with `owner_id`

#### Scenario: Reject duplicate type code

- **GIVEN** a type with code `DEPARTMENT` already exists
- **WHEN** the user attempts to create another type with code `DEPARTMENT`
- **THEN** the system returns `ResourceGroupError::TypeAlreadyExists`

#### Scenario: Reject invalid type code format

- **GIVEN** a type code with whitespace `"DEP ARTMENT"` or length > 63
- **WHEN** the user attempts to create a type
- **THEN** the system returns `ResourceGroupError::Validation` with field-specific error

---

### Requirement: Resource Group Entity Management

The system SHALL provide an API to manage resource group entities organized in hierarchical structures.

A resource group entity consists of:
- **ID**: Unique identifier (UUIDv7)
- **Type Code**: Reference to resource group type
- **Name**: Display name (1-255 chars)
- **External ID**: Optional external identifier (max 255 chars)
- **Parent ID**: Optional reference to parent entity
- **Created/Modified**: Timestamps

Entity management SHALL support:
- Creating entities with optional parent (validates parent type compatibility)
- Retrieving entities by ID
- Updating entity properties (name, external_id)
- Moving entities to new parents (subtree move)
- Deleting entities (only if no active references exist)
- Querying ancestors and descendants efficiently

#### Scenario: Create entity with parent

- **GIVEN** a valid parent entity `parent-id` of type `ORGANIZATION`
- **AND** a type `DEPARTMENT` that allows `ORGANIZATION` as parent
- **WHEN** the user creates an entity of type `DEPARTMENT` with `parent_id = parent-id`
- **THEN** the system creates the entity
- **AND** creates closure table entries for all ancestors
- **AND** returns the created entity

#### Scenario: Reject invalid parent type

- **GIVEN** a parent entity of type `DEPARTMENT`
- **AND** a type `ORGANIZATION` that does NOT allow `DEPARTMENT` as parent
- **WHEN** the user attempts to create an `ORGANIZATION` entity with `DEPARTMENT` as parent
- **THEN** the system returns `ResourceGroupError::InvalidParentType`

#### Scenario: Move subtree to new parent

- **GIVEN** an entity `node-id` with descendants
- **AND** a valid new parent `new-parent-id`
- **WHEN** the user calls `move_entity` to move `node-id` to `new-parent-id`
- **THEN** the system moves the entire subtree
- **AND** rebuilds closure table entries for all affected nodes
- **AND** returns success

#### Scenario: Reject cycle creation

- **GIVEN** an entity `node-id` with a descendant `descendant-id`
- **WHEN** the user attempts to move `descendant-id` to be a child of `node-id`
- **THEN** the system returns `ResourceGroupError::CycleDetected`

---

### Requirement: Hierarchy Operations

The system SHALL provide efficient hierarchy queries using the Closure Table pattern.

Hierarchy operations SHALL support:
- Querying all ancestors of an entity (ordered by depth)
- Querying all descendants of an entity (ordered by depth)
- Efficient subtree operations (move, delete)

Hierarchy query operations (ancestors/descendants) SHALL apply service-level constraints configured via service configuration:
- **max_depth**: Maximum traversal depth (positive integer). If not configured, the system SHALL use a default value of `10`. The configured value MUST be `<= 10`.
- **max_width**: Maximum number of children to include per parent node in the response (positive integer). If not configured, the system SHALL not apply a width limit.

The system SHALL treat the effective constraint set `(max_depth, max_width)` as a **query profile** that can be used to define and track SLOs for hierarchy queries.

Changing these constraints MUST NOT delete or rewrite existing hierarchy data in the database.

When constraints are reduced and existing data exceeds the new limits, the operator SHALL independently implement and run a data-migration script/process to bring stored hierarchies into compliance with the new limits (e.g., restructure the tree, split nodes, or otherwise reduce depth/width).

If the data-migration has NOT been performed, then after reducing constraints:
- Read/query operations SHALL return all data stored in the database (no truncation/obrezanie due to the configured limits)
- Write operations that would create or increase a violation of the configured limits (e.g., create/move that increases depth beyond `max_depth`, or adds a child beyond `max_width`) SHALL be rejected and require reducing depth/width via data-migration (or increasing limits)

The closure table SHALL maintain:
- `parent_id`: Ancestor entity ID
- `child_id`: Descendant entity ID
- `depth`: Distance from parent to child (0 for self-reference)

#### Scenario: Query ancestors

- **GIVEN** an entity `node-id` in a hierarchy: `ROOT -> ORG -> DEPT -> node-id`
- **WHEN** the user calls `get_ancestors` for `node-id`
- **THEN** the system returns `[ROOT, ORG, DEPT]` ordered by depth (ascending)

#### Scenario: Query descendants

- **GIVEN** an entity `org-id` with descendants: `DEPT1, DEPT2, TEAM1` (child of DEPT1)
- **WHEN** the user calls `get_descendants` for `org-id`
- **THEN** the system applies the default constraint `max_depth = 10`
- **AND** returns all descendants `[DEPT1, DEPT2, TEAM1]` ordered by depth

#### Scenario: Query descendants with configured constraints

- **GIVEN** an entity `org-id` with descendants across multiple levels
- **AND** the service is configured with `max_depth = 3` and `max_width = 50`
- **WHEN** the user calls `get_descendants` for `org-id`
- **THEN** the system returns descendants up to depth 3
- **AND** for each parent in the returned set, includes at most 50 direct children

#### Scenario: Configuration reduced after deeper hierarchy already exists

- **GIVEN** a hierarchy exists in the database with depth > 3
- **AND** the service configuration is changed to `max_depth = 3`
- **WHEN** a client requests `descendants` for an entity with deeper descendants
- **THEN** the system returns all descendants stored in the database (including deeper nodes)
- **AND** the deeper nodes remain stored in the database

---

### Requirement: Resource Group References

The system SHALL provide functionality to link resource groups to external resources.

A resource group reference consists of:
- **Group ID**: Reference to resource group entity
- **Resource Type**: Type of external resource (string)
- **Resource ID**: External resource identifier (string)
- **Application ID**: Application that created the reference

Reference management SHALL support:
- Creating references from groups to external resources
- Deleting references
- Preventing deletion of groups with active references
- Reference counting for efficient deletion checks

#### Scenario: Create reference
- **GIVEN** a resource group entity `group-id`
- **AND** an external resource `resource-type/resource-id`
- **WHEN** the user calls `create_reference` with reference data
- **THEN** the system creates the reference
- **AND** increments reference count for the group
- **AND** returns the created reference

#### Scenario: Prevent deletion with active references
- **GIVEN** a resource group entity `group-id` with active references
- **WHEN** the user attempts to delete `group-id`
- **THEN** the system returns `ResourceGroupError::GroupHasReferences`

#### Scenario: Delete reference
- **GIVEN** a reference from `group-id` to `resource-type/resource-id`
- **WHEN** the user calls `delete_reference`
- **THEN** the system deletes the reference
- **AND** decrements reference count for the group

---

### Requirement: Authorization and Access Control

The system SHALL enforce application-based authorization for all operations.

Authorization rules:
- Only authenticated applications can modify resource groups
- Type owners can modify their types
- All operations require valid `SecurityCtx` for tenant isolation

#### Scenario: Authorize type creation

- **GIVEN** an authenticated application `app-uuid`
- **WHEN** the application creates a type
- **THEN** the system sets `owner_id = app-uuid` as owner
- **AND** allows the application to modify the type

---

### Requirement: REST API Endpoints

The system SHALL expose REST API endpoints for resource group operations.
Base path: `/resource-group/v1`

#### Resource Group Types
- `POST /types` - Create a new type
- `GET /types` - List all types
- `GET /types/{code}` - Get a specific type
- `PUT /types/{code}` - Update a type
- `DELETE /types/{code}` - Delete a type

#### Resource Group Entities
- `POST /groups` - Create a new entity
- `GET /groups/{id}` - Get a specific entity
- `PUT /groups/{id}` - Update an entity
- `DELETE /groups/{id}` - Delete an entity
- `GET /groups/{id}/ancestors` - Get all ancestors
- `GET /groups/{id}/descendants` - Get all descendants

#### Resource Group References

- `POST /groups/{id}/references` - Create a reference
- `DELETE /groups/{id}/references` - Delete a reference

All endpoints SHALL:
- Require authentication (Bearer token)
- Return RFC-9457 Problem Details for errors
- Follow HyperSpot REST API conventions

#### Scenario: Create type via REST

- **GIVEN** a valid JSON payload with type data
- **WHEN** POST request is sent to `/resource-group/v1/types`
- **THEN** the system creates the type and returns `201 Created` with location header

---

### Requirement: Service Configuration for Hierarchy Constraints

The service SHALL support configuring hierarchy query constraints for resource group hierarchy operations.

Configuration options:
- `max_depth` (positive integer, default `10`, MUST be `<= 10`)
- `max_width` (positive integer, optional; if provided, limits the number of children included per parent node)

The configured constraints SHALL apply to all hierarchy query operations exposed by the module (service API and REST API).

#### Scenario: Default constraints

- **GIVEN** the service is started without hierarchy constraint configuration
- **WHEN** a client requests `descendants` for an entity
- **THEN** the system applies `max_depth = 10`

#### Scenario: Custom constraints

- **GIVEN** the service is configured with `max_depth = 3` and `max_width = 50`
- **WHEN** a client requests `descendants` for an entity
- **THEN** the system returns descendants up to depth 3
- **AND** limits included children per parent node to 50

#### Scenario: Reject invalid max_depth configuration

- **GIVEN** the service is configured with `max_depth = 11`
- **WHEN** the service starts
- **THEN** the service rejects the configuration as invalid

#### Scenario: Reject create/move that would violate configured max_depth

- **GIVEN** the service is configured with `max_depth = 3`
- **AND** a hierarchy exists where an entity `a` is at depth 3 relative to the root
- **WHEN** a client attempts to create or move an entity under `a` such that its depth would become 4
- **THEN** the system rejects the operation with `ResourceGroupError::Validation` with a field-specific error for `max_depth`

#### Scenario: Reduced limits require data migration (no read truncation, write blocked)

- **GIVEN** a hierarchy exists in the database that exceeds configured limits (`max_depth` and/or `max_width`)
- **WHEN** the operator reduces the service configuration limits
- **THEN** the operator MUST implement and run a data-migration script/process to bring the stored hierarchy into compliance
- **AND** until the migration is completed, read/query operations return all stored data (no truncation)
- **AND** write operations that would create or increase a violation are rejected and require reducing depth/width (or increasing limits)

---

### Requirement: Database Schema

The system SHALL use SeaORM with the following tables:

**`resource_group_type`:**
- `code` (PK): String, unique, case-insensitive
- `parents`: JSON array of type codes
- `owner_id`: UUID (owner)
- `owner_type`: String (owner type)
- `created_at`, `updated_at`: Timestamps

**`resource_group`:**
- `id` (PK): UUIDv7
- `type_code` (FK): String → `resource_group_type.code`
- `name`: String (1-255 chars)
- `external_id`: Optional String (max 255 chars)
- `created_at`, `updated_at`: Timestamps

**`resource_group_closure`:**
- `parent_id` (FK): UUID → `resource_group.id`
- `child_id` (FK): UUID → `resource_group.id`
- `depth`: Integer (0 for self-reference)
- Composite PK: `(parent_id, child_id)`
- Indexes on `parent_id`, `child_id`, `depth`

**`resource_group_reference`:**
- `group_id` (FK): UUID → `resource_group.id`
- `resource_type`: String
- `resource_id`: String
- `application_id`: UUID
- `created_at`: Timestamp
- Composite PK / Unique Constraint on `(group_id, resource_type, resource_id, application_id)`

All tables SHALL use Secure ORM.

#### Scenario: Create database schema

- **GIVEN** SeaORM migrations are defined
- **WHEN** the module initializes
- **THEN** migrations create core tables (`type`, `group`, `closure`)
- **AND** create `reference` table
- **AND** Secure ORM scoping is configured
