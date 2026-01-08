# Implementation Plan: Query Returns Schema

**Feature**: `feature-schema-query-returns`  
**Version**: 1.0  
**Last Updated**: 2026-01-09  
**Status**: ⏳ NOT_STARTED

**Feature DESIGN**: `@/modules/analytics/architecture/features/feature-schema-query-returns/DESIGN.md`

---

## Summary

**Total Changes**: 5  
**Completed**: 1  
**In Progress**: 0  
**Not Started**: 4

**Estimated Effort**: 17 hours (AI agent)

---

## Change 1: Rust GTS Type Definitions

**ID**: fdd-analytics-feature-schema-query-returns-change-rust-gts-types  
**Status**: ✅ COMPLETED  
**Priority**: HIGH  
**Effort**: 3 hours  
**Implements**: `fdd-analytics-feature-schema-query-returns-req-type-definition`

---

### Objective

Define Rust GTS type structures using `struct_to_gts_schema` macro to generate JSON Schema files for schema.v1~ base type and query_returns.v1~ specialization.

### Requirements Coverage

**Implements**:
- **`fdd-analytics-feature-schema-query-returns-req-type-definition`**: System SHALL support gts.hypernetix.hyperspot.ax.schema.v1~ base type and query_returns specialization

**References**:
- Technical Detail: Section E.1 - Database Schema (references GTS types)
- Overall Design: Section C.2 - Domain Model (GTS type hierarchy)

### Tasks

## 1. Implementation

### 1.1 Create GTS Module in SDK
- [x] 1.1.1 Create directory `modules/analytics/analytics-sdk/src/gts/`
- [x] 1.1.2 Create file `modules/analytics/analytics-sdk/src/gts/mod.rs`
- [x] 1.1.3 Create file `modules/analytics/analytics-sdk/src/gts/schema.rs`
- [x] 1.1.4 Export gts module in `analytics-sdk/src/lib.rs`

### 1.2 Define Base Schema Type
- [x] 1.2.1 Define `SchemaV1` struct as marker type (macro not used - schemas maintained manually)
- [x] 1.2.2 Reference schema_id `gts.hypernetix.hyperspot.ax.schema.v1~` in documentation
- [x] 1.2.3 Link to existing schema at `gts/types/schema/v1`
- [x] 1.2.4 Add documentation describing base schema purpose

### 1.3 Define Query Returns Specialization
- [x] 1.3.1 Define `QueryReturnsSchemaV1` struct as marker type
- [x] 1.3.2 Document relationship to SchemaV1 (inheritance in JSON Schema)
- [x] 1.3.3 Reference full query_returns type ID in documentation
- [x] 1.3.4 Add documentation for scalar-only constraint
- [x] 1.3.5 Add helper methods: `new()`, `validate_scalar_fields()`

### 1.4 Schema Verification
- [x] 1.4.1 Create backup of existing schemas: `cp -r gts/types/schema/v1 gts/types/schema/v1.backup`
- [x] 1.4.2 Note current schema file sizes and checksums
- [x] 1.4.3 Document existing schema structure for comparison

## 2. Testing

### 2.1 Build and Generation
- [x] 2.1.1 Run `cargo build -p analytics-sdk`
- [x] 2.1.2 Verify compilation successful
- [x] 2.1.3 Verify JSON Schema files unchanged (not regenerated - maintained manually)

### 2.2 Schema Comparison
- [x] 2.2.1 Compare base.schema.json: No differences found
- [x] 2.2.2 Compare query_returns.schema.json with backup: Identical
- [x] 2.2.3 Verify schema IDs are identical: ✅ Confirmed
- [x] 2.2.4 Verify "allOf" inheritance structure unchanged: ✅ Preserved
- [x] 2.2.5 Verify "examples" field preserved: ✅ Intact
- [x] 2.2.6 No differences found - schemas 100% identical

### 2.3 Final Verification
- [x] 2.3.1 Confirm schemas unchanged (marker types approach - no generation)
- [x] 2.3.2 Test that existing GTS validation still works: ✅ All tests pass (3/3)
- [x] 2.3.3 Remove backup: `rm -rf gts/types/schema/v1.backup` ✅ Done
- [x] 2.3.4 Documented approach: Marker types without macro generation

