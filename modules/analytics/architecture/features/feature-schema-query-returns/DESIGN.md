# Feature: Query Returns Schema

**Slug**: `feature-schema-query-returns`
**Status**: 🔄 IN_PROGRESS
**Dependencies**: [feature-gts-core](../feature-gts-core/)

---

## A. Feature Context

### 1. Feature Overview

**Feature**: Query Returns Schema

**Purpose**: Query result schema type for paginated OData responses with scalar-only field enforcement and validation.

**Scope**:
- Schema GTS type: `gts.hypernetix.hyperspot.ax.schema.v1~` (base) + `gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~`
- Query result schema DB tables
- Schema validation for paginated results
- Scalar-only field enforcement (no nested objects in result fields)
- Custom search/query for schemas
- Schema versioning and evolution

**References to OVERALL DESIGN**:
- **GTS Types**: 
  - `gts.hypernetix.hyperspot.ax.schema.v1~` (base schema type)
  - `gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~` (query returns specialization)
- **OpenAPI Endpoints** (see `@/docs/api/api.json` for complete spec): 
  - `POST /gts` (create schema)
  - `GET /gts/{id}` (read schema)
  - `PUT /gts/{id}` (update schema)
  - `PATCH /gts/{id}` (partial update schema)
  - `DELETE /gts/{id}` (delete schema)
  - `GET /gts?$filter=...` (search schemas)
- **Service Roles** (from OpenAPI):
  - `analytics.admin` - Full schema management
  - `analytics.developer` - Create/update schemas
  - `analytics.viewer` - Read-only access
- **User Roles** (from Overall Design):
  - System Administrator - Full access
  - Analytics Developer - Schema creation and management
  - Business Analyst - Read-only schema browsing
- **Actors**: 
  - Dashboard Designer (primary)
  - Platform Administrator
  - Query Plugin (system component)

---

## B. Actor Flows

### Actor: Dashboard Designer

**ID**: fdd-analytics-feature-schema-query-returns-flow-create-schema

**Goal**: Define schema for query result structure

**Flow**:
1. Designer opens schema creation UI
2. UI fetches available base schema types via `GET /gts?$filter=type_id eq 'gts.hypernetix.hyperspot.ax.schema.v1~'`
3. Designer selects "Query Returns Schema" type
4. UI renders schema editor with field definition form
5. Designer adds fields:
   - Field name (string, required)
   - Field type (scalar types only: string, number, boolean, date, datetime)
   - Optional flag (boolean)
   - Description (string)
6. UI validates scalar-only constraint (no nested objects)
7. Designer submits schema via `POST /gts` with payload
8. Backend validates schema structure
9. Backend stores schema in DB
10. UI displays confirmation with schema ID

**API Interactions**:
- `GET /gts?$filter=type_id eq 'gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~'` - List existing schemas
- `POST /gts` - Create new schema
- `GET /gts/{id}` - View schema details
- `PUT /gts/{id}` - Update schema definition
- `DELETE /gts/{id}` - Remove schema

### Actor: Query Plugin

**ID**: fdd-analytics-feature-schema-query-returns-flow-validate-result

**Goal**: Validate query results against registered schema

**Flow**:
1. Query execution engine receives query request
2. Engine loads query metadata including `returns_schema_id`
3. Engine fetches schema definition via `GET /gts/{returns_schema_id}`
4. Engine executes query against datasource
5. Engine validates result structure against schema
6. **IF** validation fails:
   1. Engine logs validation error
   2. Engine returns 500 error with schema mismatch details
7. **IF** validation succeeds:
   1. Engine returns paginated OData response

**API Interactions**:
- `GET /gts/{id}` - Fetch schema for validation

---

## C. Algorithms

**Use FDL (FDD Description Language)** - see FDD requirements

### 1. UI Algorithms

**ID**: fdd-analytics-feature-schema-query-returns-algo-render-editor

**Algorithm: Render Schema Editor**

Input: schema_id (optional for edit mode)
Output: Schema editor form

1. **IF** schema_id provided (edit mode):
   1. Fetch schema via `GET /gts/{schema_id}`
   2. Parse schema fields array
   3. Populate form with existing fields
2. **ELSE** (create mode):
   1. Initialize empty fields array
3. Render field list editor
4. **FOR EACH** field in fields array:
   1. Render field row with:
      - Name input (text)
      - Type selector (dropdown: string, number, boolean, date, datetime)
      - Optional checkbox
      - Description textarea
      - Remove button
5. Render "Add Field" button
6. Render "Save Schema" button

