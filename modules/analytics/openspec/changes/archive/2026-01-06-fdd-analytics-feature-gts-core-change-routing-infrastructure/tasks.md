# Implementation Tasks

## 1. Implementation

- [x] 1.1 Create routing table data structure (hash map: GTS type pattern → domain feature)
- [x] 1.2 Use existing GTS identifier parser from `gts` crate (`GtsID::new()` - already in workspace)
- [x] 1.3 Implement routing algorithm with O(1) hash table lookup
- [x] 1.4 Create API layer module structure (`api/rest/gts_core/`)
- [x] 1.5 Define OpenAPI spec for `/gts` endpoints (POST, GET, PUT, PATCH, DELETE)
- [x] 1.6 Implement basic HTTP 404 error responses for unknown types
- [x] 1.7 Wire up routing layer to ModKit REST API infrastructure

## 2. Testing

### Unit Tests (from Section F)

- [x] 2.1 Implement test: Routing Table Lookup
  - Test steps:
    1. Initialize routing table with multiple GTS type patterns
    2. **FOR EACH** GTS type pattern in routing table:
       1. Create test identifier with that type
       2. Call routing lookup function
       3. Verify correct domain feature selected
    3. Verify all patterns in routing table covered by tests

- [x] 2.2 Implement test: GTS Identifier Parsing
  - Test steps:
    1. Create test identifier: `gts.vendor.pkg.ns.type.v1~instance.v1`
    2. Call GTS identifier parser
    3. Verify extracted type equals `gts.vendor.pkg.ns.type.v1~`
    4. Test with named instance: `gts.type~my-instance.v1`
    5. Test with UUID instance: `gts.type~550e8400-e29b-41d4-a716-446655440000.v1`
    6. Verify parser handles both formats correctly

- [x] 2.3 Implement test: End-to-End Registration
  - Test steps:
    1. Create mock domain feature handler
    2. Register type via GTS Core POST `/gts`
    3. Verify request routed to correct mock feature
    4. Verify response matches expected schema
    5. Verify HTTP status code 201 or 200

- [x] 2.4 Implement test: Routing Overhead Performance
  - Test steps:
    1. Initialize routing table with 100 type patterns
    2. **FOR EACH** of 1000 test requests:
       1. Record start time
       2. Call routing lookup
       3. Record end time
       4. Calculate duration
    3. Verify average routing decision time < 1ms
    4. Verify O(1) complexity (constant time regardless of table size)

- [x] 2.5 Implement test: Concurrent Requests
  - Test steps:
    1. Initialize routing table
    2. **PARALLEL** execute 1000 concurrent requests to router
    3. Verify no routing errors occurred
    4. Verify fair distribution to features (no starvation)

### Edge Case Tests

- [x] 2.6 Test malformed GTS identifier returns HTTP 400
- [x] 2.7 Test empty routing table returns HTTP 404
- [x] 2.8 Test very long GTS identifier (>500 chars) handled correctly
- [x] 2.9 Test special characters in identifier handled safely

### Validation

- [x] 2.10 Validate implementation matches Section B Flow 4 (GTS Core Routes CRUD Operations)
- [x] 2.11 Validate implementation matches Section C Algorithm 1 (Routing Logic)
- [x] 2.12 Validate implementation matches Section E (API Endpoints)
- [x] 2.13 Run `cargo check --package analytics` to verify compilation
- [x] 2.14 Update documentation if needed
