# Implementation Tasks

## 1. Implementation

- [x] 1.1 Create JWT validation middleware using `modkit-auth::AuthDispatcher`
- [x] 1.2 Create SecurityCtx injection middleware extracting tenant_id from JWT claims
- [x] 1.3 Implement OData parameter parser using `modkit-odata` crate
- [x] 1.4 Create query optimization validator checking $filter against indexed fields
- [x] 1.5 Wire middleware chain to Axum router (JWT → SecurityCtx → OData → routing)
- [x] 1.6 Implement HTTP 401 error responses for authentication failures
- [x] 1.7 Implement HTTP 400 error responses for invalid OData parameters

## 2. Testing

### Unit Tests (from Section F)

- [x] 2.1 Implement test: JWT Validation
  - Test steps:
    1. Create request with invalid JWT signature
    2. Call JWT validation middleware
    3. Verify HTTP 401 returned
    4. Verify request does not reach routing layer
    5. Test with expired token, malformed token, missing token

- [x] 2.2 Implement test: SecurityCtx Injection
  - Test steps:
    1. Create valid JWT with tenant_id claim
    2. Call SecurityCtx middleware
    3. Verify SecurityCtx created with correct tenant_id
    4. Verify SecurityCtx accessible in downstream handlers
    5. Test with multiple tenant IDs

- [x] 2.3 Implement test: OData Parameter Parsing
  - Test steps:
    1. Create GET request with complex $filter: `entity/name eq 'test' and entity/age gt 18`
    2. Call OData parser middleware
    3. Verify parameters parsed into AST
    4. Verify $select, $orderby, $top, $skiptoken, $count parsed
    5. Test with nested filters and multiple operators

- [x] 2.4 Implement test: Query Optimization Validator
  - Test steps:
    1. Define indexed fields: ["entity/name", "entity/created_at"]
    2. Send $filter with unsupported field: `entity/unsupported_field eq 'value'`
    3. Verify HTTP 400 returned
    4. Verify response includes list of available fields
    5. Test with valid indexed fields passes

### Integration Tests

- [x] 2.5 Implement test: End-to-End Authenticated Request
  - Test steps:
    1. Create valid JWT for tenant "acme"
    2. Send GET request to `/gts/{id}` with JWT
    3. Verify request passes middleware chain
    4. Verify SecurityCtx contains tenant "acme"
    5. Verify routing succeeds

- [x] 2.6 Implement test: OData Query with Routing
  - Test steps:
    1. Send GET request with $filter and $select parameters
    2. Verify OData parsing succeeds
    3. Verify routing to correct feature
    4. Verify query optimization validator runs

### Edge Case Tests

- [x] 2.7 Test missing JWT returns HTTP 401
- [x] 2.8 Test invalid $filter syntax returns HTTP 400
- [x] 2.9 Test $filter with non-indexed fields returns HTTP 400
- [x] 2.10 Test JWT without tenant_id claim returns HTTP 401

### Validation

- [x] 2.11 Validate implementation matches Section B Flow 4 (Middleware Chain)
- [x] 2.12 Validate implementation matches Section C Algorithm 2 (Query Optimization Validator)
- [x] 2.13 Validate implementation matches Section E (Access Control)
- [x] 2.14 Run `cargo check --package analytics` to verify compilation
- [x] 2.15 Run `cargo test --package analytics --lib` to verify all tests pass
