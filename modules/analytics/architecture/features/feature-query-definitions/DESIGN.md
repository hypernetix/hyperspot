# Feature: Query Definitions

**Status**: ⏳ NOT_STARTED  
**Feature Slug**: `feature-query-definitions`

---

## A. Feature Context

### Overview

**Feature**: Query Definitions

**Purpose**: Query type registration and metadata management - the core query type that represents data access definitions with their schemas, capabilities, and default parameters.

**Scope**:
- Query GTS type: `query.v1~` (main query type)
- Query definition DB tables
- Query metadata (category, returns_schema_id, capabilities_id)
- Query registration API
- Custom search for queries
- Query CRUD operations
- Query versioning and lifecycle

**Out of Scope**:
- Query execution - handled by feature-query-execution
- Query capabilities definitions - handled by feature-query-capabilities
- Query values definitions - handled by feature-query-values
- Schema definitions - handled by feature-schema-query-returns

### GTS Types

This feature **owns** the following GTS type:

**Type owned**:
- `gts://gts.hypernetix.hyperspot.ax.query.v1~` - Query type definition

**Uses types from** (references only):
- `gts://gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~*` - Query result schemas
- `gts://gts.hypernetix.hyperspot.ax.query_capabilities.v1~*` - Query OData capabilities
- `gts://gts.hypernetix.hyperspot.ax.query.v1~hypernetix.hyperspot.ax.values.v1~*` - Query default values
- `gts://gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.query.v1~*` - Query categories

References from `gts/types/`:
- [query.v1.schema.json](../../../gts/types/hypernetix/hyperspot/ax/query.v1.schema.json)

### OpenAPI Endpoints

From `architecture/openapi/v1/api.yaml`:
- `GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.query.v1~')` - List/search queries
- `POST /api/analytics/v1/gts` - Create query instance
- `GET /api/analytics/v1/gts/{query-id}` - Get query by ID
- `PUT /api/analytics/v1/gts/{query-id}` - Update query
- `DELETE /api/analytics/v1/gts/{query-id}` - Soft-delete query
- `PUT /api/analytics/v1/gts/{query-id}/enablement` - Enable query for tenants

### Actors

**Human Actors** (from Overall Design):
- **Query Developer** - Creates and maintains query definitions
- **Platform Admin** - Manages query registry and enablement
- **Data Engineer** - Configures queries for data access

**System Actors**:
- **Query Registry Manager** - Orchestrates query CRUD operations
- **Query Validator** - Validates query metadata against schemas
- **Query Enablement Manager** - Manages tenant access to queries

**Service Roles** (from OpenAPI):
- `analytics:queries:read` - View queries
- `analytics:queries:write` - Create/edit queries
- `analytics:queries:delete` - Delete queries
- `analytics:admin` - Manage query enablement

---

## B. Actor Flows

### Flow 1: Query Developer Creates New Query

**Actor**: Query Developer  
**Trigger**: Need new data access query  
**Goal**: Register query in GTS registry

**Steps**:
1. Navigate to Queries → Create New
2. Enter query metadata:
   - ID (gts.hypernetix.hyperspot.ax.query.v1~vendor.domain._.query_name.v1)
   - Name, description
   - Category
3. Select returns_schema_id (query result structure)
4. Select capabilities_id (OData restrictions)
5. Optionally set default values_id (default $filter, $orderby, etc.)
6. Configure query-specific properties
7. Save query to GTS registry

**API Interaction**:
```
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns')
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.query_capabilities.v1~')
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.query.v1~hypernetix.hyperspot.ax.values.v1~')
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.query.v1~')

POST /api/analytics/v1/gts
Type: gts.hypernetix.hyperspot.ax.query.v1~
Instance: gts.hypernetix.hyperspot.ax.query.v1~acme.sales._.revenue_by_region.v1
```

---

### Flow 2: Query Developer Edits Query Metadata

**Actor**: Query Developer  
**Trigger**: Query needs metadata update  
**Goal**: Update query configuration

