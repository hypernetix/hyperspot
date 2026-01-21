# Feature Design: Resource Group

**Feature ID**: `fdd-hyperspot-feature-resource-group`
**Status**: IN_DESIGN

## Section A: Feature Context

### Overview
The Resource Group module provides hierarchical resource organization with type-based access control. It enables applications to organize resources in a tree structure with strict type validation, permission management, and efficient hierarchy operations using the Closure Table pattern.

### Purpose
To allow applications to model complex organizational structures (e.g., Organization -> Department -> Team) and enforce permissions and constraints based on these structures.

### Actors
- `fdd-hyperspot-actor-application` - Authenticated application managing resource groups
- `fdd-hyperspot-actor-system-admin` - Administrator managing resource group types

### References
- [Overall Design](../../../../../../docs/ARCHITECTURE_MANIFEST.md)
- [ADR-0001: Closure Table](../ADR.md)

---

## Section B: Actor Flows

### Create Resource Group Type

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-flow-create-type`

1. [ ] - `ph-1` - **Actor** provides type code, allowed parents, and allowed app IDs - `inst-provide-data`
2. [ ] - `ph-1` - **System** validates type code format (no whitespace, length limit) - `inst-validate-format`
3. [ ] - `ph-1` - **System** checks if type code already exists - `inst-check-duplicate`
4. [ ] - `ph-1` - **IF** type exists:
   - [ ] - `ph-1` - **System** returns error `TypeAlreadyExists` - `inst-err-duplicate`
   - [ ] - `ph-1` - **RETURN** - `inst-ret-duplicate`
5. [ ] - `ph-1` - **System** creates new type record with current app as owner - `inst-create-record`
6. [ ] - `ph-1` - **System** returns created type - `inst-return-success`

### Create Resource Group Entity

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-flow-create-entity`

1. [ ] - `ph-1` - **Actor** provides name, type code, and optional parent ID - `inst-provide-data`
2. [ ] - `ph-1` - **System** validates actor has permission to create entity of this type - `inst-validate-perm`
3. [ ] - `ph-1` - **IF** parent ID is provided:
   - [ ] - `ph-1` - **System** fetches parent entity - `inst-fetch-parent`
   - [ ] - `ph-1` - **System** validates parent type is allowed for this entity type - `inst-validate-parent-type`
   - [ ] - `ph-1` - **System** checks hierarchy depth constraints (`max_depth`) - `inst-check-depth`
   - [ ] - `ph-1` - **System** checks hierarchy width constraints (`max_width`) - `inst-check-width`
4. [ ] - `ph-1` - **System** creates entity record - `inst-create-entity`
5. [ ] - `ph-1` - **System** inserts closure table entries (self + ancestors) - `inst-insert-closure`
6. [ ] - `ph-1` - **System** returns created entity - `inst-return-success`

### Move Entity Subtree

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-flow-move-entity`

1. [ ] - `ph-2` - **Actor** requests to move entity `node-id` to `new-parent-id` - `inst-req-move`
2. [ ] - `ph-2` - **System** validates `new-parent-id` exists and is valid parent type - `inst-validate-parent`
3. [ ] - `ph-2` - **System** checks for cycles (is `new-parent-id` a descendant of `node-id`?) - `inst-check-cycle`
4. [ ] - `ph-2` - **IF** cycle detected:
   - [ ] - `ph-2` - **System** returns error `CycleDetected` - `inst-err-cycle`
   - [ ] - `ph-2` - **RETURN** - `inst-ret-cycle`
5. [ ] - `ph-2` - **System** validates new depth constraints for entire subtree - `inst-validate-depth`
6. [ ] - `ph-2` - **System** updates closure table (delete old paths, insert new paths) - `inst-update-closure`
7. [ ] - `ph-2` - **System** updates `parent_id` on entity - `inst-update-entity`
8. [ ] - `ph-2` - **System** returns success - `inst-return-success`

### Manage References

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-flow-manage-refs`

