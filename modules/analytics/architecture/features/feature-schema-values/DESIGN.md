# Feature: Values Schema

**Slug**: `feature-schema-values`
**Status**: 🔄 IN_PROGRESS
**Dependencies**: [feature-gts-core](../feature-gts-core/)

---

## A. Feature Context

### 1. Feature Overview

**Feature**: Values Schema

**Purpose**: Value lists schema for UI selectors (dropdowns, autocomplete, pickers) with validation and custom indexing.

**Scope**:
- Schema GTS type: `schema.v1~values.v1~`
- Values schema DB tables
- Value list validation
- Custom indexing for value lists
- Schema versioning and evolution

**References to OVERALL DESIGN**:
- **GTS Types**: 
  - `schema.v1~` (base schema type - inherited)
  - `schema.v1~values.v1~` (values list specialization)
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
  - Values Selector Templates (consumer)

---

## B. Actor Flows

### Actor: Analytics Developer

**Goal**: Define schema for value lists used in UI selectors

**Flow**:
1. Developer opens values schema creation UI
2. UI fetches available schema types via `GET /gts?$filter=type_id eq 'schema.v1~'`
3. Developer selects "Values Schema" type
4. UI renders values schema editor
5. Developer defines value list structure:
   - Value field name (string, required)
   - Label field name (string, required)
   - Description field name (string, optional)
   - Additional metadata fields (optional)
6. Developer configures value constraints:
   - Allowed value types (string, number, boolean)
   - Value validation rules (regex, range, enum)
   - Required fields
7. Developer submits schema via `POST /gts` with payload
8. Backend validates schema structure
9. Backend stores schema in DB with custom indexes
10. UI displays confirmation with schema ID

**API Interactions**:
- `GET /gts?$filter=type_id eq 'schema.v1~values.v1~'` - List existing value schemas
- `POST /gts` - Create new value schema
- `GET /gts/{id}` - View schema details
- `PUT /gts/{id}` - Update schema definition
- `DELETE /gts/{id}` - Remove schema

### Actor: Values Selector Template

**Goal**: Use value schema to render UI selector controls

**Flow**:
1. Template receives values_schema_id from configuration
2. Template fetches schema via `GET /gts/{values_schema_id}`
3. Template parses schema structure (value field, label field, description field)
4. Template fetches actual values data based on schema
5. Template renders selector control (dropdown, autocomplete, etc.)
6. **IF** user selects value:
   1. Template validates selection against schema constraints
   2. Template returns selected value in expected format
7. **IF** validation fails:
   1. Template displays validation error
   2. Template prevents invalid selection

**API Interactions**:
- `GET /gts/{id}` - Fetch values schema definition

---

## C. Algorithms

**Use ADL (Algorithm Description Language)** - see `@/guidelines/ALGORITHM_DESCRIPTION_LANGUAGE.md`

### 1. UI Algorithms

**Algorithm: Render Values Schema Editor**

Input: schema_id (optional for edit mode)
Output: Values schema editor form

1. **IF** schema_id provided (edit mode):
   1. Fetch schema via `GET /gts/{schema_id}`
   2. Parse schema structure definition
   3. Populate form with existing configuration
2. **ELSE** (create mode):
   1. Initialize empty schema structure
3. Render schema configuration form:
   - Schema name (text input)
   - Schema description (textarea)
   - Value field name (text input, required)
   - Label field name (text input, required)
   - Description field name (text input, optional)
4. Render value constraints section:
   - Value type selector (dropdown: string, number, boolean)
   - Validation rules editor
   - Required fields checkboxes
5. Render metadata fields editor (optional additional fields)
6. Render "Save Schema" button

**Algorithm: Validate Values Schema Client-Side**

Input: schema_object
Output: validation_result (boolean), error_messages (array)

1. Initialize error_messages as empty array
2. **IF** schema name is empty:
   1. Add "Schema name is required" to error_messages
3. **IF** value_field_name is empty:
   1. Add "Value field name is required" to error_messages
4. **IF** label_field_name is empty:
   1. Add "Label field name is required" to error_messages
5. **IF** value_type not in allowed_types (string, number, boolean):
   1. Add "Invalid value type" to error_messages
6. **IF** field names contain spaces or special characters:
   1. Add "Invalid field name format" to error_messages
7. **IF** value_field_name equals label_field_name:
   1. Add "Value and label fields must be different" to error_messages
