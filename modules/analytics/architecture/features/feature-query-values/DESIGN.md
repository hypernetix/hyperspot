# Feature: Query Values

**Slug**: `feature-query-values`
**Status**: 🔄 IN_PROGRESS
**Dependencies**: [feature-gts-core](../feature-gts-core/)

---

## A. Feature Context

### 1. Feature Overview

**Feature**: Query Values

**Purpose**: Manage default OData query options for queries. This feature stores and provides default values for OData parameters ($filter, $select, $orderby, $top, $skip) that can be applied to queries when no explicit parameters are provided by the user. This enables queries to have sensible defaults and provides a foundation for query presets and saved query configurations.

**Scope**:
- GTS type `query.v1~values.v1~` for default OData options storage
- Database tables for query values persistence
- CRUD operations for query values via GTS unified API
- Validation of OData parameters against query capabilities
- Values search and retrieval logic
- Integration with query definitions for default parameter application

**References to OVERALL DESIGN**:
- **GTS Types**: 
  - `query.v1~values.v1~` (owned by this feature) - [gts/types/query/v1/values.schema.json](../../../gts/types/query/v1/values.schema.json)
  - `query.v1~` (referenced) - Base query type
  - `query_capabilities.v1~` (referenced) - For validation

- **OpenAPI Endpoints** (from GTS unified API):
  - `POST /api/analytics/v1/gts` - Create query values instance
  - `GET /api/analytics/v1/gts/{id}` - Retrieve query values
  - `PUT /api/analytics/v1/gts/{id}` - Update query values
  - `PATCH /api/analytics/v1/gts/{id}` - Partial update with JSON Patch
  - `DELETE /api/analytics/v1/gts/{id}` - Delete query values
  - `GET /api/analytics/v1/gts?type=query.v1~values.v1~` - List/search query values

- **Service Roles** (from OpenAPI):
  - `analytics:admin` - Full query values management
  - `analytics:editor` - Create/edit query values
  - `analytics:viewer` - Read-only access

- **User Roles** (from Overall Design):
  - **Platform Administrator** - Configure system-wide default query values
  - **Dashboard Designer** - Define default values for dashboard queries
  - **Business Analyst** - Create saved query configurations with defaults
  - **Plugin Developer** - Register query types with default OData options

- **Actors**: Platform Administrator, Dashboard Designer, Business Analyst, Plugin Developer

---

## B. Actor Flows

### Flow 1: Dashboard Designer Defines Default Query Values

**Actor**: Dashboard Designer  
**Goal**: Set default OData parameters for a query to provide consistent filtering and sorting

1. User navigates to Query Configuration page in UI
2. User selects existing query to configure defaults
3. UI loads query details and capabilities via `GET /api/analytics/v1/gts/{query_id}`
4. UI displays form for OData default values based on query capabilities
5. User enters default values:
   - Default $filter: `status eq 'active'`
   - Default $orderby: `created_at desc`
   - Default $top: `100`
   - Default $select: `id,name,status,created_at`
6. User clicks "Save Defaults"
7. **API**: `POST /api/analytics/v1/gts`
   - Headers: `Authorization: Bearer {jwt}`, `Content-Type: application/json`
   - Body:
     ```json
     {
       "type": "query.v1~values.v1~",
       "query_id": "{query_id}",
       "filter": "status eq 'active'",
       "orderby": "created_at desc",
       "top": 100,
       "select": "id,name,status,created_at"
     }
     ```
   - Returns: `201 Created` with query values instance ID
8. UI shows success notification
9. Query now uses these defaults when no parameters are explicitly provided

### Flow 2: Business Analyst Retrieves Query Values

**Actor**: Business Analyst  
**Goal**: View current default values for a query

1. User opens query details page
2. UI requests query values via `GET /api/analytics/v1/gts?type=query.v1~values.v1~&filter=query_id eq '{query_id}'`
3. **API** returns query values instance or empty result
4. **IF** values exist:
   1. UI displays default parameters in read-only view
   2. Shows: filter, orderby, top, skip, select values
5. **ELSE**:
   1. UI shows "No default values configured"
   2. Displays option to create defaults

### Flow 3: Platform Administrator Updates Query Values

**Actor**: Platform Administrator  
**Goal**: Modify default query parameters

1. User navigates to Query Management console
2. User searches for query values: `GET /api/analytics/v1/gts?type=query.v1~values.v1~`
3. UI displays list of queries with configured defaults
4. User selects query values to edit
5. UI shows edit form with current values populated
6. User modifies values:
   - Changes $top from `100` to `50`
   - Adds $skip: `0`
