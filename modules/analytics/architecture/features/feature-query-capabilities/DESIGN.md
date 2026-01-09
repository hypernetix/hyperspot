# Feature: Query Capabilities

**Status**: ⏳ NOT_STARTED  
**Feature Slug**: `feature-query-capabilities`

---

## A. Feature Context

### Overview

**Feature**: Query Capabilities

**Purpose**: OData capabilities annotations for query restrictions - defines what operations each query supports (filtering, sorting, pagination, search, etc.)

**Scope**:
- Query capabilities GTS type: `query_capabilities.v1~`
- Capabilities DB tables (FilterRestrictions, SortRestrictions, SearchRestrictions, etc.)
- OData annotations management
- Capability indexing and validation
- Query capability CRUD operations

**Out of Scope**:
- Query execution - handled by feature-query-execution
- Query definitions - handled by feature-query-definitions
- Schema definitions - handled by feature-schema-query-returns

### GTS Types

This feature **owns** the following GTS type:

**Type owned**:
- `gts://gts.hypernetix.hyperspot.ax.query_capabilities.v1~` - Query capabilities type definition

**Uses types from** (references only):
- `gts://gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.query_capabilities.v1~*` - Capability categories

References from `gts/types/`:
- [query_capabilities.v1.schema.json](../../../gts/types/hypernetix/hyperspot/ax/query_capabilities.v1.schema.json)

### OpenAPI Endpoints

From `architecture/openapi/v1/api.yaml`:
- `GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.query_capabilities.v1~')` - List/search capabilities
- `POST /api/analytics/v1/gts` - Create capability instance
- `GET /api/analytics/v1/gts/{capability-id}` - Get capability by ID
- `PUT /api/analytics/v1/gts/{capability-id}` - Update capability
- `DELETE /api/analytics/v1/gts/{capability-id}` - Soft-delete capability
- `PUT /api/analytics/v1/gts/{capability-id}/enablement` - Enable capability for tenants

### Actors

**Human Actors** (from Overall Design):
- **Query Developer** - Defines capability restrictions for queries
- **Platform Admin** - Manages capability registry and enablement
- **Data Engineer** - Configures OData capabilities

**System Actors**:
- **Capability Registry Manager** - Orchestrates capability CRUD operations
- **Capability Validator** - Validates capability configurations
- **Query Execution Engine** - Enforces capabilities at runtime

**Service Roles** (from OpenAPI):
- `analytics:queries:read` - View capabilities
- `analytics:queries:write` - Create/edit capabilities
- `analytics:queries:delete` - Delete capabilities
- `analytics:admin` - Manage capability enablement

---

## B. Actor Flows

### Flow 1: Query Developer Creates Capability Definition

**Actor**: Query Developer  
**Trigger**: Need to define OData restrictions for query  
**Goal**: Create capability definition in GTS registry

**Steps**:
1. Navigate to Capabilities → Create New
2. Enter capability metadata:
   - ID (gts.hypernetix.hyperspot.ax.query_capabilities.v1~vendor.domain._.capability_name.v1)
   - Name, description
   - Category
3. Configure filter restrictions:
   - Allowed filter functions (contains, startswith, endswith, etc.)
   - Filterable properties
   - Required filters
4. Configure sort restrictions:
   - Sortable properties
   - Ascending/descending allowed
5. Configure search restrictions:
   - Search enabled/disabled
   - Searchable properties
6. Configure pagination:
   - Top supported (max page size)
   - Skip supported
7. Configure field selection:
   - Select support enabled/disabled
   - Selectable properties
8. Configure expand:
   - Expand enabled/disabled
   - Expandable navigation properties
9. Save capability to GTS registry

**API Interaction**:
```
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.query_capabilities.v1~')

POST /api/analytics/v1/gts
Type: gts.hypernetix.hyperspot.ax.query_capabilities.v1~
Instance: Full capability configuration
```

---

### Flow 2: Query Developer Edits Capability

**Actor**: Query Developer  
**Trigger**: Capability needs update (e.g., add new filter function)  
**Goal**: Update capability configuration

