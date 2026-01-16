## ADDED Requirements

### Requirement: Resource Group Type Management

The system SHALL provide an API to manage resource group types that define the schema and allowed parent-child relationships for resource groups.

A resource group type consists of:
- **Code**: Unique identifier (1-63 chars, no whitespace, case-insensitive)
- **Parents**: Array of type codes that can be parents of this type
- **Application ID**: Owner application that created the type
- **Allowed App IDs**: Applications allowed to create/modify groups of this type

Type management SHALL support:
- Creating new types with validation
- Listing all types
- Retrieving a specific type by code
- Updating type properties (parents, allowed apps)
- Deleting types (only if no entities of this type exist)

#### Scenario: Create a new resource group type

- **GIVEN** a valid type code `DEPARTMENT` and allowed parent types `["ORGANIZATION", "DIVISION"]`
- **AND** an authenticated application with ID `app-uuid`
- **WHEN** the user calls `create_type` with type data
- **THEN** the system creates the type with the application as owner
- **AND** returns the created type with `application_id` set

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
- **THEN** the system returns all descendants `[DEPT1, DEPT2, TEAM1]` ordered by depth

---

### Requirement: Reference Management

The system SHALL provide an API to link resource groups to external resources.

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
- Applications in `allowed_app_ids` can create/modify entities of that type
- All operations require valid `SecurityCtx` for tenant isolation

#### Scenario: Authorize type creation

- **GIVEN** an authenticated application `app-uuid`
- **WHEN** the application creates a type
- **THEN** the system sets `application_id = app-uuid` as owner
- **AND** allows the application to modify the type

#### Scenario: Reject unauthorized entity creation

- **GIVEN** a type `DEPARTMENT` with `allowed_app_ids = [app1-uuid]`
- **AND** an authenticated application `app2-uuid` not in allowed list
- **WHEN** `app2-uuid` attempts to create an entity of type `DEPARTMENT`
- **THEN** the system returns `ResourceGroupError::Unauthorized`

---

### Requirement: REST API Endpoints

The system SHALL expose REST API endpoints for resource group operations.

Endpoints SHALL include:
- `POST /resource-group/v1/types` — Create type
- `GET /resource-group/v1/types` — List types
- `GET /resource-group/v1/types/{code}` — Get type
- `PUT /resource-group/v1/types/{code}` — Update type
- `DELETE /resource-group/v1/types/{code}` — Delete type
- `POST /resource-group/v1/groups` — Create entity
- `GET /resource-group/v1/groups/{id}` — Get entity
- `PUT /resource-group/v1/groups/{id}` — Update entity
- `DELETE /resource-group/v1/groups/{id}` — Delete entity
- `GET /resource-group/v1/groups/{id}/ancestors` — Get ancestors
- `GET /resource-group/v1/groups/{id}/descendants` — Get descendants
- `POST /resource-group/v1/groups/{id}/references` — Create reference
- `DELETE /resource-group/v1/groups/{id}/references` — Delete reference

All endpoints SHALL:
- Require authentication (Bearer token)
- Return RFC-9457 Problem Details for errors
- Follow HyperSpot REST API conventions

#### Scenario: Create type via REST

- **GIVEN** a valid JSON payload with type data
- **WHEN** POST request is sent to `/resource-group/v1/types`
- **THEN** the system creates the type and returns `201 Created` with location header

#### Scenario: List entities via REST

- **GIVEN** registered entities in the system
- **WHEN** GET request is sent to `/resource-group/v1/groups`
- **THEN** response contains entities

---

### Requirement: Database Schema

The system SHALL use SeaORM with the following tables:

**`resource_group_type`:**
- `code` (PK): String, unique, case-insensitive
- `parents`: JSON array of type codes
- `application_id`: UUID (owner)
- `allowed_app_ids`: JSON array of UUIDs
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
- `id` (PK): UUIDv7
- `group_id` (FK): UUID → `resource_group.id`
- `resource_type`: String
- `resource_id`: String
- `application_id`: UUID
- `created_at`: Timestamp
- Composite index on `(resource_type, resource_id)`

All tables SHALL use Secure ORM.

#### Scenario: Create database schema

- **GIVEN** SeaORM migrations are defined
- **WHEN** the module initializes
- **THEN** migrations create all tables with proper indexes
- **AND** Secure ORM scoping is configured