### Specification

**Code Changes**:
- Module: `analytics-sdk/src/gts/`
- Files: `mod.rs`, `schema.rs`
- Structs: `SchemaV1`, `QueryReturnsSchemaV1`
- Macros: `#[struct_to_gts_schema(...)]`
- Generated: JSON Schema files at `gts/types/schema/v1/`
- **Code Tagging**: MUST tag with `// @fdd-change:fdd-analytics-feature-schema-query-returns-change-rust-gts-types` or `// @fdd-change:change-rust-gts-types`

**Example Structure**:
```rust
// analytics-sdk/src/gts/schema.rs
use gts_macros::struct_to_gts_schema;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[struct_to_gts_schema(
    dir_path = "../../../gts/types/schema/v1",
    schema_id = "gts.hypernetix.hyperspot.ax.schema.v1~",
    description = "Base schema type for defining data structures"
)]
pub struct SchemaV1;

#[struct_to_gts_schema(
    dir_path = "../../../gts/types/schema/v1",
    base = SchemaV1,
    schema_id = "gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~",
    description = "Query returns schema with scalar-only fields"
)]
pub struct QueryReturnsSchemaV1;
```

### Dependencies

**Depends on**: None (foundational)

**Blocks**: Change 2, 3, 4, 5

### Testing

**Build Tests**:
- Verify: Rust code compiles without errors
- Verify: JSON Schema files generated
- Verify: Schema structure matches design specification

### Validation Criteria

**Code validation**:
- All tasks completed
- Rust structs compile successfully
- JSON Schema files generated in correct location
- **CRITICAL**: Generated schemas are identical to existing schemas (diff shows no changes OR only acceptable improvements)
- Schema IDs match specification exactly
- Inheritance structure correct (allOf reference)
- Examples preserved in generated schemas
- **Code tagged**: All code has `@fdd-change:change-rust-gts-types` tags
- Implements `fdd-analytics-feature-schema-query-returns-req-type-definition`

**Safety checks**:
- Backup created before generation
- Diff performed between old and new schemas
- Any differences documented and justified
- No breaking changes to existing schema structure

---

## Change 2: Schema Service Layer

**ID**: fdd-analytics-feature-schema-query-returns-change-schema-service  
**Status**: ⏳ NOT_STARTED  
**Priority**: HIGH  
**Effort**: 5 hours  
**Implements**: `fdd-analytics-feature-schema-query-returns-req-scalar-only`, `fdd-analytics-feature-schema-query-returns-req-crud-ops`, `fdd-analytics-feature-schema-query-returns-req-odata-search`

---

### Objective

Implement schema CRUD operations with SecurityCtx enforcement, scalar-only field validation, and OData search capabilities.

### Requirements Coverage

**Implements**:
- **`fdd-analytics-feature-schema-query-returns-req-scalar-only`**: Enforce scalar-only field types
- **`fdd-analytics-feature-schema-query-returns-req-crud-ops`**: Complete CRUD operations with SecurityCtx
- **`fdd-analytics-feature-schema-query-returns-req-odata-search`**: OData v4 query support

**References**:
- Algorithm: Section C - Create Schema, Search Schemas algorithms
- Technical Detail: Section E.2 - Database Operations
- Technical Detail: Section E.3 - Access Control
- Actor Flow: Section B - Analytics Developer flow

### Tasks

## 1. Implementation

### 1.1 Create Service Module
- [ ] 1.1.1 Create service file `modules/analytics/src/services/schema_service.rs`
- [ ] 1.1.2 Define SchemaService struct with repository dependency
- [ ] 1.1.3 Add SecurityCtx to all method signatures

### 1.2 Implement CRUD Operations
- [ ] 1.2.1 Implement `create_schema(&SecurityCtx, payload) -> Result<SchemaId>`
- [ ] 1.2.2 Implement `get_schema(&SecurityCtx, id) -> Result<Schema>`
- [ ] 1.2.3 Implement `update_schema(&SecurityCtx, id, payload) -> Result<Schema>`
- [ ] 1.2.4 Implement `delete_schema(&SecurityCtx, id) -> Result<()>`
- [ ] 1.2.5 Implement optimistic locking version check in update

