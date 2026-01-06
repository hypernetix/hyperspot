# Tasks: quality-assurance

## 1. Implementation

### 1.1 RFC 7807 Error Handler

- [x] 1.1.1 Create `error_handler.rs` module with unified error handling
- [x] 1.1.2 Implement RFC 7807 Problem Details response format
- [x] 1.1.3 Define error type hierarchy (routing, auth, validation, service)
- [x] 1.1.4 Add trace_id injection for distributed tracing
- [x] 1.1.5 Integrate error handler into existing handlers.rs

### 1.2 Error Scenarios Implementation

- [x] 1.2.1 Implement routing errors (404: unknown type, invalid identifier)
- [x] 1.2.2 Implement authentication errors (401: missing/invalid/expired JWT)
- [x] 1.2.3 Implement authorization errors (403: read-only entity modification)
- [x] 1.2.4 Implement validation errors (400: invalid OData, malformed JSON Patch)
- [x] 1.2.5 Implement service errors (503: domain feature unavailable)

### 1.3 Mock Domain Feature for Testing

- [x] 1.3.1 Create `mock_domain_feature.rs` with configurable behavior
- [x] 1.3.2 Implement CRUD operations simulation
- [x] 1.3.3 Add SecurityCtx validation
- [x] 1.3.4 Add error scenario simulation

## 2. Testing

### 2.1 Routing Requirement Tests (fdd-analytics-feature-gts-core-req-routing)

**From Feature DESIGN.md Section F - Unit Tests:**

- [x] 2.1.1 Test: Routing Table Lookup
  - Input: Various GTS identifiers
  - Expected: Correct domain feature selected
  - Verify: All patterns in routing table covered

- [x] 2.1.2 Test: GTS Identifier Parsing
  - Input: `gts.vendor.pkg.ns.type.v1~instance.v1`
  - Expected: Extract type = `gts.vendor.pkg.ns.type.v1~`
  - Verify: Handles named and UUID instances

- [x] 2.1.3 Test: Query Optimization Validator
  - Input: `$filter=entity/unsupported_field eq 'value'`
  - Expected: HTTP 400 with available fields list
  - Verify: Prevents full table scans

- [x] 2.1.4 Test: Tolerant Reader Pattern
  - Input: POST with system fields in request
  - Expected: System fields ignored, generated values used
  - Verify: Client cannot override id, type, tenant

**From Feature DESIGN.md Section F - Integration Tests:**

- [x] 2.1.5 Test: End-to-End Registration
  - Register type via GTS Core
  - Verify routed to correct domain feature
  - Verify response matches schema

- [x] 2.1.6 Test: OData Query Routing
  - List entities with complex $filter
  - Verify routing to correct features
  - Verify pagination works across features

- [x] 2.1.7 Test: Multi-Feature Metadata
  - Request /$metadata
  - Verify aggregates from all features
  - Verify valid OData CSDL

**From Feature DESIGN.md Section F - Performance Tests:**

- [x] 2.1.8 Test: Routing Overhead
  - Measure routing decision time
  - Target: <1ms per request
  - Verify: O(1) hash lookup

- [x] 2.1.9 Test: Concurrent Requests
  - 1000 concurrent requests
  - Verify: No routing errors
  - Verify: Fair distribution to features

**From Feature DESIGN.md Section F - Edge Cases:**

- [x] 2.1.10 Test: Malformed GTS identifier
- [x] 2.1.11 Test: Empty routing table (no features registered)
- [x] 2.1.12 Test: Feature returns error (propagate correctly)
- [x] 2.1.13 Test: Very long GTS identifier (>500 chars)
- [x] 2.1.14 Test: Special characters in identifier

### 2.2 Middleware Requirement Tests (fdd-analytics-feature-gts-core-req-middleware)

**From Feature DESIGN.md Section F - Testing Scenarios:**

- [x] 2.2.1 Test: JWT Validation with invalid signature
  - Send request with invalid JWT signature
  - Verify HTTP 401 returned
  - Expected: No routing to domain features

- [x] 2.2.2 Test: SecurityCtx Injection
  - Send valid JWT with tenant_id claim
  - Verify SecurityCtx created with correct tenant_id
  - Expected: All downstream calls include SecurityCtx

- [x] 2.2.3 Test: OData Parameter Parsing
  - Send GET request with complex $filter expression
  - Verify parameters parsed into AST
  - Expected: Filter validated against indexed fields

### 2.3 Tolerant Reader Requirement Tests (fdd-analytics-feature-gts-core-req-tolerant-reader)

**From Feature DESIGN.md Section F - Testing Scenarios:**

- [x] 2.3.1 Test: Client Cannot Override System Fields
  - POST request with id, type, tenant in body
  - Verify system fields ignored and generated
  - Expected: Client values discarded

- [x] 2.3.2 Test: Secrets Not Returned
  - GET request for entity with API key in entity object
  - Verify response omits sensitive fields
  - Expected: API keys and credentials excluded from response

- [x] 2.3.3 Test: PATCH Operations Restricted
  - PATCH request attempting to modify /id or /type
  - Verify request rejected with HTTP 400
  - Expected: Only /entity/* paths allowed in JSON Patch

### 2.4 Error Handling Tests

- [x] 2.4.1 Test: RFC 7807 format for all error types (routing, auth, validation, service)
- [x] 2.4.2 Test: trace_id present in all error responses
- [x] 2.4.3 Test: Appropriate HTTP status codes for each error category
- [x] 2.4.4 Test: Error messages are clear and actionable

### 2.5 Validation

- [x] 2.5.1 Validate all tests pass against Feature DESIGN.md Section F acceptance criteria
- [x] 2.5.2 Validate all edge cases from Section F covered
- [x] 2.5.3 Validate performance targets met (routing <1ms, 1000 concurrent OK)
- [x] 2.5.4 Run `cargo test --package analytics` and verify all tests pass
- [x] 2.5.5 Update documentation if needed

## 3. Completion

- [x] 3.1 Run full test suite and verify 100% pass rate
- [x] 3.2 Generate test coverage report (target: >90% for gts_core module)
- [x] 3.3 Update COMPLETION.md with test results and validation scores
- [x] 3.4 Mark change as ready for completion workflow (11)