**Steps**:
1. Search for capability in registry
2. Open capability for editing
3. Modify restrictions:
   - Add/remove filter functions
   - Update filterable properties
   - Change sort restrictions
   - Update search configuration
4. Validate changes
5. Save updated capability

**API Interaction**:
```
GET /api/analytics/v1/gts?$filter=contains(name,'sales')&$select=...
GET /api/analytics/v1/gts/{capability-id}
PUT /api/analytics/v1/gts/{capability-id}
```

---

### Flow 3: Platform Admin Enables Capability for Tenants

**Actor**: Platform Admin  
**Trigger**: Capability ready for tenant use  
**Goal**: Enable capability access for specific tenants

**Steps**:
1. Navigate to Capability Registry
2. Select capability to enable
3. Choose enablement strategy:
   - Enable for specific tenants
   - Enable for all tenants
4. Confirm enablement

**API Interaction**:
```
PUT /api/analytics/v1/gts/{capability-id}/enablement
Body: { "enabled_for": ["tenant-1", "tenant-2"] } or { "enabled_for": "all" }
```

---

### Flow 4: Query Execution Engine Enforces Capabilities

**Actor**: Query Execution Engine  
**Trigger**: Query execution request received  
**Goal**: Validate OData parameters against capability restrictions

**Steps**:
1. Load query definition
2. Load capability definition from query.capabilities_id
3. Validate $filter:
   - Check filter functions against allowed list
   - Verify filtered properties are allowed
4. Validate $orderby:
   - Check sorted properties are sortable
5. Validate $search:
   - Check if search is allowed
6. Validate $top/$skip:
   - Check if pagination is supported
   - Validate top value against max
7. Validate $select:
   - Check if select is supported
   - Verify selected properties are allowed
8. Validate $expand:
   - Check if expand is supported
   - Verify expanded properties are allowed
9. If validation passes, proceed with query execution
10. If validation fails, return 400 Bad Request with details

**API Interaction**:
```
Internal: Load capability from GTS registry
Internal: Validate OData params against capability
```

---

### Flow 5: Query Developer Searches Capabilities

**Actor**: Query Developer  
**Trigger**: Need to find existing capability  
**Goal**: Search and browse capability registry

**Steps**:
1. Navigate to Capabilities
2. Apply filters:
   - By category
   - By name/description
   - By restriction type
3. Browse results
4. Select capability for details

**API Interaction**:
```
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.query_capabilities.v1~') and category_id eq 'sales'&$search=odata&$select=id,name,description,category_id&$orderby=name
GET /api/analytics/v1/gts/{capability-id}
```

---

## C. Algorithms

### Service Algorithm 1: Validate OData Parameters Against Capabilities

**Purpose**: Enforce capability restrictions at query execution time

Input: odata_params (filter, orderby, search, top, skip, select, expand), capabilities  
Output: validation result or error

1. Validate filter parameter:
   1. **IF** odata_params has filter:
      1. Parse filter expression
      2. Extract filter functions used
      3. **FOR EACH** function in filter functions:
         1. Check function is in capabilities.filter_functions
         2. **IF** function not allowed:
            1. **RETURN** error: "Filter function not supported"
      4. Extract filtered properties
      5. **FOR EACH** property in filtered properties:
         1. Check property is in capabilities.filterable_properties
         2. **IF** property not filterable:
            1. **RETURN** error: "Property not filterable"
2. Validate orderby parameter:
   1. **IF** odata_params has orderby:
      1. Parse orderby expression
      2. Extract sorted properties
      3. **FOR EACH** property in sorted properties:
         1. Check property is in capabilities.sortable_properties
         2. **IF** property not sortable:
            1. **RETURN** error: "Property not sortable"
3. Validate search parameter:
   1. **IF** odata_params has search:
      1. Check capabilities.search_restrictions.searchable is true
      2. **IF** search not allowed:
         1. **RETURN** error: "Search not supported"
4. Validate pagination:
   1. **IF** odata_params has top:
      1. Check capabilities.top_supported is true
      2. Check top value does not exceed capabilities.max_top
      3. **IF** validation fails:
         1. **RETURN** error: "Top value exceeds maximum"
   2. **IF** odata_params has skip:
      1. Check capabilities.skip_supported is true
      2. **IF** skip not supported:
         1. **RETURN** error: "Skip not supported"