7. User clicks "Update"
8. **API**: `PUT /api/analytics/v1/gts/{values_id}`
   - Body: Complete updated query values object
   - Returns: `200 OK` with updated instance
9. UI confirms update successful
10. Changes take effect immediately for new query executions

---

## C. Algorithms

### 1. UI Algorithms

**Algorithm: Render Query Values Form**

Input: query_id, query_capabilities  
Output: Form fields for OData defaults

1. Load query capabilities from API: `GET /api/analytics/v1/gts/{capabilities_id}`
2. Initialize empty form configuration
3. **IF** capabilities allow filtering:
   1. Add filter text input field
   2. Show OData filter syntax help
   3. Display filterable properties from capabilities
4. **IF** capabilities allow sorting:
   1. Add orderby field with property selector
   2. Show available sortable properties
   3. Add asc/desc toggle
5. **IF** capabilities allow select:
   1. Add multi-select field for properties
   2. Display all available properties from query schema
6. **IF** capabilities allow paging:
   1. Add $top number input (default: 100)
   2. Add $skip number input (default: 0)
7. Load existing query values if present
8. **IF** existing values found:
   1. Populate form fields with current values
9. Return form configuration

**Algorithm: Validate Query Values Before Submit**

Input: form_values, query_capabilities  
Output: Validation result (success/error list)

1. Initialize empty errors list
2. **IF** $filter is provided:
   1. Parse OData filter expression
   2. **TRY**:
      1. Validate filter syntax
      2. Check properties exist in query schema
   3. **CATCH** ParseError:
      1. Add "Invalid filter syntax" to errors
3. **IF** $orderby is provided:
   1. **FOR EACH** property in orderby:
      1. **IF** property not in sortable list from capabilities:
         1. Add "Property {property} not sortable" to errors
4. **IF** $select is provided:
   1. **FOR EACH** property in select:
      1. **IF** property not in query schema:
         1. Add "Property {property} not found" to errors
5. **IF** $top is provided:
   1. **IF** $top < 1 OR $top > 10000:
      1. Add "$top must be between 1 and 10000" to errors
6. **IF** $skip is provided:
   1. **IF** $skip < 0:
      1. Add "$skip must be non-negative" to errors
7. **IF** errors list is empty:
   1. **RETURN** validation success
8. **ELSE**:
   1. **RETURN** errors list

### 2. Service Algorithms

**Algorithm: Create Query Values**

Input: SecurityCtx, values_payload  
Output: Created query values instance or error

Requires: User has "analytics:editor" or "analytics:admin" permission

1. Extract tenant_id from SecurityCtx
2. Validate request payload structure
3. **IF** payload validation fails:
   1. **RETURN** 400 Bad Request error
4. Verify query_id exists in database
5. **IF** query not found:
   1. **RETURN** 404 error "Query not found"
6. Check tenant access to query
7. **IF** query not enabled for this tenant:
   1. **RETURN** 403 Forbidden error
8. Load query capabilities
9. Validate OData parameters against capabilities (see "Validate OData Parameters" algorithm)
10. **IF** validation fails:
    1. **RETURN** 400 error with validation details
11. Check if query values already exist for this query_id
12. **IF** values already exist:
    1. **RETURN** 409 Conflict error "Query values already exist, use PUT to update"
13. Generate new GTS ID for query values instance
14. Create database record:
    - id, type, tenant_id, query_id
    - filter, orderby, top, skip, select
    - created_at, updated_at, created_by
15. **TRY**:
    1. Insert into query_values table
    2. Commit transaction
16. **CATCH** DatabaseError:
    1. Rollback transaction
    2. **RETURN** 500 Internal Server Error
17. Log audit trail: "Query values created for query {query_id}"
18. **RETURN** 201 Created with query values instance

Performance: < 100ms  
Complexity: O(1)

**Algorithm: Retrieve Query Values by Query ID**

Input: SecurityCtx, query_id  
Output: Query values instance or 404

1. Extract tenant_id from SecurityCtx
2. Query database:
   ```
   SELECT * FROM query_values 
   WHERE query_id = {query_id} 
   AND tenant_id IN (SecurityCtx.accessible_tenants)
   ```
3. **IF** no results:
   1. **RETURN** 404 Not Found
4. Load single result
5. **RETURN** query values instance

Performance: < 50ms  
Complexity: O(1) with index on (query_id, tenant_id)

**Algorithm: Update Query Values**

Input: SecurityCtx, values_id, updated_payload  
Output: Updated instance or error

Requires: User has "analytics:editor" or "analytics:admin" permission