### 1.3 Implement Validation
- [ ] 1.3.1 Implement scalar-only field type validation function
- [ ] 1.3.2 Add validation in create_schema before DB insert
- [ ] 1.3.3 Add validation in update_schema before DB update
- [ ] 1.3.4 Return 400 Bad Request with clear error on validation failure

### 1.4 Implement Search
- [ ] 1.4.1 Implement `search_schemas(&SecurityCtx, ODataParams) -> Result<Vec<Schema>>`
- [ ] 1.4.2 Parse $filter, $orderby, $top, $skip, $select
- [ ] 1.4.3 Build SQL query with tenant_id filter
- [ ] 1.4.4 Apply field projection from $select

### 1.5 Implement Security
- [ ] 1.5.1 Add tenant isolation: filter all queries by SecurityCtx.tenant_id
- [ ] 1.5.2 Add permission checks: analytics.developer for write, analytics.viewer for read
- [ ] 1.5.3 Populate created_by, updated_by from SecurityCtx.user_id

## 2. Testing

### 2.1 Service Tests
- [ ] 2.1.1 Create test file `modules/analytics/tests/services/schema_service_test.rs`
- [ ] 2.1.2 Test create with valid scalar fields → success
- [ ] 2.1.3 Test create with nested object field → 400 error
- [ ] 2.1.4 Test get by ID with correct tenant → success
- [ ] 2.1.5 Test get by ID with wrong tenant → 404
- [ ] 2.1.6 Test update with correct version → success
- [ ] 2.1.7 Test update with wrong version → 409 conflict
- [ ] 2.1.8 Test delete unused schema → success
- [ ] 2.1.9 Test search with $filter → correct results
- [ ] 2.1.10 Test search respects tenant isolation

### Specification

**Code Changes**:
- Module: `services/schema_service.rs`
- Functions:
  - `create_schema(&SecurityCtx, CreateSchemaPayload) -> Result<Schema, DomainError>`
  - `get_schema(&SecurityCtx, SchemaId) -> Result<Schema, DomainError>`
  - `update_schema(&SecurityCtx, SchemaId, UpdateSchemaPayload) -> Result<Schema, DomainError>`
  - `delete_schema(&SecurityCtx, SchemaId) -> Result<(), DomainError>`
  - `search_schemas(&SecurityCtx, ODataQueryParams) -> Result<Vec<Schema>, DomainError>`
  - `validate_scalar_fields(&[FieldDefinition]) -> Result<(), ValidationError>`
- Implementation: Business logic layer with Secure ORM
- **Code Tagging**: MUST tag all service code with `// @fdd-change:fdd-analytics-feature-schema-query-returns-change-schema-service` or short `// @fdd-change:change-schema-service`

### Dependencies

**Depends on**: Change 1 (Rust GTS types must be defined)

**Blocks**: Change 3, 4, 5

### Testing

**Unit Tests**:
- Test: All service operations with various scenarios
- File: `modules/analytics/tests/services/schema_service_test.rs`
- Validates: Business logic, validation, security enforcement

### Validation Criteria

**Code validation**:
- All tasks completed
- All tests pass
- Scalar-only validation works
- Tenant isolation enforced
- Permission checks work
- SecurityCtx propagated correctly
- **Code tagged**: All service functions have `@fdd-change:change-schema-service` tags
- Implements all 3 requirements

---

## Change 3: Schema REST API Handlers

**ID**: fdd-analytics-feature-schema-query-returns-change-schema-rest-api  
**Status**: ⏳ NOT_STARTED  
**Priority**: HIGH  
**Effort**: 4 hours  
**Implements**: `fdd-analytics-feature-schema-query-returns-req-crud-ops`, `fdd-analytics-feature-schema-query-returns-req-odata-search`

---

### Objective

Expose schema CRUD and search operations via REST API endpoints using GTS unified API pattern with OpenAPI documentation.

### Requirements Coverage

**Implements**:
- **`fdd-analytics-feature-schema-query-returns-req-crud-ops`**: REST endpoints for CRUD
- **`fdd-analytics-feature-schema-query-returns-req-odata-search`**: REST endpoint for OData search

