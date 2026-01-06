# GTS Core - Request Middleware (Delta)

## ADDED Requirements

### Requirement: Request Processing Middleware Chain

The system SHALL implement a middleware chain that validates JWT tokens, injects SecurityCtx with tenant isolation, and parses OData query parameters before routing requests to domain features. The middleware MUST support all OData v4 query parameters including $filter, $select, $orderby, $top, $skiptoken, and $count.

**Normative Requirements**:
- SHALL validate JWT signature on all requests
- MUST extract tenant_id from JWT claims and inject into SecurityCtx
- SHALL parse OData v4 query parameters into AST
- MUST validate $filter expressions against indexed fields
- SHALL return HTTP 401 for authentication failures
- SHALL return HTTP 400 for invalid query parameters with available fields list

#### Scenario: JWT validation with invalid signature

- **WHEN** client sends request with JWT having invalid signature
- **THEN** middleware validates JWT signature
- **THEN** validation fails
- **THEN** system returns HTTP 401 status
- **THEN** request does not reach routing layer

#### Scenario: SecurityCtx injection with valid JWT

- **WHEN** client sends request with valid JWT containing tenant_id claim "acme"
- **THEN** middleware validates JWT successfully
- **THEN** middleware extracts tenant_id from claims
- **THEN** middleware creates SecurityCtx with tenant_id="acme"
- **THEN** SecurityCtx injected into request context
- **THEN** downstream handlers access SecurityCtx

#### Scenario: OData parameter parsing with complex filter

- **WHEN** client sends GET request with `$filter=entity/name eq 'test' and entity/age gt 18`
- **THEN** middleware parses $filter into AST
- **THEN** AST represents AND operation with two comparisons
- **THEN** middleware also parses $select, $orderby, $top parameters if present
- **THEN** parsed parameters available to routing layer

#### Scenario: Query optimization validation rejects unindexed field

- **WHEN** client sends request with `$filter=entity/unsupported_field eq 'value'`
- **THEN** validator checks field against indexed fields list
- **THEN** field "entity/unsupported_field" not in indexed fields
- **THEN** system returns HTTP 400 status
- **THEN** response includes list of available indexed fields
- **THEN** prevents full table scan

#### Scenario: End-to-end authenticated request with OData

- **WHEN** client sends GET with valid JWT and `$filter=entity/name eq 'test'&$select=entity/name,entity/created_at`
- **THEN** JWT validation succeeds
- **THEN** SecurityCtx injected with tenant_id
- **THEN** OData parameters parsed successfully
- **THEN** query validator accepts indexed fields
- **THEN** request routed to domain feature
- **THEN** response includes only selected fields