8. **IF** error_messages is empty:
   1. **RETURN** true, empty array
9. **ELSE**:
   1. **RETURN** false, error_messages

**Algorithm: Submit Values Schema**

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

**Algorithm: Create Values Schema**

Input: SecurityCtx, schema_payload
Output: schema_id, HTTP status

1. Validate SecurityCtx has `analytics.developer` or higher role
2. **IF** unauthorized:
   1. **RETURN** 403 Forbidden
3. Extract tenant_id from SecurityCtx
4. Validate schema_payload structure:
   1. Check required fields (name, type_id, value_field, label_field)
   2. Validate type_id matches `schema.v1~values.v1~`
   3. Validate value_field is non-empty string
   4. Validate label_field is non-empty string
5. **IF** value_field equals label_field:
   1. **RETURN** 400 Bad Request with "Value and label fields must be different"
6. Validate value_type is one of: string, number, boolean
7. **IF** value_type invalid:
   1. **RETURN** 400 Bad Request with "Invalid value type"
8. Generate unique schema_id (UUID)
9. Create schema record in DB:
   - id: schema_id
   - tenant_id: from SecurityCtx
   - type_id: `schema.v1~values.v1~`
   - name: from payload
   - value_field: from payload
   - label_field: from payload
   - description_field: from payload (optional)
   - value_type: from payload
   - validation_rules: from payload (JSON)
   - metadata_fields: from payload (JSON array)
   - created_at: current timestamp
   - created_by: user_id from SecurityCtx
10. Create custom indexes for value field and label field
11. **RETURN** 201 Created with schema_id

**Algorithm: Validate Value List Against Schema**

Input: value_list_data, schema_id
Output: is_valid (boolean), validation_errors (array)

1. Fetch schema from DB by schema_id
2. **IF** schema not found:
   1. **RETURN** false, ["Schema not found"]
3. Extract schema configuration (value_field, label_field, value_type, validation_rules)
4. Initialize validation_errors as empty array
5. **IF** value_list_data is not array:
   1. Add "Value list must be array" to validation_errors
   2. **RETURN** false, validation_errors
6. **FOR EACH** item in value_list_data:
   1. **IF** value_field not in item:
      1. Add "Missing required value field" to validation_errors
   2. **IF** label_field not in item:
      1. Add "Missing required label field" to validation_errors
   3. **IF** value_field in item:
      1. Extract value from item
      2. **IF** value type does not match schema value_type:
         1. Add "Type mismatch for value field" to validation_errors
      3. Apply validation_rules to value
      4. **IF** validation fails:
         1. Add validation error to validation_errors
7. **IF** validation_errors is empty:
   1. **RETURN** true, empty array
8. **ELSE**:
   1. **RETURN** false, validation_errors

**Algorithm: Search Values Schemas**

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
   1. Base: `SELECT * FROM gts_schemas WHERE tenant_id = ? AND type_id = 'schema.v1~values.v1~'`
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

**Values Schema Lifecycle**:

```
[DRAFT] --validate--> [ACTIVE]
[ACTIVE] --deprecate--> [DEPRECATED]
[DEPRECATED] --archive--> [ARCHIVED]
```

**States**:
- **DRAFT**: Schema created but not yet validated/published
- **ACTIVE**: Schema in use, can be referenced by value selector templates
- **DEPRECATED**: Schema marked for removal, existing selectors still work but new selectors cannot use it
- **ARCHIVED**: Schema removed from active use, read-only for historical reference

**Transitions**:
- DRAFT → ACTIVE: Manual validation and publication
- ACTIVE → DEPRECATED: Manual deprecation by admin
- DEPRECATED → ARCHIVED: Automated after grace period (e.g., 90 days)

---

## E. Technical Details

### 1. High-Level DB Schema

**Table: `gts_schemas`** (unified GTS storage - reuses same table as query returns schemas)

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PRIMARY KEY | Schema ID |
| tenant_id | UUID | NOT NULL, INDEX | Tenant isolation |
| type_id | VARCHAR(255) | NOT NULL, INDEX | GTS type (schema.v1~values.v1~) |
| name | VARCHAR(500) | NOT NULL | Schema display name |
| description | TEXT | NULL | Schema description |
| structure | JSONB | NOT NULL | Schema structure definition |
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
- GIN: `structure` - JSON field search
- BTREE: `created_at` - Time-based queries

