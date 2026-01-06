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
- Schema GTS type: `schema.v1~` (base) + `schema.v1~query_returns.v1~`
- Query result schema DB tables
- Schema validation for paginated results
- Scalar-only field enforcement (no nested objects in result fields)
- Custom search/query for schemas
- Schema versioning and evolution

**References to OVERALL DESIGN**:
- **GTS Types**: 
  - `schema.v1~` (base schema type)
  - `schema.v1~query_returns.v1~` (query returns specialization)
- **OpenAPI Endpoints**: 
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
  - Analytics Developer (primary)
  - System Administrator
  - Query Execution Engine (consumer)

---

## B. Actor Flows

### Actor: Analytics Developer

**Goal**: Define schema for query result structure

**Flow**:
1. Developer opens schema creation UI
2. UI fetches available base schema types via `GET /gts?$filter=type_id eq 'schema.v1~'`
3. Developer selects "Query Returns Schema" type
4. UI renders schema editor with field definition form
5. Developer adds fields:
   - Field name (string, required)
   - Field type (scalar types only: string, number, boolean, date, datetime)
   - Optional flag (boolean)
   - Description (string)
6. UI validates scalar-only constraint (no nested objects)
7. Developer submits schema via `POST /gts` with payload
8. Backend validates schema structure
9. Backend stores schema in DB
10. UI displays confirmation with schema ID

**API Interactions**:
- `GET /gts?$filter=type_id eq 'schema.v1~query_returns.v1~'` - List existing schemas
- `POST /gts` - Create new schema
- `GET /gts/{id}` - View schema details
- `PUT /gts/{id}` - Update schema definition
- `DELETE /gts/{id}` - Remove schema

### Actor: Query Execution Engine

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

**Use ADL (Algorithm Description Language)** - see `@/guidelines/ALGORITHM_DESCRIPTION_LANGUAGE.md`

### 1. UI Algorithms

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

**Algorithm: Create Schema**

Input: SecurityCtx, schema_payload
Output: schema_id, HTTP status

1. Validate SecurityCtx has `analytics.developer` or higher role
2. **IF** unauthorized:
   1. **RETURN** 403 Forbidden
3. Extract tenant_id from SecurityCtx
4. Validate schema_payload structure:
   1. Check required fields (name, type_id, fields)
   2. Validate type_id matches `schema.v1~query_returns.v1~`
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
   - type_id: `schema.v1~query_returns.v1~`
   - name: from payload
   - fields: JSON array from payload
   - created_at: current timestamp
   - created_by: user_id from SecurityCtx
8. Index schema for search (name, type_id, tenant_id)
9. **RETURN** 201 Created with schema_id

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
   1. Base: `SELECT * FROM schemas WHERE tenant_id = ? AND type_id LIKE 'schema.v1~%'`
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

**Table: `gts_schemas`** (unified GTS storage)

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PRIMARY KEY | Schema ID |
| tenant_id | UUID | NOT NULL, INDEX | Tenant isolation |
| type_id | VARCHAR(255) | NOT NULL, INDEX | GTS type (schema.v1~query_returns.v1~) |
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
  AND type_id LIKE 'schema.v1~%'
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

## F. Validation & Implementation

### 1. Testing Scenarios

**Unit Tests**:
- Schema creation with valid fields
- Schema creation with non-scalar field (should fail)
- Schema validation against result data
- Field type validation (string, number, boolean, date, datetime)
- Optional field handling
- Tenant isolation enforcement
- Permission checks for each role

**Integration Tests**:
- End-to-end schema CRUD via REST API
- OData query with $filter on schema fields
- OData query with $select field projection
- Optimistic locking (concurrent updates)
- Schema deletion with dependency check
- Query result validation using schema

**Edge Cases**:
- Empty fields array (should fail)
- Field name with special characters
- Very long schema name (>500 chars)
- Large number of fields (100+)
- Malformed JSON in fields array
- Cross-tenant schema access attempt
- Schema update with version mismatch

