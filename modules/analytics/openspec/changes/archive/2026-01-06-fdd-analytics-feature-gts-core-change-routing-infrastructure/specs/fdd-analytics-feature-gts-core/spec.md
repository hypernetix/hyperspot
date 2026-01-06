# GTS Core - Routing Infrastructure (Delta)

## ADDED Requirements

### Requirement: Thin Routing Layer

The system SHALL implement a thin routing layer that routes GTS API requests to domain-specific features based on GTS type patterns. The routing layer MUST provide O(1) lookup performance using hash table matching and MUST NOT contain any database layer or domain-specific business logic.

**Normative Requirements**:
- SHALL route requests based on GTS type extracted from identifier
- MUST achieve O(1) lookup performance using hash table
- MUST NOT contain database queries or domain business logic
- SHALL return HTTP 404 for unknown GTS types
- SHALL forward requests to domain feature with SecurityCtx

#### Scenario: Routing table lookup with valid type

- **WHEN** system receives GTS request with identifier `gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.sales.v1`
- **THEN** system extracts type `gts.hypernetix.hyperspot.ax.query.v1~`
- **THEN** system looks up type in routing table hash map
- **THEN** system selects `feature-query-definitions` as handler
- **THEN** system forwards request to domain feature

#### Scenario: GTS identifier parsing

- **WHEN** system receives identifier `gts.vendor.pkg.ns.type.v1~instance.v1`
- **THEN** parser extracts base type as `gts.vendor.pkg.ns.type.v1~`
- **THEN** parser handles named instances (e.g., `my-instance.v1`)
- **THEN** parser handles UUID instances (e.g., `550e8400-e29b-41d4-a716-446655440000.v1`)

#### Scenario: Unknown GTS type

- **WHEN** system receives GTS request with type not in routing table
- **THEN** system returns HTTP 404 status
- **THEN** response includes error message indicating unknown type
- **THEN** response does not forward to any domain feature

#### Scenario: Routing performance

- **WHEN** routing table contains 100+ GTS type patterns
- **THEN** routing decision completes in <1ms per request
- **THEN** lookup time remains constant (O(1)) regardless of table size
- **THEN** concurrent requests handled without routing errors

#### Scenario: End-to-end request routing

- **WHEN** client sends POST request to `/api/analytics/v1/gts`
- **THEN** system extracts GTS type from request body
- **THEN** system routes to correct domain feature based on type
- **THEN** domain feature processes request
- **THEN** system returns response from domain feature to client