1. [ ] - `ph-3` - **Actor** links group to external resource - `inst-link-ref`
2. [ ] - `ph-3` - **System** stores reference and increments counter - `inst-store-ref`
3. [ ] - **IF** **Actor** tries to delete group with refs:
   - [ ] - `ph-3` - **System** returns error `GroupHasReferences` - `inst-err-refs`

---

## Section C: Algorithms

### Closure Table Insert

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-algo-closure-insert`

**Input**: `node_id`, `parent_id`

1. [ ] - `ph-1` - **System** inserts self-reference: `(node_id, node_id, 0)` - `inst-insert-self`
2. [ ] - `ph-1` - **IF** `parent_id` is NOT NULL:
   - [ ] - `ph-1` - **System** selects all ancestors of `parent_id` (including `parent_id` itself) - `inst-select-ancestors`
   - [ ] - `ph-1` - **FOR EACH** ancestor `a` in ancestors:
     - [ ] - `ph-1` - **System** insert `(a.parent_id, node_id, a.depth + 1)` - `inst-insert-path`

### Cycle Detection

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-algo-cycle-detection`

**Input**: `node_id`, `target_parent_id`

1. [ ] - `ph-2` - **System** queries closure table for path from `node_id` to `target_parent_id` - `inst-query-path`
2. [ ] - `ph-2` - **IF** path exists (meaning `target_parent_id` is descendant of `node_id`):
   - [ ] - `ph-2` - **RETURN** `true` (Cycle Detected) - `inst-ret-true`
3. [ ] - `ph-2` - **RETURN** `false` - `inst-ret-false`

---

## Section D: States

### Entity Lifecycle

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-state-entity`

1. [ ] - `ph-1` - **WHEN** entity created -> **Active** - `inst-state-active`
2. [ ] - `ph-1` - **WHEN** entity deleted -> **Deleted** (removed from DB) - `inst-state-deleted`

---

## Section E: Technical Details

### Database Schema

**Tables**:
1. `resource_group_type`
   - `code` (PK): String
   - `parents`: JSON
   - `application_id`: UUID
   - `allowed_app_ids`: JSON
2. `resource_group`
   - `id` (PK): UUIDv7
   - `type_code` (FK): String
   - `parent_id`: UUID (nullable)
   - `name`: String
3. `resource_group_closure`
   - `parent_id` (FK): UUID
   - `child_id` (FK): UUID
   - `depth`: Integer
   - PK: `(parent_id, child_id)`

**References**:
- [Secure ORM](../../../../../../docs/SECURE-ORM.md)

### API Endpoints

- `POST /resource-group/v1/types`
- `GET /resource-group/v1/types`
- `POST /resource-group/v1/groups`
- `GET /resource-group/v1/groups/{id}`
- `GET /resource-group/v1/groups/{id}/ancestors`
- `GET /resource-group/v1/groups/{id}/descendants`

**References**:
- [API Specification](../../../../openspec/changes/add-resource-group-module/specs/resource-group/spec.md)

### Security
- All operations require `SecurityCtx`.
- Tenant isolation enforced via Secure ORM.
- Application ownership checks for Types.

---

## Section F: Requirements

### Type Management

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-req-type-mgmt`

### Type Management
**Status**: ⏳ NOT_STARTED
**Description**: The system SHALL provide an API to manage resource group types that define the schema and allowed parent-child relationships.
**Phases**:
- [ ] `ph-1`: Create and list types
- [ ] `ph-1`: Validate type codes
**Tests Covered**:
- `fdd-hyperspot-feature-resource-group-test-create-type`
- `fdd-hyperspot-feature-resource-group-test-duplicate-type`
- `fdd-hyperspot-feature-resource-group-test-invalid-type`
**Acceptance Criteria**:
- Verify that creating a new resource group type with valid data succeeds and sets the application owner.
- Verify that attempting to create a type with a duplicate code returns `TypeAlreadyExists` error.
- Verify that type codes with whitespace or invalid length are rejected.

