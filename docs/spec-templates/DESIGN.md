# Technical Design — {Module Name}

<!--
=============================================================================
TECHNICAL DESIGN DOCUMENT
=============================================================================
PURPOSE: Define HOW the system is built — architecture, components, APIs,
data models, and technical decisions that realize the requirements.

DESIGN IS PRIMARY: DESIGN defines the "what" (architecture and behavior).
ADRs record the "why" (rationale and trade-offs) for selected design
decisions; ADRs are not a parallel spec, it's a traceability artifact.

SCOPE:
  ✓ Architecture overview and vision
  ✓ Design principles and constraints
  ✓ Component model and interactions
  ✓ API contracts and interfaces
  ✓ Data models and database schemas
  ✓ Technology stack choices

NOT IN THIS DOCUMENT (see other templates):
  ✗ Requirements → PRD.md
  ✗ Detailed rationale for decisions → ADR/
  ✗ Step-by-step implementation flows → features/

STANDARDS ALIGNMENT:
  - IEEE 1016-2009 (Software Design Description)
  - IEEE 42010 (Architecture Description — viewpoints, views, concerns)
  - ISO/IEC 15288 / 12207 (Architecture & Design Definition processes)

ARCHITECTURE VIEWS (per IEEE 42010):
  - Context view: system boundaries and external actors
  - Functional view: components and their responsibilities
  - Information view: data models and flows
  - Deployment view: infrastructure topology

DESIGN LANGUAGE:
  - Be specific and clear; no fluff, bloat, or emoji
  - Reference PRD requirements using `fdd-{module}-req-{slug}` IDs
  - Reference ADR documents using `fdd-{module}-adr-{slug}` IDs
=============================================================================
-->

## 1. Architecture Overview

### 1.1 Architectural Vision

{2-3 paragraphs: Technical approach, key decisions, design philosophy. How does this architecture satisfy the requirements?}

### 1.2 Architecture Drivers

Requirements that significantly influence architecture decisions.

#### Functional Drivers

| Requirement | Design Response |
|-------------|-----------------|
| `fdd-{module}-req-{slug}` | {How architecture addresses this requirement} |

#### NFR Allocation

This table maps non-functional requirements from PRD to specific design/architecture responses, demonstrating how quality attributes are realized.

| NFR ID | NFR Summary | Allocated To | Design Response | Verification Approach |
|--------|-------------|--------------|-----------------|----------------------|
| `fdd-{module}-req-{nfr-slug}` | {Brief NFR description} | {Component/layer/mechanism} | {How this design element realizes the NFR} | {How compliance is verified} |

### 1.3 Architecture Layers

{Add architecture diagram here: Mermaid or ASCII}

| Layer | Responsibility | Technology |
|-------|---------------|------------|
| Presentation | {description} | {tech} |
| Application | {description} | {tech} |
| Domain | {description} | {tech} |
| Infrastructure | {description} | {tech} |

## 2. Principles & Constraints

### 2.1 Design Principles

#### {Principle Name}

- [ ] `p2` - **ID**: `fdd-{module}-design-{slug}`

{Description of the principle and why it matters for this system.}

**ADRs**: `fdd-{module}-adr-{slug}`

### 2.2 Constraints

#### {Constraint Name}

- [ ] `p2` - **ID**: `fdd-{module}-design-{slug}`

{Description of the constraint (technical, regulatory, organizational) and its impact on design.}

**ADRs**: `fdd-{module}-adr-{slug}`

## 3. Technical Architecture

### 3.1 Domain Model

**Technology**: {GTS, Rust structs},

**Location**: [{domain-model-file}]({path/to/domain-model})

**Core Entities**:

| Entity | Description | Schema |
|--------|-------------|--------|
| {EntityName} | {Purpose} | [{file}]({path}) |

**Relationships**:
- {Entity1} → {Entity2}: {Relationship description}

### 3.2 API Contracts

**Technology**: {REST/OpenAPI | GraphQL | gRPC | etc.}

**Location**: [{api-spec-file}]({path/to/api-spec})

**Endpoints Overview**:

| Method | Path | Description | Stability |
|--------|------|-------------|-----------|
| `{METHOD}` | `{/path}` | {Description} | {stable/unstable} |


### 3.3 Module Dependencies

{Internal module dependencies within the platform. All inter-module communication goes through versioned contracts, SDK clients, or plugin interfaces — never through internal types.}

| Dependency Module | Interface Used | Purpose |
|-------------------|---------------|---------|----------|
| {module_name} | {contract / SDK client / plugin} | {Why this module is needed} |

**Dependency Rules** (per project conventions):
- No circular dependencies
- Always use SDK modules for inter-module communication
- No cross-category sideways deps except through contracts
- Only integration/adapter modules talk to external systems
- `SecurityContext` must be propagated across all in-process calls

### 3.4 External Dependencies

External systems, databases, and third-party services this module interacts with. Define protocols, data formats, and integration points.

#### {External System / Database / Service Name}

- [ ] `p2` - **ID**: `fdd-{module}-design-ext-{slug}`

**Type**: {Database | External API | Message Broker | Object Storage | etc.}

**Direction**: {inbound | outbound | bidirectional}

**Protocol / Driver**: {HTTP REST | gRPC | SeaORM/SQL | S3 API | etc.}

**Data Format**: {JSON | Protocol Buffers | SQL | binary | etc.}

**Compatibility**: {Versioning / backward compatibility guarantees}

**References**: Links to PRD or Public Library Interfaces

### 3.5 Sequences & Interactions

```mermaid
sequenceDiagram
    User ->> System: Request
    System ->> Module B: Call
    System ->> Database: Query
    Database -->> System: Result
    System -->> User: Response
```

**Key Flows**: Reference use cases from PRD via FDD IDs.

### 3.6 Database Schema

#### Table: {name}

**ID**: `fdd-{module}-design-{slug}`

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| {col} | {type} | {PK/FK/NOT NULL/etc.} | {description} |

**Indexes**: {Index definitions}

**Notes**: {Additional constraints, triggers, etc.}

## 4. Additional context

{whatever useful additional context}

## 5. Traceability

- **PRD**: [PRD.md](./PRD.md)
- **ADRs**: [ADR/](./ADR/)
- **Features**: [features/](./features/)
