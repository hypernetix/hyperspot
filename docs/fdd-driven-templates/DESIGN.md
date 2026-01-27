# Technical Design: {MODULE NAME}

## A. Architecture Overview

### Architectural Vision

{2-3 paragraphs describing the technical approach, key architectural decisions, and design philosophy}

### Architecture drivers

#### Product requirements

#### Functional requirements

| FDD ID | Solution short description |
|--------|----------------------------|
| `fdd-{module-name}-fr-{slug}` | {short description of how to solve} |

#### Non-functional requirements

| FDD ID | Solution short description |
|--------|----------------------------|
| `fdd-{module-name}-nfr-{slug}` | {short description of how to solve} |

### Architecture Layers

<!-- TODO: Add architecture diagram (draw.io, Mermaid, or embedded image) -->

| Layer | Responsibility | Technology |
|-------|---------------|------------|
| Presentation | {description} | {tech} |
| Application | {description} | {tech} |
| Domain | {description} | {tech} |
| Infrastructure | {description} | {tech} |

## B. Principles & Constraints

### B.1: Design Principles

#### {Principle Name}

**ID**: `fdd-{module-name}-principle-{principle-slug}`

<!-- fdd-id-content -->
**ADRs**: `fdd-{module-name}-adr-{adr-slug}`

{Description of the principle and why it matters}
<!-- fdd-id-content -->

<!-- TODO: Add more design principles as needed -->

### B.2: Constraints

#### {Constraint Name}

**ID**: `fdd-{module-name}-constraint-{constraint-slug}`

<!-- fdd-id-content -->
**ADRs**: `fdd-{module-name}-adr-{adr-slug}`

{Description of the constraint and its impact}
<!-- fdd-id-content -->

<!-- TODO: Add more constraints as needed -->

## C. Technical Architecture

### C.1: Domain Model

**Technology**: {GTS}

**Location**: [{domain-model-file}]({path/to/domain-model})

**Core Entities**:
- [{EntityName}]({path/to/entity.schema}) - {Description}

**Relationships**:
- {Entity1} → {Entity2}: {Relationship description}

### C.2: Component Model

<!-- TODO: Add component diagram (draw.io, Mermaid, or ASCII) -->
```mermaid
```

**Components**:
- **{Component 1}**: {Purpose and responsibility}
- **{Component 2}**: {Purpose and responsibility}

**Interactions**:
- {Component 1} → {Component 2}: {Description of interaction}

### C.3: API Contracts

**Technology**: {REST/OpenAPI | GraphQL | gRPC | CLISPEC}

**Location**: [{api-spec-file}]({path/to/api-spec})

**Endpoints Overview**:
- `{METHOD} {/path}` - {Description}

### C.4: Interactions & Sequences

<!-- TODO: Add sequence diagram (draw.io, Mermaid, or ASCII) -->
```mermaid
```

**Use cases**: FDD ID from PRD.

**Actors**: FDD ID from PRD.

### C.5 Database schemas & tables

<!-- Keep empty if not relevant. -->

#### Table {name}

**ID**: `fdd-{module-name}-db-table-{slug}`

**Schema**

| Column | Type | Description |
|--------|------|-------------|

**PK**: {PK}

**Constraints**: {Constraints}

**Additional info**: {Additional info}

**Example**

| Col name A | B | C |
|------------|---|---|
| values     |   |   |

### C.6: Topology (optional)

Physical view, files, pods, containers, DC, virtual machines, etc.

**ID**: `fdd-{module-name}-topology-{slug}`

### C.7: Tech stack (optional)

**ID**: `fdd-{module-name}-tech-{slug}`

## D. Additional Context

**ID**: `fdd-{module-name}-design-context-{slug}`

<!-- TODO: Add any additional technical context, architect notes, rationale, etc. -->
<!-- This section is optional and not validated by FDD -->