1. Extract tenant_id from SecurityCtx
2. Load existing query values from database
3. **IF** values not found:
   1. **RETURN** 404 Not Found
4. Check tenant access (values.tenant_id in SecurityCtx.accessible_tenants)
5. **IF** access denied:
   1. **RETURN** 403 Forbidden
6. Load query capabilities
7. Validate updated OData parameters against capabilities
8. **IF** validation fails:
   1. **RETURN** 400 error with validation details
9. **TRY**:
   1. Update database record with new values
   2. Set updated_at = current_timestamp
   3. Set updated_by = SecurityCtx.user_id
   4. Commit transaction
10. **CATCH** DatabaseError:
    1. Rollback transaction
    2. **RETURN** 500 Internal Server Error
11. Log audit trail: "Query values updated for {values_id}"
12. **RETURN** 200 OK with updated instance

Performance: < 100ms  
Complexity: O(1)

**Algorithm: Validate OData Parameters**

Input: odata_params, query_capabilities  
Output: Validation result (success/errors)

1. Initialize empty errors list
2. **IF** $filter provided:
   1. **IF** capabilities.FilterRestrictions.Filterable is false:
      1. Add error: "Filtering not allowed for this query"
   2. **ELSE**:
      1. Parse filter expression
      2. **TRY**:
         1. Validate OData filter syntax
         2. Extract properties from filter
         3. **FOR EACH** property in filter:
            1. **IF** property not in FilterRestrictions.FilterableProperties:
               1. Add error: "Property {property} not filterable"
      3. **CATCH** ParseError:
         1. Add error: "Invalid filter syntax: {error_message}"
3. **IF** $orderby provided:
   1. **IF** capabilities.SortRestrictions.Sortable is false:
      1. Add error: "Sorting not allowed for this query"
   2. **ELSE**:
      1. Parse orderby expression
      2. Extract properties
      3. **FOR EACH** property in orderby:
         1. **IF** property not in SortRestrictions.SortableProperties:
            1. Add error: "Property {property} not sortable"
4. **IF** $select provided:
   1. **IF** capabilities.SelectSupport is undefined:
      1. Add error: "Select not supported for this query"
   2. **ELSE**:
      1. Parse select expression
      2. **FOR EACH** property in select:
         1. **IF** property not in query schema:
            1. Add error: "Property {property} not found in schema"
5. **IF** $top provided:
   1. **IF** capabilities.TopSupport is undefined:
      1. Add error: "Top not supported"
   2. **ELSE IF** $top > capabilities.TopSupport.MaxValue:
      1. Add error: "$top exceeds maximum {MaxValue}"
   3. **ELSE IF** $top < 1:
      1. Add error: "$top must be positive"
6. **IF** $skip provided:
   1. **IF** capabilities.SkipSupport is undefined:
      1. Add error: "Skip not supported"
   2. **ELSE IF** $skip < 0:
      1. Add error: "$skip must be non-negative"
7. **IF** errors list is empty:
   1. **RETURN** validation success
8. **ELSE**:
   1. **RETURN** errors list

**Algorithm: Delete Query Values**

Input: SecurityCtx, values_id  
Output: Success or error

Requires: User has "analytics:admin" permission

1. Extract tenant_id from SecurityCtx
2. Load query values from database
3. **IF** not found:
   1. **RETURN** 404 Not Found
4. Check tenant access
5. **IF** access denied:
   1. **RETURN** 403 Forbidden
6. **TRY**:
   1. Delete record from query_values table
   2. Commit transaction
7. **CATCH** DatabaseError:
   1. Rollback transaction
   2. **RETURN** 500 Internal Server Error
8. Log audit trail: "Query values deleted for {values_id}"
9. **RETURN** 204 No Content

Performance: < 50ms  
Complexity: O(1)

**Algorithm: Search Query Values**

Input: SecurityCtx, search_filters (optional)  
Output: Paginated list of query values

1. Extract tenant_id from SecurityCtx
2. Build base query:
   ```
   SELECT * FROM query_values 
   WHERE tenant_id IN (SecurityCtx.accessible_tenants)
   ```
3. **IF** search_filters provided:
   1. **FOR EACH** filter in search_filters:
      1. Add WHERE clause for filter
4. Add ORDER BY clause (default: created_at DESC)
5. Add pagination (LIMIT, OFFSET)
6. Execute query
7. Count total results for pagination metadata
8. **RETURN** paginated result:
   - items: List of query values
   - total_count
   - page, page_size

Performance: < 200ms  
Complexity: O(n) where n = result count

---