**Structure JSON Format** (for values schemas):
```json
{
  "value_field": "id",
  "label_field": "name",
  "description_field": "description",
  "value_type": "string",
  "validation_rules": {
    "pattern": "^[A-Z0-9_]+$",
    "min_length": 1,
    "max_length": 50
  },
  "metadata_fields": [
    {
      "name": "category",
      "type": "string",
      "optional": true
    },
    {
      "name": "priority",
      "type": "number",
      "optional": true
    }
  ]
}
```

### 2. Database Operations

**Query Patterns**:

1. **Create Values Schema**:
```sql
INSERT INTO gts_schemas (id, tenant_id, type_id, name, structure, created_at, created_by, updated_at, updated_by, version)
VALUES (?, ?, 'schema.v1~values.v1~', ?, ?, NOW(), ?, NOW(), ?, 1)
```

2. **Get Values Schema by ID**:
```sql
SELECT * FROM gts_schemas
WHERE id = ? AND tenant_id = ? AND type_id = 'schema.v1~values.v1~'
```

3. **Search Values Schemas with OData**:
```sql
SELECT * FROM gts_schemas
WHERE tenant_id = ?
  AND type_id = 'schema.v1~values.v1~'
  AND state = 'ACTIVE'
  AND (name ILIKE ? OR description ILIKE ?)
ORDER BY created_at DESC
LIMIT ? OFFSET ?
```

4. **Update Values Schema**:
```sql
UPDATE gts_schemas
SET structure = ?,
    updated_at = NOW(),
    updated_by = ?,
    version = version + 1
WHERE id = ? AND tenant_id = ? AND version = ?
```