**ID**: fdd-analytics-feature-schema-query-returns-algo-validate-client

**Algorithm: Validate Schema Client-Side**

Input: schema_object
Output: validation_result (boolean), error_messages (array)

1. Initialize error_messages as empty array
2. **IF** schema name is empty:
   1. Add "Schema name is required" to error_messages
3. **IF** fields array is empty:
   1. Add "At least one field required" to error_messages
4. **FOR EACH** field in fields array:
   1. **IF** field name is empty:
      1. Add "Field name required" to error_messages
   2. **IF** field type not in allowed_scalar_types:
      1. Add "Field type must be scalar" to error_messages
   3. **IF** field name contains spaces or special characters:
      1. Add "Invalid field name format" to error_messages
5. **IF** error_messages is empty:
   1. **RETURN** true, empty array
6. **ELSE**:
   1. **RETURN** false, error_messages

**ID**: fdd-analytics-feature-schema-query-returns-algo-submit-schema

**Algorithm: Submit Schema**

Input: schema_object, mode (create or update)
Output: success (boolean), schema_id

1. Validate schema client-side
2. **IF** validation fails:
   1. Display error messages
   2. **RETURN** false, null
3. **IF** mode is create:
   1. Send `POST /gts` with schema_object
4. **ELSE IF** mode is update:
   1. Send `PUT /gts/{schema_id}` with schema_object
5. **IF** API returns 2xx:
   1. Extract schema_id from response
   2. Display success message
   3. **RETURN** true, schema_id
6. **ELSE**:
   1. Parse error response
   2. Display error messages
   3. **RETURN** false, null

### 2. Service Algorithms

**ID**: fdd-analytics-feature-schema-query-returns-algo-create-schema

**Algorithm: Create Schema**

Input: SecurityCtx, schema_payload
Output: schema_id, HTTP status

1. Validate SecurityCtx has `analytics.developer` or higher role
2. **IF** unauthorized:
   1. **RETURN** 403 Forbidden
3. Extract tenant_id from SecurityCtx
4. Validate schema_payload structure:
   1. Check required fields (name, type_id, fields)
   2. Validate type_id matches `gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~`
   3. Validate fields array not empty
5. **FOR EACH** field in fields array:
   1. Validate field name is non-empty string
   2. Validate field type is scalar (string, number, boolean, date, datetime)
   3. **IF** field type is not scalar:
      1. **RETURN** 400 Bad Request with "Only scalar types allowed in query returns"
6. Generate unique schema_id (UUID)
7. Create schema record in DB:
   - id: schema_id
   - tenant_id: from SecurityCtx
   - type_id: `gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~`
   - name: from payload
   - fields: JSON array from payload
   - created_at: current timestamp
   - created_by: user_id from SecurityCtx
8. Index schema for search (name, type_id, tenant_id)
9. **RETURN** 201 Created with schema_id

**ID**: fdd-analytics-feature-schema-query-returns-algo-validate-result

**Algorithm: Validate Query Result Against Schema**

Input: result_data, schema_id
Output: is_valid (boolean), validation_errors (array)

1. Fetch schema from DB by schema_id
2. **IF** schema not found:
   1. **RETURN** false, ["Schema not found"]
3. Extract fields array from schema
4. Initialize validation_errors as empty array
5. **IF** result_data is not array:
   1. Add "Result must be array" to validation_errors
   2. **RETURN** false, validation_errors
6. **FOR EACH** row in result_data:
   1. **FOR EACH** field_def in schema fields:
      1. Extract field_name from field_def
      2. Extract field_type from field_def
      3. Extract is_optional from field_def
      4. **IF** field_name not in row AND not is_optional:
         1. Add "Missing required field: {field_name}" to validation_errors
      5. **ELSE IF** field_name in row:
         1. Extract field_value from row
         2. **IF** field_value type does not match field_type:
            1. Add "Type mismatch for field {field_name}" to validation_errors
7. **IF** validation_errors is empty:
   1. **RETURN** true, empty array
8. **ELSE**:
   1. **RETURN** false, validation_errors

**ID**: fdd-analytics-feature-schema-query-returns-algo-search-schemas

**Algorithm: Search Schemas**

Input: SecurityCtx, odata_query_params
Output: schemas_array, HTTP status

1. Validate SecurityCtx has `analytics.viewer` or higher role
2. **IF** unauthorized:
   1. **RETURN** 403 Forbidden
3. Extract tenant_id from SecurityCtx
4. Parse OData query parameters:
   - $filter
   - $orderby
   - $top
   - $skip
   - $select