## D. States

Query Values are stateless configuration objects. No state machine needed.

---

## E. Technical Details

### 1. High-Level DB Schema

**Table: query_values**

Primary table for storing default OData query options.

Columns:
- `id` (VARCHAR, PRIMARY KEY) - GTS ID: `gts.{domain}.query.v1~{tenant}.values.v1~{uuid}`
- `type` (VARCHAR, NOT NULL) - Always `query.v1~values.v1~`
- `tenant_id` (VARCHAR, NOT NULL) - Tenant identifier from SecurityCtx
- `query_id` (VARCHAR, NOT NULL, UNIQUE) - Reference to query GTS ID (one-to-one)
- `filter` (TEXT, NULLABLE) - Default OData $filter expression
- `orderby` (TEXT, NULLABLE) - Default OData $orderby expression
- `top` (INTEGER, NULLABLE) - Default $top value (page size)
- `skip` (INTEGER, NULLABLE) - Default $skip value (offset)
- `select` (TEXT, NULLABLE) - Default $select fields (comma-separated)
- `created_at` (TIMESTAMP, NOT NULL) - Creation timestamp
- `updated_at` (TIMESTAMP, NOT NULL) - Last update timestamp
- `created_by` (VARCHAR, NOT NULL) - User ID who created
- `updated_by` (VARCHAR, NULLABLE) - User ID who last updated

Indexes:
- PRIMARY KEY on `id`
- UNIQUE INDEX on `query_id` (one query can have only one values instance)
- INDEX on `(tenant_id, type)` for search queries
- INDEX on `created_at` for sorting

Foreign Keys:
- `query_id` references query table (soft reference, validated at application layer)

Constraints:
- `top` CHECK (top IS NULL OR top > 0)
- `skip` CHECK (skip IS NULL OR skip >= 0)
- `type` CHECK (type = 'query.v1~values.v1~')

### 2. Database Operations

**Insert Query Values**:
```sql
INSERT INTO query_values (
  id, type, tenant_id, query_id, 
  filter, orderby, top, skip, select,
  created_at, updated_at, created_by
) VALUES (
  $1, 'query.v1~values.v1~', $2, $3,
  $4, $5, $6, $7, $8,
  NOW(), NOW(), $9
)
```

**Select by Query ID**:
```sql
SELECT * FROM query_values
WHERE query_id = $1 
  AND tenant_id = ANY($2)
```

**Update Query Values**:
```sql
UPDATE query_values
SET filter = $1, orderby = $2, top = $3, 
    skip = $4, select = $5,
    updated_at = NOW(), updated_by = $6
WHERE id = $7 AND tenant_id = ANY($8)
```

**Search Query Values**:
```sql
SELECT * FROM query_values
WHERE tenant_id = ANY($1)
  AND type = 'query.v1~values.v1~'
  AND ($2::text IS NULL OR query_id LIKE $2)
ORDER BY created_at DESC
LIMIT $3 OFFSET $4
```

**Delete Query Values**:
```sql
DELETE FROM query_values
WHERE id = $1 AND tenant_id = ANY($2)
```

### 3. Access Control

**SecurityCtx Requirements**:
- `tenant_id` - Required for tenant isolation
- `user_id` - Required for audit logging
- `permissions` - Required for authorization checks

**Permission Checks**:
- **CREATE**: Requires `analytics:editor` or `analytics:admin`
- **READ**: Requires `analytics:viewer`, `analytics:editor`, or `analytics:admin`
- **UPDATE**: Requires `analytics:editor` or `analytics:admin`
- **DELETE**: Requires `analytics:admin` only

**Tenant Isolation**:
All database queries MUST include tenant_id filter:
```sql
WHERE tenant_id IN (SecurityCtx.accessible_tenants)
```

### 4. Error Handling

**Error Scenarios**:

1. **404 Not Found**:
   - Query values ID doesn't exist
   - Referenced query_id not found
   - Response: `{"error": "Query values not found", "code": "NOT_FOUND"}`

2. **400 Bad Request**:
   - Invalid OData parameter syntax
   - Parameter violates query capabilities
   - Missing required fields
   - Response: `{"error": "Invalid OData parameters", "details": [...], "code": "VALIDATION_ERROR"}`

3. **403 Forbidden**:
   - User lacks required permission
   - Query not enabled for tenant
   - Response: `{"error": "Access denied", "code": "FORBIDDEN"}`

4. **409 Conflict**:
   - Query values already exist for query_id (on CREATE)
   - Response: `{"error": "Query values already exist", "code": "CONFLICT"}`