### Entity Management

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-req-entity-mgmt`

### Entity Management
**Status**: ⏳ NOT_STARTED
**Description**: The system SHALL provide an API to manage resource group entities organized in hierarchical structures.
**Phases**:
- [ ] `ph-1`: Create entity with parent validation
- [ ] `ph-2`: Move entity subtree
**Tests Covered**:
- `fdd-hyperspot-feature-resource-group-test-create-entity`
- `fdd-hyperspot-feature-resource-group-test-move-entity`
- `fdd-hyperspot-feature-resource-group-test-invalid-parent`
- `fdd-hyperspot-feature-resource-group-test-cycle-detection`
**Acceptance Criteria**:
- Verify that creating an entity with a valid parent and type compatibility succeeds.
- Verify that creating an entity with an incompatible parent type returns `InvalidParentType` error.
- Verify that moving an entity subtree to a new valid parent succeeds and updates closure table.
- Verify that attempting to move an entity to one of its descendants (cycle) returns `CycleDetected` error.

### Hierarchy Operations

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-req-hierarchy-ops`

### Hierarchy Operations
**Status**: ⏳ NOT_STARTED
**Description**: The system SHALL provide efficient hierarchy queries using the Closure Table pattern, respecting configured depth/width limits.
**Phases**:
- [ ] `ph-1`: Query ancestors/descendants
- [ ] `ph-1`: Enforce max_depth default (10)
- [ ] `ph-2`: Enforce configured max_depth/max_width
**Tests Covered**:
- `fdd-hyperspot-feature-resource-group-test-hierarchy`
- `fdd-hyperspot-feature-resource-group-test-hierarchy-constraints`
**Acceptance Criteria**:
- Verify that querying ancestors returns the correct path ordered by depth.
- Verify that querying descendants returns all children ordered by depth.
- Verify that hierarchy queries respect the configured `max_depth` and `max_width`.
- Verify that existing data exceeding new stricter limits is returned without truncation during reads.

### References

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-req-refs`

### References
**Status**: ⏳ NOT_STARTED
**Description**: The system SHALL provide functionality to link resource groups to external resources and prevent deletion of referenced groups.
**Phases**:
- [ ] `ph-3`: Link/Unlink references
- [ ] `ph-3`: Prevent deletion if references exist
**Tests Covered**:
- `fdd-hyperspot-feature-resource-group-test-refs`
**Acceptance Criteria**:
- Verify that references can be created.
- Verify that deleting a group with active references is prevented.
- Verify that references can be deleted and the group becomes deletable.

---

## Section G: Testing Scenarios

### Test: Create Type

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-test-create-type`

### Create Type
**Validates**: `fdd-hyperspot-feature-resource-group-req-type-mgmt`

1. [ ] - `ph-1` - **Actor** calls create_type with code "DEPT" - `inst-call-api`
2. [ ] - `ph-1` - **System** creates type - `inst-create`
3. [ ] - `ph-1` - **Actor** verifies response contains created type - `inst-verify`

### Test: Duplicate Type

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-test-duplicate-type`

### Duplicate Type
**Validates**: `fdd-hyperspot-feature-resource-group-req-type-mgmt`

1. [ ] - `ph-1` - **Actor** creates type with code "DEPT" - `inst-create-first`
2. [ ] - `ph-1` - **Actor** attempts to create type with code "DEPT" again - `inst-create-dup`
3. [ ] - `ph-1` - **System** returns error `TypeAlreadyExists` - `inst-verify-err`

### Test: Invalid Type Format

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-test-invalid-type`

### Invalid Type Format
**Validates**: `fdd-hyperspot-feature-resource-group-req-type-mgmt`

1. [ ] - `ph-1` - **Actor** attempts to create type with code "DEP ARTMENT" (whitespace) - `inst-create-invalid`
2. [ ] - `ph-1` - **System** returns validation error - `inst-verify-err`