5. Build SQL query:
   1. Base: `SELECT * FROM gts_schemas WHERE tenant_id = ? AND type_id LIKE 'gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~%'`
   2. Apply $filter clauses
   3. Apply $orderby
   4. Apply pagination ($skip, $top)
6. Execute query against DB
7. **FOR EACH** schema in results:
   1. Apply $select field projection
8. **RETURN** 200 OK with schemas array

---

## D. States

### 1. State Machines (Optional)

**ID**: fdd-analytics-feature-schema-query-returns-state-schema

**Schema Lifecycle**:

```
[DRAFT] --validate--> [ACTIVE]
[ACTIVE] --deprecate--> [DEPRECATED]
[DEPRECATED] --archive--> [ARCHIVED]
```

**States**:
- **DRAFT**: Schema created but not yet validated/published
- **ACTIVE**: Schema in use, can be referenced by queries
- **DEPRECATED**: Schema marked for removal, existing queries still work but new queries cannot use it
- **ARCHIVED**: Schema removed from active use, read-only for historical reference

**Transitions**:
- DRAFT → ACTIVE: Manual validation and publication
- ACTIVE → DEPRECATED: Manual deprecation by admin
- DEPRECATED → ARCHIVED: Automated after grace period (e.g., 90 days)

---

## E. Technical Details

### 1. High-Level DB Schema

**Domain Model Reference**: `@/modules/analytics/architecture/DESIGN.md` (Section C: Domain Model)

**GTS Type Definitions**:
- Base type: `gts.hypernetix.hyperspot.ax.schema.v1~` - Generic schema container
- Specialization: `gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~` - Query returns schema

**Table: `gts_schemas`** (unified GTS storage)

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PRIMARY KEY | Schema ID |
| tenant_id | UUID | NOT NULL, INDEX | Tenant isolation |
| type_id | VARCHAR(255) | NOT NULL, INDEX | GTS type (gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~) |
| name | VARCHAR(500) | NOT NULL | Schema display name |
| description | TEXT | NULL | Schema description |
| fields | JSONB | NOT NULL | Array of field definitions |
| metadata | JSONB | NULL | Additional metadata |
| state | VARCHAR(50) | NOT NULL, DEFAULT 'ACTIVE' | Lifecycle state |
| created_at | TIMESTAMP | NOT NULL | Creation timestamp |
| created_by | UUID | NOT NULL | Creator user ID |
| updated_at | TIMESTAMP | NOT NULL | Last update timestamp |
| updated_by | UUID | NOT NULL | Last updater user ID |
| version | INTEGER | NOT NULL, DEFAULT 1 | Optimistic locking |

**Indexes**:
- PRIMARY: `id`
- COMPOSITE: `(tenant_id, type_id)` - Fast tenant + type filtering
- COMPOSITE: `(tenant_id, state)` - Fast state filtering
- GIN: `fields` - JSON field search
- BTREE: `created_at` - Time-based queries

**Fields JSON Structure**:
```json
[
  {
    "name": "order_id",
    "type": "string",
    "optional": false,
    "description": "Order identifier"
  },
  {
    "name": "total_amount",
    "type": "number",
    "optional": false,
    "description": "Order total"
  },
  {
    "name": "order_date",
    "type": "datetime",
    "optional": false,
    "description": "When order was placed"
  }
]
```

### 2. Database Operations

**Query Patterns**:

1. **Create Schema**:
```sql
INSERT INTO gts_schemas (id, tenant_id, type_id, name, fields, created_at, created_by, updated_at, updated_by, version)
VALUES (?, ?, ?, ?, ?, NOW(), ?, NOW(), ?, 1)
```

2. **Get Schema by ID**:
```sql
SELECT * FROM gts_schemas
WHERE id = ? AND tenant_id = ?
```

3. **Search Schemas with OData**:
```sql
SELECT * FROM gts_schemas
WHERE tenant_id = ?
  AND type_id LIKE 'gts.hypernetix.hyperspot.ax.schema.v1~%'
  AND state = 'ACTIVE'
  AND (name ILIKE ? OR description ILIKE ?)
ORDER BY created_at DESC
LIMIT ? OFFSET ?
```

4. **Update Schema**:
```sql
UPDATE gts_schemas
SET fields = ?,
    updated_at = NOW(),
    updated_by = ?,
    version = version + 1
WHERE id = ? AND tenant_id = ? AND version = ?
```

5. **Count Queries Using Schema** (dependency check):
```sql
SELECT COUNT(*) FROM gts_queries
WHERE tenant_id = ? AND returns_schema_id = ?
```