5. Validate select parameter:
   1. **IF** odata_params has select:
      1. Check capabilities.select_support is true
      2. **IF** select not supported:
         1. **RETURN** error: "Select not supported"
      3. **FOR EACH** field in selected fields:
         1. Check field is in capabilities.selectable_properties
         2. **IF** field not selectable:
            1. **RETURN** error: "Property not selectable"
6. Validate expand parameter:
   1. **IF** odata_params has expand:
      1. Check capabilities.expand_support is true
      2. **IF** expand not supported:
         1. **RETURN** error: "Expand not supported"
      3. **FOR EACH** property in expanded properties:
         1. Check property is in capabilities.expandable_properties
         2. **IF** property not expandable:
            1. **RETURN** error: "Property not expandable"
7. **RETURN** success

---

### Service Algorithm 2: Merge Default and User-Provided Capabilities

**Purpose**: Combine platform defaults with custom capability restrictions

Input: default_capabilities, custom_restrictions  
Output: merged capabilities

1. Start with default_capabilities as base
2. **IF** custom_restrictions has filter_functions:
   1. Intersect with default filter functions (more restrictive)
3. **IF** custom_restrictions has filterable_properties:
   1. Intersect with default filterable properties
4. **IF** custom_restrictions has sortable_properties:
   1. Intersect with default sortable properties
5. **IF** custom_restrictions has search_restrictions:
   1. Use custom search restrictions
6. **IF** custom_restrictions has top_supported:
   1. Use custom top_supported
   2. Use minimum of default and custom max_top
7. **IF** custom_restrictions has skip_supported:
   1. Use custom skip_supported
8. **IF** custom_restrictions has select_support:
   1. Use custom select_support
   2. Intersect selectable properties if provided
9. **IF** custom_restrictions has expand_support:
   1. Use custom expand_support
   2. Intersect expandable properties if provided
10. **RETURN** merged capabilities

---

## D. States

Capability lifecycle state machine:

```
[NOT_CREATED] --create--> [DRAFT]
[DRAFT] --validate--> [VALID]
[VALID] --enable--> [ENABLED]
[ENABLED] --disable--> [DISABLED]
[DISABLED] --enable--> [ENABLED]
[ENABLED/DISABLED] --delete--> [DELETED]
```

**State Transitions**:
- **DRAFT → VALID**: All restrictions validated
- **VALID → ENABLED**: Enablement API called for at least one tenant
- **ENABLED → DISABLED**: Disabled for all tenants
- **DISABLED → ENABLED**: Re-enabled for tenants
- **DELETED**: Soft-deleted (deleted_at timestamp set)

---

## E. Technical Details

### High-Level DB Schema

**Tables**:

**gts_instances** (via GTS registry):
- id (PK): `gts.hypernetix.hyperspot.ax.query_capabilities.v1~vendor.domain._.capability_name.v1`
- type: `gts.hypernetix.hyperspot.ax.query_capabilities.v1~`
- entity (JSONB): Capability configuration
  - name: String
  - description: String
  - category_id: String (FK to category)
  - filter_functions: Array<String> (contains, startswith, endswith, eq, ne, gt, lt, ge, le, and, or, not)
  - filterable_properties: Array<String>
  - required_filters: Array<String>
  - sortable_properties: Array<String>
  - sort_ascending_allowed: Boolean
  - sort_descending_allowed: Boolean
  - search_restrictions:
    - searchable: Boolean
    - searchable_properties: Array<String>
  - top_supported: Boolean
  - max_top: Integer
  - skip_supported: Boolean
  - select_support: Boolean
  - selectable_properties: Array<String>
  - expand_support: Boolean
  - expandable_properties: Array<String>
- created_by: UserID
- created_at: Timestamp
- updated_at: Timestamp
- deleted_at: Timestamp (nullable, for soft-delete)

**gts_enablement** (via GTS registry):
- instance_id (FK to gts_instances.id)
- tenant_id (FK to tenants)
- enabled_at: Timestamp