5. **Count Selectors Using Schema** (dependency check):
```sql
SELECT COUNT(*) FROM gts_templates
WHERE tenant_id = ? 
  AND type_id = 'template.v1~values_selector.v1~'
  AND structure->>'values_schema_id' = ?
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
| Create Values Schema | `analytics.developer` | Can create schemas in own tenant |
| Read Values Schema | `analytics.viewer` | Can view schemas in own tenant |
| Update Values Schema | `analytics.developer` | Can modify schemas in own tenant |
| Delete Values Schema | `analytics.admin` | Only if no selectors reference it |
| Search Values Schemas | `analytics.viewer` | Filtered by tenant automatically |

**Row-Level Security**:
- All queries automatically filtered by `tenant_id` from SecurityCtx
- Cross-tenant access blocked at DB layer

### 4. Error Handling

**Error Scenarios**:

1. **Values Schema Not Found**:
   - Status: 404 Not Found
   - Body: `{"error": "Values schema not found", "schema_id": "..."}`

2. **Invalid Value Type**:
   - Status: 400 Bad Request
   - Body: `{"error": "Invalid value type", "allowed_types": ["string", "number", "boolean"]}`

3. **Duplicate Field Names**:
   - Status: 400 Bad Request
   - Body: `{"error": "Value and label fields must be different"}`

4. **Schema In Use** (cannot delete):
   - Status: 409 Conflict
   - Body: `{"error": "Schema is referenced by active selector templates", "selector_count": 3}`

5. **Version Conflict** (optimistic locking):
   - Status: 409 Conflict
   - Body: `{"error": "Schema was modified by another user", "current_version": 2}`

6. **Unauthorized Access**:
   - Status: 403 Forbidden
   - Body: `{"error": "Insufficient permissions", "required_role": "analytics.developer"}`

**Fallback Logic**:
- Failed schema validation returns detailed field-level errors
- Schema fetch failures during selector rendering return empty value list
- Transaction rollback on any DB constraint violation

---

## F. Validation & Implementation

### 1. Testing Scenarios

**Unit Tests**:
- Values schema creation with valid structure
- Values schema creation with duplicate field names (should fail)
- Value list validation against schema
- Value type validation (string, number, boolean)
- Validation rules enforcement (regex, range, enum)
- Tenant isolation enforcement
- Permission checks for each role

**Integration Tests**:
- End-to-end values schema CRUD via REST API
- OData query with $filter on schema fields
- OData query with $select field projection
- Optimistic locking (concurrent updates)
- Schema deletion with dependency check
- Value list validation using schema

**Edge Cases**:
- Missing required fields (value_field, label_field)
- Invalid field names (spaces, special characters)
- Very long schema name (>500 chars)
- Large metadata fields array (20+ fields)
- Malformed JSON in structure
- Cross-tenant schema access attempt
- Schema update with version mismatch

### 2. OpenSpec Changes Plan

**Total Changes**: 5
**Estimated Effort**: 15-20 hours (with AI agent)

---

### Change 001: GTS Values Schema Type Definition

**Status**: ⏳ NOT_STARTED

**Scope**: Define GTS type for schema.v1~values.v1~

**Tasks**:
- [ ] Define `schema.v1~values.v1~` specialization
- [ ] Define structure JSON schema for values schemas
- [ ] Define value type enum (string, number, boolean)
- [ ] Define validation rules schema
- [ ] Document type hierarchy and inheritance from schema.v1~

**Files**:
- Backend: `modules/analytics/gts/schema_values.gts`
- Tests: `modules/analytics/tests/gts/schema_values_types_test.rs`

**Dependencies**: None

**Effort**: 3 hours (AI agent)

---

### Change 002: Values Schema DB Extensions

**Status**: ⏳ NOT_STARTED

**Scope**: Add values schema support to existing gts_schemas table

**Tasks**:
- [ ] Verify gts_schemas table supports values schema structure
- [ ] Add custom indexes for value field and label field search
- [ ] Create test fixtures for values schemas
- [ ] Document structure JSON format for values

**Files**:
- Backend: `modules/analytics/migrations/003_add_values_schema_indexes.sql`
- Tests: `modules/analytics/tests/fixtures/values_schemas.sql`

**Dependencies**: Change 001

**Effort**: 2 hours (AI agent)

---

### Change 003: Values Schema Service Layer

**Status**: ⏳ NOT_STARTED

**Scope**: Implement values schema operations with SecurityCtx

**Tasks**:
- [ ] Implement `create_values_schema(SecurityCtx, payload)`
- [ ] Implement `get_values_schema(SecurityCtx, id)`
- [ ] Implement `update_values_schema(SecurityCtx, id, payload)`
- [ ] Implement `delete_values_schema(SecurityCtx, id)`
- [ ] Implement `search_values_schemas(SecurityCtx, odata_params)`
- [ ] Implement value type validation
- [ ] Implement field name validation (no duplicates)
- [ ] Implement tenant isolation
- [ ] Add permission checks

**Files**:
- Backend: `modules/analytics/src/services/values_schema_service.rs`
- Tests: `modules/analytics/tests/services/values_schema_service_test.rs`

**Dependencies**: Change 002

**Effort**: 5 hours (AI agent)

---

### Change 004: Values Schema REST API Handlers

**Status**: ⏳ NOT_STARTED

**Scope**: Expose values schema operations via REST endpoints

**Tasks**:
- [ ] Implement `POST /gts` handler for values schemas
- [ ] Implement `GET /gts/{id}` handler with type filtering
- [ ] Implement `PUT /gts/{id}` handler
- [ ] Implement `PATCH /gts/{id}` handler
- [ ] Implement `DELETE /gts/{id}` handler with dependency check
- [ ] Implement `GET /gts` with OData support for values schemas
- [ ] Add OpenAPI specs for values schema operations
- [ ] Add request/response DTOs for values schemas

**Files**:
- Backend: `modules/analytics/src/api/rest/values_schema_handlers.rs`
- Backend: `modules/analytics/src/api/rest/dto/values_schema_dto.rs`
- Tests: `modules/analytics/tests/api/rest/values_schema_endpoints_test.rs`

**Dependencies**: Change 003

**Effort**: 4 hours (AI agent)

---

### Change 005: Value List Validation Utility

**Status**: ⏳ NOT_STARTED

**Scope**: Value list data validation against values schema

**Tasks**:
- [ ] Implement `validate_value_list(value_list_data, schema)`
- [ ] Implement value type checking (string, number, boolean)
- [ ] Implement validation rules enforcement (regex, range, enum)
- [ ] Implement required field checking
- [ ] Add detailed validation error messages
- [ ] Add validation metrics/logging

**Files**:
- Backend: `modules/analytics/src/utils/values_schema_validator.rs`
- Tests: `modules/analytics/tests/utils/values_schema_validator_test.rs`

**Dependencies**: Change 003

**Effort**: 3 hours (AI agent)

---