### 3. Access Control

**SecurityCtx Usage**:

All endpoints require `SecurityCtx` as first parameter:
- `tenant_id` - Ensures tenant isolation
- `user_id` - Audit trail (created_by, updated_by)
- `roles` - Permission checks

**Permission Matrix**:

| Operation | Required Role | Notes |
|-----------|--------------|-------|
| Create Schema | `analytics.developer` | Can create schemas in own tenant |
| Read Schema | `analytics.viewer` | Can view schemas in own tenant |
| Update Schema | `analytics.developer` | Can modify schemas in own tenant |
| Delete Schema | `analytics.admin` | Only if no queries reference it |
| Search Schemas | `analytics.viewer` | Filtered by tenant automatically |

**Row-Level Security**:
- All queries automatically filtered by `tenant_id` from SecurityCtx
- Cross-tenant access blocked at DB layer

### 4. Error Handling

**Error Scenarios**:

1. **Schema Not Found**:
   - Status: 404 Not Found
   - Body: `{"error": "Schema not found", "schema_id": "..."}`

2. **Non-Scalar Field Type**:
   - Status: 400 Bad Request
   - Body: `{"error": "Only scalar types allowed in query returns", "field": "..."}`

3. **Duplicate Schema Name** (within tenant):
   - Status: 409 Conflict
   - Body: `{"error": "Schema name already exists", "name": "..."}`

4. **Schema In Use** (cannot delete):
   - Status: 409 Conflict
   - Body: `{"error": "Schema is referenced by active queries", "query_count": 5}`

5. **Version Conflict** (optimistic locking):
   - Status: 409 Conflict
   - Body: `{"error": "Schema was modified by another user", "current_version": 3}`

6. **Unauthorized Access**:
   - Status: 403 Forbidden
   - Body: `{"error": "Insufficient permissions", "required_role": "analytics.developer"}`

**Fallback Logic**:
- Failed schema validation returns detailed field-level errors
- Schema fetch failures during query execution return 500 with context
- Transaction rollback on any DB constraint violation

---

## F. Requirements

### Requirement 1: Schema Type Definition

**ID**: fdd-analytics-feature-schema-query-returns-req-type-definition

**Status**: ⏳ NOT_STARTED

**Description**: The system SHALL support gts.hypernetix.hyperspot.ax.schema.v1~ base type and gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~ specialization for defining query result structures.