**References**:
- Actor Flow: Section B - Analytics Developer flow (API interactions)
- Technical Detail: Section E.4 - Error Handling
- Overall Design: Section C - OpenAPI Endpoints

### Tasks

## 1. Implementation

### 1.1 Create DTOs
- [ ] 1.1.1 Create DTO file `modules/analytics/src/api/rest/dto/schema_dto.rs`
- [ ] 1.1.2 Define CreateSchemaRequest DTO with validation
- [ ] 1.1.3 Define UpdateSchemaRequest DTO
- [ ] 1.1.4 Define SchemaResponse DTO with ToSchema derive
- [ ] 1.1.5 Define SchemaListResponse DTO for search results

### 1.2 Implement Handlers
- [ ] 1.2.1 Create handler file `modules/analytics/src/api/rest/gts_handlers.rs`
- [ ] 1.2.2 Implement `POST /gts` handler (schemas filtered by type_id)
- [ ] 1.2.3 Implement `GET /gts/{id}` handler
- [ ] 1.2.4 Implement `PUT /gts/{id}` handler
- [ ] 1.2.5 Implement `PATCH /gts/{id}` handler (JSON Patch support)
- [ ] 1.2.6 Implement `DELETE /gts/{id}` handler
- [ ] 1.2.7 Implement `GET /gts` with OData query params

### 1.3 Add OpenAPI Documentation
- [ ] 1.3.1 Add OpenAPI operation specs for all endpoints
- [ ] 1.3.2 Document request/response schemas
- [ ] 1.3.3 Document error responses (400, 403, 404, 409, 500)
- [ ] 1.3.4 Add examples for each endpoint

### 1.4 Error Handling
- [ ] 1.4.1 Map DomainError::NotFound → 404
- [ ] 1.4.2 Map DomainError::ValidationError → 400 with details
- [ ] 1.4.3 Map DomainError::Conflict → 409 (version, name duplicate)
- [ ] 1.4.4 Map DomainError::Unauthorized → 403
- [ ] 1.4.5 Return RFC 7807 Problem Details format

## 2. Testing

### 2.1 API Tests
- [ ] 2.1.1 Create test file `modules/analytics/tests/api/rest/schema_endpoints_test.rs`
- [ ] 2.1.2 Test POST /gts → 201 Created with schema ID
- [ ] 2.1.3 Test GET /gts/{id} → 200 OK with schema
- [ ] 2.1.4 Test GET /gts/{id} non-existent → 404
- [ ] 2.1.5 Test PUT /gts/{id} → 200 OK with updated schema
- [ ] 2.1.6 Test DELETE /gts/{id} → 204 No Content
- [ ] 2.1.7 Test GET /gts with $filter → 200 OK with filtered results
- [ ] 2.1.8 Test GET /gts with $top/$skip → pagination works
- [ ] 2.1.9 Test POST with nested field → 400 Bad Request

### Specification

**API Changes**:
- Endpoints: POST/GET/PUT/PATCH/DELETE /gts, GET /gts (list)
- All endpoints filtered by type_id=gts.hypernetix.hyperspot.ax.schema.v1~ variants
- Request DTOs: CreateSchemaRequest, UpdateSchemaRequest
- Response DTOs: SchemaResponse, SchemaListResponse
- Error responses: 400, 403, 404, 409, 500 with Problem Details

**Code Changes**:
- Module: `api/rest/`
- Files: `gts_handlers.rs`, `dto/schema_dto.rs`
- Implementation: Axum handlers with OperationBuilder
- **Code Tagging**: MUST tag all API code with `// @fdd-change:fdd-analytics-feature-schema-query-returns-change-schema-rest-api` or short `// @fdd-change:change-schema-rest-api`

### Dependencies

**Depends on**: Change 2 (service layer must exist)

**Blocks**: None

### Testing

**Integration Tests**:
- Test: End-to-end API operations
- File: `modules/analytics/tests/api/rest/schema_endpoints_test.rs`
- Validates: All endpoints work, OpenAPI spec correct, errors formatted correctly

### Validation Criteria