**Indexes**:
- `idx_capabilities_type` on (type) WHERE type = 'gts.hypernetix.hyperspot.ax.query_capabilities.v1~'
- `idx_capabilities_category` on ((entity->>'category_id'))
- `idx_capabilities_deleted` on (deleted_at) WHERE deleted_at IS NULL
- `idx_capabilities_tenant` on (instance_id, tenant_id) in gts_enablement

---

### Database Operations

**Create Capability**:
```
INSERT INTO gts_instances (id, type, entity, created_by, created_at, updated_at)
VALUES ($1, 'gts.hypernetix.hyperspot.ax.query_capabilities.v1~', $2, $3, NOW(), NOW())
```

**Search Capabilities**:
```
SELECT id, type, entity, created_at, updated_at
FROM gts_instances
WHERE type = 'gts.hypernetix.hyperspot.ax.query_capabilities.v1~'
  AND deleted_at IS NULL
  AND (entity->>'category_id' = $1 OR $1 IS NULL)
  AND (entity->>'name' ILIKE $2 OR $2 IS NULL)
ORDER BY entity->>'name'
LIMIT $3 OFFSET $4
```

**Enable Capability for Tenant**:
```
INSERT INTO gts_enablement (instance_id, tenant_id, enabled_at)
VALUES ($1, $2, NOW())
ON CONFLICT (instance_id, tenant_id) DO NOTHING
```

**Get Capability with Enablement**:
```
SELECT i.id, i.type, i.entity, i.created_at, i.updated_at,
       array_agg(e.tenant_id) as enabled_tenants
FROM gts_instances i
LEFT JOIN gts_enablement e ON i.id = e.instance_id
WHERE i.id = $1 AND i.deleted_at IS NULL
GROUP BY i.id
```

---

### Access Control

**SecurityCtx Enforcement**:
- All capability operations require authenticated user
- Tenant isolation enforced on all queries
- Capability ownership via `created_by` field
- Admin role required for enablement operations

**Permission Checks**:
- **Capability creation**: Requires `analytics:queries:write` permission
- **Capability modification**: Requires `analytics:queries:write` permission AND (user is capability creator OR user is admin)
- **Capability enablement**: Requires `analytics:admin` permission
- **Capability deletion**: Requires `analytics:queries:delete` permission AND (user is capability creator OR user is admin)

---

### Error Handling

**Common Errors**:
- **404 Not Found**: Capability not found or soft-deleted
- **400 Bad Request**: Invalid capability configuration
- **403 Forbidden**: Insufficient permissions
- **422 Unprocessable Entity**: Capability validation failure
- **409 Conflict**: Capability ID already exists

**Runtime Validation Errors** (when enforcing capabilities):
- **400 Bad Request**: OData parameter violates capability restrictions
  - "Filter function 'substringof' not supported"
  - "Property 'salary' is not filterable"
  - "Property 'password' is not sortable"
  - "Search not supported for this query"
  - "Top value 1000 exceeds maximum 500"
  - "Skip not supported for this query"
  - "Select not supported for this query"
  - "Property 'internal_notes' is not selectable"
  - "Expand not supported for this query"

**Error Response Format (RFC 7807)**:
```json
{
  "type": "https://example.com/problems/odata-capability-violation",
  "title": "OData Capability Violation",
  "status": 400,
  "detail": "Filter function 'substringof' is not supported by this query",
  "instance": "/api/analytics/v1/queries/revenue-by-region"
}
```

---

## F. Validation & Implementation

### Testing Scenarios

**Unit Tests**:
- Capability configuration validation
- OData parameter validation against capabilities
- Filter function checks
- Property filterability checks
- Sort restrictions enforcement
- Search restrictions enforcement
- Pagination limits enforcement
- Select/expand support checks

**Integration Tests**:
- Capability CRUD via GTS API
- Capability search with filters
- Enablement operations
- Soft-delete and recovery
- Tenant isolation
- Runtime capability enforcement

**Edge Cases**:
1. Capability with empty filter functions (nothing allowed)
2. Capability with no filterable properties
3. Capability with conflicting restrictions
4. OData query with multiple violations
5. Concurrent capability updates
6. Capability in use by queries (prevent deletion)