**References**:
- [Section C: Algorithm - Create Schema](#2-service-algorithms)
- [Section E: Database Schema](#1-high-level-db-schema)

**Testing Scenarios**:

**ID**: fdd-analytics-feature-schema-query-returns-test-create-base-schema

1. Designer creates schema with type_id gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~
2. System validates type_id format
3. System stores schema in gts_schemas table
4. Verify schema ID generated
5. Verify tenant_id populated from SecurityCtx

**Acceptance Criteria**:
- Schema type follows GTS naming convention
- Base schema type can be extended
- Schema versioning supported

---

### Requirement 2: Scalar-Only Field Enforcement

**ID**: fdd-analytics-feature-schema-query-returns-req-scalar-only

**Status**: ⏳ NOT_STARTED

**Description**: The system MUST enforce scalar-only field types (string, number, boolean, date, datetime) in query return schemas. Nested objects SHALL NOT be allowed.

**References**:
- [Section C: Algorithm - Validate Schema Client-Side](#1-ui-algorithms)
- [Section C: Algorithm - Create Schema](#2-service-algorithms)

**Testing Scenarios**:

**ID**: fdd-analytics-feature-schema-query-returns-test-reject-nested

1. Designer attempts to create schema with nested object field
2. System validates field types
3. System returns 400 Bad Request
4. Verify error message: "Only scalar types allowed in query returns"
5. Verify schema not created

**Acceptance Criteria**:
- Only scalar types accepted: string, number, boolean, date, datetime
- Nested objects rejected with clear error
- Arrays of scalars allowed

---

### Requirement 3: Schema CRUD Operations

**ID**: fdd-analytics-feature-schema-query-returns-req-crud-ops

**Status**: ⏳ NOT_STARTED

**Description**: The system SHALL provide complete CRUD operations for query return schemas via GTS unified API endpoints with SecurityCtx enforcement.

**References**:
- [Section B: Actor Flow - Create Schema](#actor-analytics-developer)
- [Section E: Access Control](#3-access-control)

**Testing Scenarios**:

**ID**: fdd-analytics-feature-schema-query-returns-test-crud-lifecycle

1. Designer with analytics.developer role creates schema via POST /gts
2. System validates SecurityCtx permissions
3. System creates schema with tenant isolation
4. Designer retrieves schema via GET /gts/{id}
5. Designer updates schema via PUT /gts/{id}
6. System validates optimistic locking version
7. Designer deletes unused schema via DELETE /gts/{id}
8. Verify all operations filtered by tenant_id

**Acceptance Criteria**:
- Create, Read, Update, Delete operations supported
- Tenant isolation enforced on all operations
- Optimistic locking prevents concurrent update conflicts
- Permission checks per operation type

---

### Requirement 4: Query Result Validation

**ID**: fdd-analytics-feature-schema-query-returns-req-result-validation

**Status**: ⏳ NOT_STARTED

**Description**: The query execution engine MUST validate query results against registered schemas before returning data to clients.

**References**:
- [Section B: Actor Flow - Validate Result](#actor-query-execution-engine)
- [Section C: Algorithm - Validate Query Result](#2-service-algorithms)
- [Section E: Error Handling](#4-error-handling)

**Testing Scenarios**:

**ID**: fdd-analytics-feature-schema-query-returns-test-validation-success

1. Query execution engine fetches schema by returns_schema_id
2. Engine executes query against datasource
3. Engine validates result structure against schema
4. All required fields present in result
5. All field types match schema definition
6. Engine returns validated result with 200 OK

**ID**: fdd-analytics-feature-schema-query-returns-test-validation-failure

1. Query execution engine fetches schema
2. Engine executes query
3. Result missing required field
4. Engine detects validation error
5. Engine returns 500 with schema mismatch details
6. Verify error logged for debugging

**Acceptance Criteria**:
- All query results validated before return
- Required field presence checked
- Field type matching enforced
- Validation errors returned with details
- Optional fields handled correctly

---

### Requirement 5: OData Search Support

**ID**: fdd-analytics-feature-schema-query-returns-req-odata-search

**Status**: ⏳ NOT_STARTED

**Description**: The system SHALL support OData v4 query capabilities for searching and filtering schemas including $filter, $orderby, $top, $skip, and $select.

**References**:
- [Section C: Algorithm - Search Schemas](#2-service-algorithms)
- [Section E: Database Operations](#2-database-operations)

**Testing Scenarios**:

**ID**: fdd-analytics-feature-schema-query-returns-test-odata-filter

1. Client sends GET /gts?$filter=type_id eq 'gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~'
2. System parses OData filter expression
3. System builds SQL query with tenant_id and type_id filters
4. System executes query
5. System returns matching schemas
6. Verify only schemas from current tenant returned

**ID**: fdd-analytics-feature-schema-query-returns-test-odata-pagination

1. Client sends GET /gts?$top=10&$skip=20
2. System applies LIMIT and OFFSET to SQL
3. System returns page 3 of results (records 21-30)
4. Verify correct pagination

**Acceptance Criteria**:
- $filter supports equality, comparison, logical operators
- $orderby supports ascending/descending sort
- $top and $skip enable pagination
- $select enables field projection
- All operations respect tenant isolation

---

## G. Additional Context

### Dependencies

**Depends On**:
- [feature-gts-core](../feature-gts-core/) - GTS unified API routing layer (REQUIRED)

**Blocks**:
- [feature-query-definitions](../feature-query-definitions/) - Queries reference schema via returns_schema_id

**Related Features**:
- [feature-schema-template-config](../feature-schema-template-config/) - Similar schema pattern for templates
- [feature-schema-values](../feature-schema-values/) - Similar schema pattern for value lists

### References

- **Overall Design**: `@/modules/analytics/architecture/DESIGN.md`
- **FEATURES Manifest**: `@/modules/analytics/architecture/features/FEATURES.md`
- **GTS Documentation**: `@/docs/GTS.md`
- **Secure ORM Guide**: `@/docs/SECURE-ORM.md`
- **OData Spec**: `@/docs/ODATA_SELECT.md`

### Implementation Notes

**Implementation Plan**: See `CHANGES.md` for complete implementation plan with 5 atomic changes (17 hours estimated)

**Design Decisions**:
- Unified gts_schemas table for all schema types (not schema-specific tables)
- JSONB for flexible field definitions
- Optimistic locking via version column
- GIN index on fields JSONB for search performance

**Future Enhancements**:
- Schema migration tools for breaking changes
- Schema diff/changelog generation
- Schema templates/presets library
- Schema import/export utilities

---