### Test: Create Entity with Parent

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-test-create-entity`

### Create Entity with Parent
**Validates**: `fdd-hyperspot-feature-resource-group-req-entity-mgmt`

1. [ ] - `ph-1` - **Actor** creates parent entity "ORG" - `inst-create-parent`
2. [ ] - `ph-1` - **Actor** creates child entity "TEAM" with parent "ORG" - `inst-create-child`
3. [ ] - `ph-1` - **System** verifies closure table has path ORG->TEAM - `inst-verify-closure`

### Test: Invalid Parent Type

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-test-invalid-parent`

### Invalid Parent Type
**Validates**: `fdd-hyperspot-feature-resource-group-req-entity-mgmt`

1. [ ] - `ph-1` - **Actor** creates entity "ORG" of type "ORGANIZATION" - `inst-create-org`
2. [ ] - `ph-1` - **Actor** attempts to create "DEPT" with parent "ORG" where type "ORGANIZATION" is NOT allowed parent - `inst-create-invalid`
3. [ ] - `ph-1` - **System** returns error `InvalidParentType` - `inst-verify-err`

### Test: Move Entity Subtree

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-test-move-entity`

### Move Entity Subtree
**Validates**: `fdd-hyperspot-feature-resource-group-req-entity-mgmt`

1. [ ] - `ph-2` - **Actor** creates hierarchy A->B - `inst-create-ab`
2. [ ] - `ph-2` - **Actor** creates entity C - `inst-create-c`
3. [ ] - `ph-2` - **Actor** moves B to be child of C - `inst-move-b`
4. [ ] - `ph-2` - **System** verifies closure table has path C->B - `inst-verify-move`
5. [ ] - `ph-2` - **System** verifies path A->B is removed - `inst-verify-cleanup`

### Test: Cycle Detection

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-test-cycle-detection`

### Cycle Detection
**Validates**: `fdd-hyperspot-feature-resource-group-req-entity-mgmt`

1. [ ] - `ph-2` - **Actor** creates hierarchy A->B - `inst-create-ab`
2. [ ] - `ph-2` - **Actor** attempts to move A to be child of B - `inst-move-cycle`
3. [ ] - `ph-2` - **System** returns error `CycleDetected` - `inst-verify-err`

### Test: Hierarchy Queries

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-test-hierarchy`

### Hierarchy Queries
**Validates**: `fdd-hyperspot-feature-resource-group-req-hierarchy-ops`

1. [ ] - `ph-1` - **Actor** creates chain A->B->C - `inst-create-chain`
2. [ ] - `ph-1` - **Actor** requests descendants of A - `inst-req-desc`
3. [ ] - `ph-1` - **System** returns [B, C] - `inst-verify-res`
4. [ ] - `ph-1` - **Actor** requests ancestors of C - `inst-req-anc`
5. [ ] - `ph-1` - **System** returns [A, B] - `inst-verify-anc`

### Test: Hierarchy Constraints

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-test-hierarchy-constraints`

### Hierarchy Constraints
**Validates**: `fdd-hyperspot-feature-resource-group-req-hierarchy-ops`

1. [ ] - `ph-2` - **System** is configured with `max_depth=1` - `inst-config`
2. [ ] - `ph-2` - **Actor** creates chain A->B - `inst-create-valid`
3. [ ] - `ph-2` - **Actor** attempts to create child C under B (depth 2) - `inst-create-deep`
4. [ ] - `ph-2` - **System** returns validation error for max depth - `inst-verify-err`

### Test: References

- [ ] **ID**: `fdd-hyperspot-feature-resource-group-test-refs`

### References
**Validates**: `fdd-hyperspot-feature-resource-group-req-refs`

1. [ ] - `ph-3` - **Actor** creates reference from Group A to Resource X - `inst-create-ref`
2. [ ] - `ph-3` - **Actor** attempts to delete Group A - `inst-delete-fail`
3. [ ] - `ph-3` - **System** returns error `GroupHasReferences` - `inst-verify-err`
4. [ ] - `ph-3` - **Actor** deletes reference - `inst-delete-ref`
5. [ ] - `ph-3` - **Actor** deletes Group A - `inst-delete-success`
6. [ ] - `ph-3` - **System** successfully deletes group - `inst-verify-delete`
