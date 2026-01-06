## ADDED Requirements

### Requirement: Error Handling with RFC 7807 Problem Details

The system SHALL implement RFC 7807 Problem Details format for all error responses across routing, authentication, authorization, validation, and service error scenarios. The error handler MUST include trace_id for distributed tracing and MUST provide clear, actionable error messages with appropriate HTTP status codes.

**ID**: `fdd-analytics-feature-gts-core-req-error-handling`

**Normative Requirements**:
- SHALL return RFC 7807 Problem Details format for all errors
- MUST include trace_id in all error responses
- SHALL use appropriate HTTP status codes (404, 401, 403, 400, 503)
- MUST provide clear error messages with actionable details
- SHALL include problem type URI for error categorization

#### Scenario: Routing error for unknown GTS type

- **WHEN** client sends request to `/api/analytics/v1/gts/{unknown-type-id}`
- **THEN** system returns HTTP 404
- **AND** response body follows RFC 7807 format
- **AND** response includes type, title, status, detail, instance, trace_id
- **AND** detail explains which GTS type is unknown
- **AND** trace_id allows correlation with logs

#### Scenario: Authentication error for invalid JWT

- **WHEN** client sends request with invalid JWT signature
- **THEN** middleware validates JWT
- **AND** validation fails
- **THEN** system returns HTTP 401
- **AND** response follows RFC 7807 format
- **AND** detail explains JWT validation failure
- **AND** request does not reach routing layer

#### Scenario: Authorization error for read-only entity

- **WHEN** client attempts PUT to file-provisioned entity
- **THEN** system checks entity source
- **AND** detects read-only status
- **THEN** system returns HTTP 403
- **AND** response follows RFC 7807 format
- **AND** detail explains entity is read-only
- **AND** response includes entity identifier

#### Scenario: Validation error for invalid OData query

- **WHEN** client sends GET with `$filter=entity/unsupported_field eq 'value'`
- **THEN** query validator checks field against indexed fields
- **AND** field not in indexed list
- **THEN** system returns HTTP 400
- **AND** response follows RFC 7807 format
- **AND** response includes list of available indexed fields
- **AND** prevents full table scan

#### Scenario: Service error for unavailable domain feature

- **WHEN** routing layer forwards request to domain feature
- **AND** domain feature is unavailable or returns error
- **THEN** system returns HTTP 503
- **AND** response follows RFC 7807 format
- **AND** detail indicates temporary unavailability
- **AND** suggests retry strategy

---

### Requirement: Comprehensive Integration Testing with Mock Domain Features

The system SHALL provide comprehensive end-to-end integration tests covering all three implemented requirements (routing, middleware, tolerant-reader) using mock domain features. Tests MUST validate all acceptance criteria from Feature DESIGN.md Section F, MUST cover all edge cases, and MUST meet performance targets (routing <1ms, 1000 concurrent requests).

**ID**: `fdd-analytics-feature-gts-core-req-e2e-testing`

**Normative Requirements**:
- SHALL implement mock domain feature for realistic testing
- MUST cover all testing scenarios from Section F
- SHALL validate all acceptance criteria for three requirements
- MUST test all edge cases (malformed input, empty routing, errors)
- SHALL validate performance targets (routing <1ms, concurrency)
- MUST achieve >90% test coverage for gts_core module

#### Scenario: End-to-end registration with mock domain feature

- **WHEN** test registers GTS type via POST `/api/analytics/v1/gts`
- **THEN** routing layer extracts type from identifier
- **AND** routes request to registered mock domain feature
- **AND** mock feature receives SecurityCtx with tenant isolation
- **AND** mock feature processes create operation
- **AND** response processor applies Tolerant Reader pattern
- **THEN** client receives valid response
- **AND** response matches expected schema
- **AND** system fields are server-generated
- **AND** secrets are filtered from response

#### Scenario: OData query routing with complex filter

