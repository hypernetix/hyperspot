# GTS Core - Response Processing (Delta)

## ADDED Requirements

### Requirement: Tolerant Reader Pattern for Field Semantics

The system SHALL implement the Tolerant Reader pattern to handle field semantics across create, read, and update operations. The system MUST distinguish between client-provided fields, server-managed fields (read-only), computed fields (response-only), and never-returned fields (secrets/credentials).

**Normative Requirements**:
- SHALL categorize fields into: client-provided, server-managed, computed, secrets
- MUST ignore client attempts to set server-managed fields (id, type, registered_at, tenant)
- SHALL omit secret fields (credentials, API keys) from all responses
- MUST add computed fields (asset_path, etc.) to responses
- SHALL restrict JSON Patch operations to /entity/* paths only
- MUST support OData $select for field projection

#### Scenario: Client cannot override system fields

- **WHEN** client sends POST request with `{id: "custom", type: "custom", tenant: "custom", entity: {...}}`
- **THEN** Tolerant Reader processes request
- **THEN** system ignores id, type, tenant from request body
- **THEN** system generates id from entity structure
- **THEN** system derives type from id
- **THEN** system sets tenant from JWT claims
- **THEN** response includes server-generated values, not client values

#### Scenario: Secrets not returned in responses

- **WHEN** entity stored with `{entity: {api_key: "secret123", name: "Test"}}`
- **THEN** client sends GET request for entity
- **THEN** response processor filters fields
- **THEN** response excludes api_key field
- **THEN** response includes name field
- **THEN** credentials and secrets never exposed to client

#### Scenario: PATCH operations restricted to entity paths

- **WHEN** client sends PATCH with `[{op: "replace", path: "/entity/name", value: "New"}]`
- **THEN** system validates path starts with /entity/
- **THEN** operation allowed and applied

- **WHEN** client sends PATCH with `[{op: "replace", path: "/id", value: "new-id"}]`
- **THEN** system validates path
- **THEN** path does not start with /entity/
- **THEN** system returns HTTP 400 with error
- **THEN** operation rejected

#### Scenario: Computed fields added on read

- **WHEN** entity stored without asset_path
- **THEN** client sends GET request
- **THEN** system computes asset_path from id and type
- **THEN** system injects asset_path into response
- **THEN** response includes both stored and computed fields
- **THEN** computed fields not persisted to database

#### Scenario: Field projection with $select parameter

- **WHEN** client sends GET with `$select=id,entity/name,entity/created_at`
- **THEN** OData parser extracts select fields
- **THEN** response processor applies field projection
- **THEN** response includes only: id, entity.name, entity.created_at
- **THEN** response excludes all other fields
- **THEN** secrets still filtered even if selected

#### Scenario: End-to-end field handling

- **WHEN** client POSTs `{id: "override", entity: {name: "Test", api_key: "secret"}}`
- **THEN** Tolerant Reader ignores id override
- **THEN** system generates proper id
- **THEN** entity stored with api_key
- **WHEN** client GETs entity with $select=id,entity/name
- **THEN** response includes: id (generated), entity.name
- **THEN** response excludes: entity.api_key (secret), registered_at (not selected)
- **THEN** field semantics properly enforced