**Steps**:
1. Search for query in registry
2. Open query for editing
3. Modify metadata:
   - Update description
   - Change category
   - Update returns_schema_id if schema evolved
   - Update capabilities_id if restrictions changed
   - Update default values_id
4. Validate changes
5. Save updated query

**API Interaction**:
```
GET /api/analytics/v1/gts?$filter=contains(name,'revenue')&$select=...
GET /api/analytics/v1/gts/{query-id}
PUT /api/analytics/v1/gts/{query-id}
```

---

### Flow 3: Platform Admin Enables Query for Tenants

**Actor**: Platform Admin  
**Trigger**: Query ready for tenant use  
**Goal**: Enable query access for specific tenants

**Steps**:
1. Navigate to Query Registry
2. Select query to enable
3. Choose enablement strategy:
   - Enable for specific tenants
   - Enable for all tenants
4. Confirm enablement
5. System automatically enables dependencies:
   - returns_schema_id
   - capabilities_id
   - values_id (if set)

**API Interaction**:
```
PUT /api/analytics/v1/gts/{query-id}/enablement
Body: { "enabled_for": ["tenant-1", "tenant-2"] } or { "enabled_for": "all" }
```

---

### Flow 4: Query Developer Searches Queries

**Actor**: Query Developer  
**Trigger**: Need to find existing query  
**Goal**: Search and browse query registry

**Steps**:
1. Navigate to Queries
2. Apply filters:
   - By category
   - By name/description (full-text search)
   - By returns_schema_id
   - By enabled status
3. Browse results
4. Select query for details

**API Interaction**:
```
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.query.v1~') and category_id eq 'sales' and enabled eq true&$search=revenue&$select=id,name,description,category_id,returns_schema_id&$orderby=name
GET /api/analytics/v1/gts/{query-id}
```

---

### Flow 5: Query Developer Deletes Query

**Actor**: Query Developer  
**Trigger**: Query no longer needed  
**Goal**: Soft-delete query from registry

**Steps**:
1. Search for query
2. Select query to delete
3. Confirm deletion
4. System checks for dependencies:
   - Warn if used by datasources
   - Prevent deletion if in use
5. Soft-delete query (sets deleted_at)

**API Interaction**:
```
GET /api/analytics/v1/gts?$filter=contains(name,'old_query')
DELETE /api/analytics/v1/gts/{query-id}
```

---

## C. Algorithms

### Service Algorithm 1: Query Metadata Validation

**Purpose**: Validate query metadata against referenced schemas and capabilities

Input: query metadata (returns_schema_id, capabilities_id, values_id)  
Output: validation result or error

1. Validate returns_schema_id exists and is enabled for tenant
   1. Load schema from GTS registry
   2. Check schema exists
   3. Check schema is enabled for tenant
2. Validate capabilities_id exists and is enabled for tenant
   1. Load capabilities from GTS registry
   2. Check capabilities exist
   3. Check capabilities are enabled for tenant
3. **IF** query has values_id:
   1. Load values from GTS registry
   2. Check values exist
   3. Check values are enabled for tenant
   4. Validate values are compatible with capabilities
4. Validate category_id exists
   1. Load category from GTS registry
   2. Check category exists
5. **RETURN** success or error

---

### Service Algorithm 2: Query Enablement with Dependency Cascade

**Purpose**: Enable query and automatically enable all dependencies

Input: query_id, list of tenant_ids  
Output: enablement result or error

1. Load query from GTS registry
2. Collect all dependency IDs:
   1. Add returns_schema_id to dependencies list
   2. Add capabilities_id to dependencies list
   3. **IF** query has values_id:
      1. Add values_id to dependencies list
3. Enable query for all specified tenants:
   1. **FOR EACH** tenant_id in tenant_ids:
      1. Mark query as enabled for tenant_id