### 2. OpenSpec Changes Plan

**Total Changes**: 5
**Estimated Effort**: 15-20 hours (with AI agent)

---

### Change 001: GTS Schema Type Definitions

**Status**: ⏳ NOT_STARTED

**Scope**: Define GTS types for schema.v1~ base and schema.v1~query_returns.v1~

**Tasks**:
- [ ] Define `schema.v1~` base type structure
- [ ] Define `schema.v1~query_returns.v1~` specialization
- [ ] Define field definition JSON schema
- [ ] Define scalar type enum
- [ ] Document type hierarchy

**Files**:
- Backend: `modules/analytics/gts/schema.gts`
- Tests: `modules/analytics/tests/gts/schema_types_test.rs`

**Dependencies**: None

**Effort**: 3 hours (AI agent)

---

### Change 002: Database Schema & Migrations

**Status**: ⏳ NOT_STARTED

**Scope**: Create gts_schemas table with proper indexes and constraints

**Tasks**:
- [ ] Create migration for gts_schemas table
- [ ] Add indexes (tenant_id, type_id, state, fields GIN)
- [ ] Add optimistic locking version column
- [ ] Create test fixtures for schemas

**Files**:
- Backend: `modules/analytics/migrations/002_create_schemas_table.sql`
- Tests: `modules/analytics/tests/fixtures/schemas.sql`

**Dependencies**: Change 001

**Effort**: 2 hours (AI agent)

---

### Change 003: Schema Service Layer

**Status**: ⏳ NOT_STARTED

**Scope**: Implement schema CRUD operations with SecurityCtx

**Tasks**:
- [ ] Implement `create_schema(SecurityCtx, payload)`
- [ ] Implement `get_schema(SecurityCtx, id)`
- [ ] Implement `update_schema(SecurityCtx, id, payload)`
- [ ] Implement `delete_schema(SecurityCtx, id)`
- [ ] Implement `search_schemas(SecurityCtx, odata_params)`
- [ ] Implement scalar-only field validation
- [ ] Implement tenant isolation
- [ ] Add permission checks

**Files**:
- Backend: `modules/analytics/src/services/schema_service.rs`
- Tests: `modules/analytics/tests/services/schema_service_test.rs`

**Dependencies**: Change 002

**Effort**: 5 hours (AI agent)

---

### Change 004: Schema REST API Handlers

**Status**: ⏳ NOT_STARTED

**Scope**: Expose schema CRUD via REST endpoints

**Tasks**:
- [ ] Implement `POST /gts` handler for schemas
- [ ] Implement `GET /gts/{id}` handler
- [ ] Implement `PUT /gts/{id}` handler
- [ ] Implement `PATCH /gts/{id}` handler
- [ ] Implement `DELETE /gts/{id}` handler
- [ ] Implement `GET /gts` with OData support
- [ ] Add OpenAPI specs for all endpoints
- [ ] Add request/response DTOs

**Files**:
- Backend: `modules/analytics/src/api/rest/gts_handlers.rs`
- Backend: `modules/analytics/src/api/rest/dto/schema_dto.rs`
- Tests: `modules/analytics/tests/api/rest/schema_endpoints_test.rs`

**Dependencies**: Change 003

**Effort**: 4 hours (AI agent)

---

### Change 005: Schema Validation Utility

**Status**: ⏳ NOT_STARTED

**Scope**: Query result validation against schema

**Tasks**:
- [ ] Implement `validate_result(result_data, schema)`
- [ ] Implement field type checking
- [ ] Implement optional field handling
- [ ] Add detailed validation error messages
- [ ] Add validation metrics/logging

**Files**:
- Backend: `modules/analytics/src/utils/schema_validator.rs`
- Tests: `modules/analytics/tests/utils/schema_validator_test.rs`

**Dependencies**: Change 003

**Effort**: 3 hours (AI agent)

---