- **WHEN** test sends GET with `$filter=entity/name eq 'test' and entity/age gt 18`
- **THEN** middleware parses OData parameters
- **AND** query validator checks fields against indexed list
- **AND** routing layer forwards to correct mock feature
- **AND** mock feature receives parsed OData params
- **THEN** response includes filtered results
- **AND** pagination metadata is correct
- **AND** $count reflects total matching entities

#### Scenario: Multi-feature metadata aggregation

- **WHEN** test requests GET `/$metadata`
- **THEN** system aggregates metadata from all registered mock features
- **AND** generates OData JSON CSDL v4.01
- **AND** includes Capabilities vocabulary annotations
- **THEN** response is valid OData metadata
- **AND** all entity types from mock features included
- **AND** FilterRestrictions, SortRestrictions, SelectSupport present

#### Scenario: Routing performance validation

- **WHEN** test measures routing decision time for 1000 requests
- **THEN** average routing time is <1ms per request
- **AND** uses O(1) hash table lookup
- **AND** no routing errors occur
- **THEN** performance target met
- **AND** fair distribution to mock features verified

#### Scenario: Concurrent request handling

- **WHEN** test sends 1000 concurrent requests to various GTS types
- **THEN** all requests are routed correctly
- **AND** no race conditions occur
- **AND** SecurityCtx properly isolated per request
- **AND** no memory leaks detected
- **THEN** all 1000 responses valid
- **AND** fair distribution to mock features
- **AND** tenant isolation maintained

#### Scenario: JWT validation with expired token

- **WHEN** test sends request with expired JWT
- **THEN** middleware validates JWT
- **AND** detects expiration
- **THEN** returns HTTP 401 with RFC 7807 response
- **AND** detail explains token expiration
- **AND** request blocked before routing

#### Scenario: SecurityCtx injection verification

- **WHEN** test sends authenticated request with tenant_id claim
- **THEN** middleware extracts tenant_id from JWT
- **AND** creates SecurityCtx with tenant isolation
- **AND** routing forwards request to mock feature
- **THEN** mock feature receives valid SecurityCtx
- **AND** tenant_id matches JWT claim
- **AND** downstream isolation enforced

#### Scenario: Tolerant Reader field handling validation

- **WHEN** test POSTs entity with system fields in body
- **THEN** response processor ignores client-provided id, type, tenant
- **AND** generates proper values server-side
- **WHEN** test GETs entity with secrets in entity object
- **THEN** response processor filters api_key and credentials
- **AND** computed fields (asset_path) are injected
- **WHEN** test PATCHes with operations on /id path
- **THEN** validation rejects non-/entity/* paths
- **AND** returns HTTP 400 with clear error

#### Scenario: Edge case - Malformed GTS identifier

- **WHEN** test sends request with malformed identifier (invalid format)
- **THEN** GTS identifier parser rejects input
- **AND** returns HTTP 400 with RFC 7807 response
- **AND** detail explains expected identifier format
- **AND** provides example of valid identifier

#### Scenario: Edge case - Empty routing table

- **WHEN** test initializes GTS Core with no registered features
- **AND** sends request to any GTS type
- **THEN** routing table returns no match
- **AND** returns HTTP 404 with RFC 7807 response
- **AND** detail explains no features registered
- **AND** suggests registering domain feature

#### Scenario: Edge case - Feature error propagation

- **WHEN** mock domain feature configured to return error
- **AND** test sends request routed to that feature
- **THEN** routing forwards to mock feature
- **AND** mock feature returns error
- **THEN** GTS Core propagates error correctly
- **AND** error format preserved (RFC 7807)
- **AND** trace_id maintained across layers

#### Scenario: Edge case - Very long identifier

- **WHEN** test sends request with GTS identifier >500 chars
- **THEN** parser validates identifier length
- **AND** accepts if within limits (e.g., <1000 chars)
- **OR** rejects if exceeds limits
- **AND** returns appropriate error response

#### Scenario: Edge case - Special characters in identifier

- **WHEN** test sends request with special chars in GTS identifier
- **THEN** parser validates character set
- **AND** accepts valid GTS identifier characters
- **AND** rejects invalid characters
- **THEN** response explains validation failure if rejected
- **OR** routes correctly if valid
