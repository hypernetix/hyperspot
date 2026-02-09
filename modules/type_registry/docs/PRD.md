# PRD

## 1. Overview

**Purpose**: Type Registry provides GTS schema storage and resolution for LLM Gateway tool definitions.

Type Registry is a schema catalog that stores GTS (Generic Type System) schemas for function/tool definitions. LLM Gateway queries the registry to resolve tool schema references before sending requests to providers. This enables consumers to reference tools by ID rather than embedding full schemas in every request.

The registry supports both single and batch schema lookups for efficient tool resolution when multiple tools are used in a request.

**Target Users**:
- **LLM Gateway** - Primary consumer for tool schema resolution

**Key Problems Solved**:
- **Schema management**: Centralized storage for tool/function schemas
- **Reference resolution**: Convert schema IDs to full GTS schemas
- **Batch lookup**: Efficient resolution of multiple tools per request

**Success Criteria**:
- All scenarios (S1-S2) implemented and operational
- Schema resolution latency < 10ms P99
- Consistent schema ID format enforced

**Capabilities**:
- Get schema by ID
- Batch get schemas
- Schema ID validation

## 2. Actors

### 2.1 Human Actors

<!-- No direct human actors for LLM Gateway scope -->

### 2.2 System Actors

#### LLM Gateway

**ID**: `cpt-cf-type-registry-actor-llm-gateway`

**Role**: Resolves tool schema references to full GTS schemas before provider calls.

## 3. Functional Requirements

#### Get Schema by ID

- [ ] `p1` - **ID**: `cpt-cf-type-registry-fr-get-schema-v1`

The system must resolve a schema ID to full GTS schema for LLM Gateway tool resolution.

**Actors**: `cpt-cf-type-registry-actor-llm-gateway`

#### Batch Get Schemas

- [ ] `p1` - **ID**: `cpt-cf-type-registry-fr-batch-get-schemas-v1`

The system must resolve multiple schema IDs in a single request for efficient multi-tool resolution.

**Actors**: `cpt-cf-type-registry-actor-llm-gateway`

#### Schema ID Validation

- [ ] `p1` - **ID**: `cpt-cf-type-registry-fr-validate-schema-id-v1`

The system must validate schema ID format before lookup.

**Actors**: `cpt-cf-type-registry-actor-llm-gateway`

## 4. Use Cases

#### UC-001: Get Schema by ID

- [ ] `p1` - **ID**: `cpt-cf-type-registry-usecase-get-schema-v1`
**Actor**: `cpt-cf-type-registry-actor-llm-gateway`

**Preconditions**: Schema exists in registry.

**Flow**:
1. LLM Gateway sends get_schema(schema_id)
2. Type Registry validates schema ID format
3. Type Registry looks up schema
4. Type Registry returns GTS schema

**Postconditions**: Schema returned or error.

**Acceptance criteria**:
- Returns schema_not_found if ID does not exist
- Returns invalid_schema_id if format is wrong
- Schema ID format: `gts.x.core.faas.func.v1~<vendor>.<app>.<namespace>.<func_name>.v1`

#### UC-002: Batch Get Schemas

- [ ] `p1` - **ID**: `cpt-cf-type-registry-usecase-batch-get-schemas-v1`
**Actor**: `cpt-cf-type-registry-actor-llm-gateway`

**Preconditions**: At least one schema ID provided.

**Flow**:
1. LLM Gateway sends get_schemas([schema_id, ...])
2. Type Registry validates all schema IDs
3. Type Registry looks up all schemas
4. Type Registry returns array of GTS schemas

**Postconditions**: Schemas returned (partial success supported).

**Acceptance criteria**:
- Single request for multiple tools
- Partial success: returns found schemas, errors for missing
- More efficient than multiple single lookups

## 5. Non-functional requirements

#### N/A

- [ ] `p1` - **ID**: `cpt-cf-type-registry-nfr-na`

<!-- NFRs to be defined later -->