4. Cascade enablement to all dependencies:
   1. **FOR EACH** dependency_id in dependencies list:
      1. Load dependency from GTS registry
      2. **FOR EACH** tenant_id in tenant_ids:
         1. Mark dependency as enabled for tenant_id
5. Save all changes to GTS registry:
   1. Save query
   2. **FOR EACH** dependency in dependencies list:
      1. Save dependency
6. **RETURN** success or error

---

## D. States

Query lifecycle state machine:

```
[NOT_CREATED] --create--> [DRAFT]
[DRAFT] --validate--> [VALID]
[VALID] --enable--> [ENABLED]
[ENABLED] --disable--> [DISABLED]
[DISABLED] --enable--> [ENABLED]
[ENABLED/DISABLED] --delete--> [DELETED]
```

**State Transitions**:
- **DRAFT → VALID**: All metadata validated (schema, capabilities, values exist)
- **VALID → ENABLED**: Enablement API called for at least one tenant
- **ENABLED → DISABLED**: Disabled for all tenants
- **DISABLED → ENABLED**: Re-enabled for tenants
- **DELETED**: Soft-deleted (deleted_at timestamp set)

---

## E. Technical Details

### High-Level DB Schema

**Tables**:

**gts_instances** (via GTS registry):
- id (PK): `gts.hypernetix.hyperspot.ax.query.v1~vendor.domain._.query_name.v1`
- type: `gts.hypernetix.hyperspot.ax.query.v1~`
- entity (JSONB): Query metadata
  - name: String
  - description: String
  - category_id: String (FK to category)
  - returns_schema_id: String (FK to schema)
  - capabilities_id: String (FK to capabilities)
  - values_id: String (optional, FK to values)
- created_by: UserID
- created_at: Timestamp
- updated_at: Timestamp
- deleted_at: Timestamp (nullable, for soft-delete)

**gts_enablement** (via GTS registry):
- instance_id (FK to gts_instances.id)
- tenant_id (FK to tenants)
- enabled_at: Timestamp

**Indexes**:
- `idx_queries_type` on (type) WHERE type = 'gts.hypernetix.hyperspot.ax.query.v1~'
- `idx_queries_category` on ((entity->>'category_id'))
- `idx_queries_schema` on ((entity->>'returns_schema_id'))
- `idx_queries_deleted` on (deleted_at) WHERE deleted_at IS NULL
- `idx_queries_tenant` on (instance_id, tenant_id) in gts_enablement

---

### Database Operations

**Create Query**:
```
INSERT INTO gts_instances (id, type, entity, created_by, created_at, updated_at)
VALUES ($1, 'gts.hypernetix.hyperspot.ax.query.v1~', $2, $3, NOW(), NOW())
```

**Search Queries**:
```
SELECT id, type, entity, created_at, updated_at
FROM gts_instances
WHERE type = 'gts.hypernetix.hyperspot.ax.query.v1~'
  AND deleted_at IS NULL
  AND (entity->>'category_id' = $1 OR $1 IS NULL)
  AND (entity->>'name' ILIKE $2 OR $2 IS NULL)
ORDER BY entity->>'name'
LIMIT $3 OFFSET $4
```

**Enable Query for Tenant**:
```
INSERT INTO gts_enablement (instance_id, tenant_id, enabled_at)
VALUES ($1, $2, NOW())
ON CONFLICT (instance_id, tenant_id) DO NOTHING
```

**Get Query with Enablement**:
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
- All query operations require authenticated user
- Tenant isolation enforced on all queries
- Query ownership via `created_by` field
- Admin role required for enablement operations

**Permission Checks**:
- **Query creation**: Requires `analytics:queries:write` permission
- **Query modification**: Requires `analytics:queries:write` permission AND (user is query creator OR user is admin)
- **Query enablement**: Requires `analytics:admin` permission
- **Query deletion**: Requires `analytics:queries:delete` permission AND (user is query creator OR user is admin)

---

### Error Handling