**Code validation**:
- All tasks completed
- All endpoint tests pass
- OpenAPI documentation complete
- Error handling follows RFC 7807
- DTOs have validation
- **Code tagged**: All handlers and DTOs have `@fdd-change:change-schema-rest-api` tags
- Implements both requirements

---

## Change 4: Schema Validation Utility

**ID**: fdd-analytics-feature-schema-query-returns-change-schema-validator  
**Status**: ⏳ NOT_STARTED  
**Priority**: MEDIUM  
**Effort**: 3 hours  
**Implements**: `fdd-analytics-feature-schema-query-returns-req-result-validation`

---

### Objective

Implement query result validation utility that validates result data against registered schemas before returning to clients.

### Requirements Coverage

**Implements**:
- **`fdd-analytics-feature-schema-query-returns-req-result-validation`**: Query execution engine validates results against schemas

**References**:
- Algorithm: Section C - Validate Query Result Against Schema
- Actor Flow: Section B - Query Execution Engine flow
- Technical Detail: Section E.4 - Error Handling (validation errors)

### Tasks

## 1. Implementation

### 1.1 Create Validator Module
- [ ] 1.1.1 Create utility file `modules/analytics/src/utils/schema_validator.rs`
- [ ] 1.1.2 Define ValidationResult enum (Success, Failure with errors)
- [ ] 1.1.3 Define ValidationError struct with field-level details

### 1.2 Implement Validation Logic
- [ ] 1.2.1 Implement `validate_result(result_data, schema) -> ValidationResult`
- [ ] 1.2.2 Implement field presence check for required fields
- [ ] 1.2.3 Implement field type matching (string, number, boolean, date, datetime)
- [ ] 1.2.4 Implement optional field handling
- [ ] 1.2.5 Collect all validation errors (don't fail on first error)

### 1.3 Add Error Messages
- [ ] 1.3.1 Add detailed error message for missing required field
- [ ] 1.3.2 Add detailed error message for type mismatch
- [ ] 1.3.3 Include field name, expected type, actual type in errors
- [ ] 1.3.4 Return array of all validation errors

### 1.4 Add Metrics
- [ ] 1.4.1 Add validation_success counter metric
- [ ] 1.4.2 Add validation_failure counter metric
- [ ] 1.4.3 Add validation_duration histogram metric
- [ ] 1.4.4 Log validation failures with schema_id and error details

## 2. Testing

### 2.1 Validator Tests
- [ ] 2.1.1 Create test file `modules/analytics/tests/utils/schema_validator_test.rs`
- [ ] 2.1.2 Test validation with all fields present and correct types → success
- [ ] 2.1.3 Test validation with missing required field → failure
- [ ] 2.1.4 Test validation with type mismatch → failure
- [ ] 2.1.5 Test validation with missing optional field → success
- [ ] 2.1.6 Test validation with multiple errors → all errors returned
- [ ] 2.1.7 Test validation with empty result array → success
- [ ] 2.1.8 Test validation with large result set (performance)

### Specification

**Code Changes**:
- Module: `utils/schema_validator.rs`
- Functions:
  - `validate_result(result: &[Row], schema: &Schema) -> ValidationResult`
  - `check_field_presence(row: &Row, field_def: &FieldDef) -> Option<ValidationError>`
  - `check_field_type(value: &Value, expected_type: FieldType) -> Option<ValidationError>`
- Implementation: Pure function with no side effects except metrics/logging
- **Code Tagging**: MUST tag all validator code with `// @fdd-change:fdd-analytics-feature-schema-query-returns-change-schema-validator` or short `// @fdd-change:change-schema-validator`

### Dependencies

**Depends on**: Change 2 (service layer provides schema access)

**Blocks**: None (query execution engine will use this)

### Testing

**Unit Tests**:
- Test: All validation scenarios
- File: `modules/analytics/tests/utils/schema_validator_test.rs`
- Validates: Validation logic correct, all edge cases handled

### Validation Criteria

**Code validation**:
- All tasks completed
- All tests pass
- Validation logic comprehensive
- Error messages clear and actionable
- Metrics/logging in place
- Performance acceptable for large result sets
- **Code tagged**: All validator functions have `@fdd-change:change-schema-validator` tags
- Implements requirement

---