5. **500 Internal Server Error**:
   - Database connection failure
   - Transaction rollback
   - Response: `{"error": "Internal server error", "code": "INTERNAL_ERROR"}`

**Fallback Logic**:
- **IF** query values not found during query execution: Use empty defaults (no filtering)
- **IF** validation fails: Return detailed error messages with field-level errors
- **IF** database timeout: Retry once, then fail with 503 Service Unavailable

---

## F. Validation & Implementation

### 1. Testing Scenarios

**Unit Tests**:
- Validate OData filter syntax parsing
- Validate orderby expression parsing
- Validate select field list parsing
- Test top/skip numeric validation
- Test parameter validation against capabilities
- Test SecurityCtx tenant isolation

**Integration Tests**:
- Create query values with valid parameters
- Create query values with invalid parameters (expect 400)
- Update query values
- Retrieve query values by query_id
- Delete query values
- Search query values with filters
- Test one-to-one constraint (one query = one values instance)
- Test tenant isolation (cannot access other tenant's values)

**Edge Cases**:
- Empty query values (all fields null)
- Maximum $top value enforcement
- Complex OData filter expressions with nested conditions
- Query values for non-existent query (expect 404)
- Duplicate query values creation (expect 409)
- Update with unchanged values (idempotent)

**Performance Tests**:
- Create 1000 query values instances
- Search across 10000 query values
- Concurrent updates to different query values

**Security Tests**:
- Attempt to create values without permission (expect 403)
- Attempt to access other tenant's values (expect 404/403)
- SQL injection in OData filter parameters
- XSS in filter expressions

### 2. OpenSpec Changes Plan

**Total Changes**: 3
**Estimated Effort**: 12 hours with AI agent

---

### Change 001: Database Schema & Migration

**Status**: ⏳ NOT_STARTED

**Scope**: Create query_values table and indexes

**Tasks**:
- [ ] Create migration file for query_values table
- [ ] Define table schema with all columns
- [ ] Add indexes (primary key, query_id unique, tenant_id, created_at)
- [ ] Add CHECK constraints (top > 0, skip >= 0, type validation)
- [ ] Test migration up/down
- [ ] Verify indexes created correctly

**Files**:
- Backend: `modules/analytics/migrations/00X_create_query_values_table.sql`
- Tests: `modules/analytics/tests/migrations/query_values_migration_test.rs`

**Dependencies**: None

**Effort**: 2 hours (AI agent)

---

### Change 002: Domain Layer - Query Values CRUD

**Status**: ⏳ NOT_STARTED

**Scope**: Implement domain layer for query values operations

**Tasks**:
- [ ] Create `QueryValues` domain struct
- [ ] Implement `QueryValuesRepository` trait
- [ ] Implement database operations (insert, select, update, delete)
- [ ] Add OData parameter validation logic
- [ ] Implement search with filters
- [ ] Add unit tests for validation logic
- [ ] Add integration tests for repository

**Files**:
- Backend: `modules/analytics/src/domain/query_values/mod.rs`
- Backend: `modules/analytics/src/domain/query_values/repository.rs`
- Backend: `modules/analytics/src/domain/query_values/validation.rs`
- Backend: `modules/analytics/src/infrastructure/db/query_values_repository_impl.rs`
- Tests: `modules/analytics/tests/domain/query_values_test.rs`
- Tests: `modules/analytics/tests/integration/query_values_repository_test.rs`

**Dependencies**: Change 001 (database schema)

**Effort**: 6 hours (AI agent)

---

### Change 003: API Layer - GTS Unified API Integration

**Status**: ⏳ NOT_STARTED

**Scope**: Integrate query values with GTS unified API endpoints

**Tasks**:
- [ ] Register `query.v1~values.v1~` type in GTS router
- [ ] Implement POST /gts handler for query values creation
- [ ] Implement GET /gts/{id} handler for retrieval
- [ ] Implement PUT /gts/{id} handler for updates
- [ ] Implement PATCH /gts/{id} handler for partial updates
- [ ] Implement DELETE /gts/{id} handler
- [ ] Implement GET /gts?type=query.v1~values.v1~ for search
- [ ] Add SecurityCtx checks in all handlers
- [ ] Add E2E API tests

**Files**:
- Backend: `modules/analytics/src/api/rest/gts/handlers/query_values.rs`
- Backend: `modules/analytics/src/api/rest/gts/router.rs` (register handler)
- Tests: `testing/e2e/modules/analytics/test_query_values_api.py`

**Dependencies**: Change 002 (domain layer)

**Effort**: 4 hours (AI agent)

---