**Common Errors**:
- **404 Not Found**: Query not found or soft-deleted
- **400 Bad Request**: Invalid query metadata
- **403 Forbidden**: Insufficient permissions
- **422 Unprocessable Entity**: Schema validation failure
- **409 Conflict**: Query ID already exists

**Error Response Format (RFC 7807)**:
```json
{
  "type": "https://example.com/problems/query-metadata-invalid",
  "title": "Query Metadata Invalid",
  "status": 422,
  "detail": "returns_schema_id 'invalid-schema' does not exist",
  "instance": "/api/analytics/v1/gts/query-123"
}
```

---

## F. Validation & Implementation

### Testing Scenarios

**Unit Tests**:
- Query metadata validation
- Dependency existence checks
- Enablement cascade logic
- Schema compatibility validation
- Permission checks

**Integration Tests**:
- Query CRUD via GTS API
- Query search with filters
- Enablement with dependency cascade
- Soft-delete and recovery
- Tenant isolation

**Edge Cases**:
1. Query with missing schema reference
2. Query with invalid capabilities_id
3. Query with circular category references
4. Enablement for non-existent tenant
5. Deletion of query in use by datasources
6. Concurrent query updates

**Performance Tests**:
- Query search with 10,000+ queries
- Enablement cascade with deep dependency tree
- Concurrent query registrations

---

### OpenSpec Changes Plan

#### Change 001: Query GTS Type Definition

- **Type**: gts
- **Files**: 
  - `gts/types/hypernetix/hyperspot/ax/query.v1.schema.json`
- **Description**: Define query GTS type schema
- **Dependencies**: None (foundational)
- **Effort**: 4 hours
- **Validation**: Schema validation tests

#### Change 002: Query Repository

- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/query_definitions/repository.rs`
  - `modules/analytics/src/infra/storage/query_definitions/pg_repository.rs`
- **Description**: Query persistence layer with search
- **Dependencies**: Change 001
- **Effort**: 6 hours
- **Validation**: Repository tests

#### Change 003: Query Service

- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/query_definitions/service.rs`
  - `modules/analytics/src/domain/query_definitions/validator.rs`
- **Description**: Query business logic and validation
- **Dependencies**: Change 002
- **Effort**: 8 hours
- **Validation**: Service tests, validation tests

#### Change 004: Query API Handlers

- **Type**: rust
- **Files**: 
  - `modules/analytics/src/api/rest/query_definitions/handlers.rs`
  - `modules/analytics/src/api/rest/query_definitions/dto.rs`
- **Description**: REST API for query CRUD
- **Dependencies**: Change 003
- **Effort**: 6 hours
- **Validation**: API integration tests

#### Change 005: Query Enablement Service

- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/query_definitions/enablement.rs`
- **Description**: Tenant enablement with dependency cascade
- **Dependencies**: Change 003
- **Effort**: 5 hours
- **Validation**: Enablement tests

#### Change 006: Integration Testing Suite

- **Type**: rust (tests)
- **Files**: 
  - `modules/analytics/tests/query_definitions.rs`
- **Description**: E2E query workflow tests
- **Dependencies**: All previous changes
- **Effort**: 6 hours
- **Validation**: 100% scenario coverage

**Total Effort**: 35 hours

---

## Dependencies

- **Depends On**: 
  - [feature-gts-core](../feature-gts-core/) (GTS unified API)
  - [feature-schema-query-returns](../feature-schema-query-returns/) (query result schemas)
- **Blocks**: 
  - [feature-query-execution](../feature-query-execution/) (query execution engine)
  - [feature-datasources](../feature-datasources/) (datasource configuration)

---

## References

- Overall Design: `architecture/DESIGN.md` Section 2 (Actors), Section 3 (System Capabilities)
- GTS Types: [query.v1.schema.json](../../../gts/types/hypernetix/hyperspot/ax/query.v1.schema.json)
- OpenAPI Spec: `architecture/openapi/v1/api.yaml` (query endpoints)
- Feature Manifest: `architecture/features/FEATURES.md` (feature-query-definitions entry)