**Performance Tests**:
- Capability search with 10,000+ capabilities
- OData validation with complex filter expressions
- Concurrent validation requests

---

### OpenSpec Changes Plan

**Total Changes**: 6  
**Estimated Effort**: 33 hours (with AI agent)

#### Change 001: Query Capabilities GTS Type Definition

**Status**: ⏳ NOT_STARTED

**Scope**: Define query capabilities GTS type schema

**Tasks**:
- [ ] Create JSON schema for query_capabilities.v1~
- [ ] Define all OData restriction properties
- [ ] Add validation rules for capability config

**Files**:
- `gts/types/hypernetix/hyperspot/ax/query_capabilities.v1.schema.json`

**Dependencies**: None (foundational)

**Effort**: 4 hours (AI agent)

---

#### Change 002: Capability Repository

**Status**: ⏳ NOT_STARTED

**Scope**: Capability persistence layer with search

**Tasks**:
- [ ] Create repository trait
- [ ] Implement PostgreSQL repository
- [ ] Add capability search with filters
- [ ] Add indexes for performance

**Files**:
- `modules/analytics/src/domain/query_capabilities/repository.rs`
- `modules/analytics/src/infra/storage/query_capabilities/pg_repository.rs`

**Dependencies**: Change 001

**Effort**: 6 hours (AI agent)

---

#### Change 003: Capability Service

**Status**: ⏳ NOT_STARTED

**Scope**: Capability business logic and validation

**Tasks**:
- [ ] Create capability service
- [ ] Implement capability CRUD
- [ ] Add capability validator
- [ ] Add OData parameter validator

**Files**:
- `modules/analytics/src/domain/query_capabilities/service.rs`
- `modules/analytics/src/domain/query_capabilities/validator.rs`

**Dependencies**: Change 002

**Effort**: 8 hours (AI agent)

---

#### Change 004: Capability API Handlers

**Status**: ⏳ NOT_STARTED

**Scope**: REST API for capability CRUD

**Tasks**:
- [ ] Create API handlers
- [ ] Create DTOs for requests/responses
- [ ] Add OpenAPI documentation
- [ ] Wire up routes

**Files**:
- `modules/analytics/src/api/rest/query_capabilities/handlers.rs`
- `modules/analytics/src/api/rest/query_capabilities/dto.rs`

**Dependencies**: Change 003

**Effort**: 5 hours (AI agent)

---

#### Change 005: Capability Enablement Service

**Status**: ⏳ NOT_STARTED

**Scope**: Tenant enablement operations

**Tasks**:
- [ ] Create enablement service
- [ ] Implement tenant access control
- [ ] Add bulk enablement

**Files**:
- `modules/analytics/src/domain/query_capabilities/enablement.rs`

**Dependencies**: Change 003

**Effort**: 4 hours (AI agent)

---

#### Change 006: Integration Testing Suite

**Status**: ⏳ NOT_STARTED

**Scope**: E2E capability workflow tests

**Tasks**:
- [ ] Create integration tests
- [ ] Test capability CRUD
- [ ] Test OData validation
- [ ] Test enablement workflows

**Files**:
- `modules/analytics/tests/query_capabilities.rs`

**Dependencies**: All previous changes

**Effort**: 6 hours (AI agent)

---

## Dependencies

- **Depends On**: 
  - [feature-gts-core](../feature-gts-core/) (GTS unified API)
- **Blocks**: 
  - [feature-query-definitions](../feature-query-definitions/) (queries reference capabilities)
  - [feature-query-execution](../feature-query-execution/) (runtime enforcement)

---

## References

- Overall Design: `architecture/DESIGN.md` Section 2 (Actors), Section 3 (System Capabilities)
- GTS Types: [query_capabilities.v1.schema.json](../../../gts/types/hypernetix/hyperspot/ax/query_capabilities.v1.schema.json)
- OpenAPI Spec: `architecture/openapi/v1/api.yaml` (capability endpoints)
- Feature Manifest: `architecture/features/FEATURES.md` (feature-query-capabilities entry)
- OData Spec: [OData v4.01 Part 2: Protocol](https://docs.oasis-open.org/odata/odata/v4.01/odata-v4.01-part2-protocol.html)
