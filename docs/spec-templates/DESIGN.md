# Technical Design: {MODULE NAME}

## 1. Architecture Overview

### 1.1 Architectural Vision

{2-3 paragraphs describing the technical approach, key architectural decisions, and design philosophy}

### 1.2 Architecture Drivers

#### Product requirements

#### Functional requirements

| FDD ID | Solution short description |
|--------|----------------------------|
| `fdd-{module-name}-fr-{slug}` | {short description of how to solve} |

#### Non-functional requirements

| FDD ID | Solution short description |
|--------|----------------------------|
| `fdd-{module-name}-nfr-{slug}` | {short description of how to solve} |

### 1.3 Architecture Layers

<!-- TODO: Add architecture diagram (draw.io, Mermaid, or embedded image) -->

| Layer | Responsibility | Technology |
|-------|---------------|------------|
| Presentation | {description} | {tech} |
| Application | {description} | {tech} |
| Domain | {description} | {tech} |
| Infrastructure | {description} | {tech} |

## 2. Principles & Constraints

### 2.1: Design Principles

#### {Principle Name}

**ID**: `fdd-{module-name}-principle-{principle-slug}`

<!-- fdd-id-content -->
**ADRs**: `fdd-{module-name}-adr-{adr-slug}`

{Description of the principle and why it matters}
<!-- fdd-id-content -->

<!-- TODO: Add more design principles as needed -->

### 2.2: Constraints

#### {Constraint Name}

**ID**: `fdd-{module-name}-constraint-{constraint-slug}`

<!-- fdd-id-content -->
**ADRs**: `fdd-{module-name}-adr-{adr-slug}`

{Description of the constraint and its impact}
<!-- fdd-id-content -->

<!-- TODO: Add more constraints as needed -->

## 3. Technical Architecture

### 3.1: Domain Model

**Technology**: {GTS}

**Location**: [{domain-model-file}]({path/to/domain-model})

**Core Entities**:
- [{EntityName}]({path/to/entity.schema}) - {Description}

**Relationships**:
- {Entity1} → {Entity2}: {Relationship description}

### 3.2: Component Model

<!-- TODO: Add component diagram (draw.io, Mermaid, or ASCII) -->
```mermaid
```

**Components**:
- **{Component 1}**: {Purpose and responsibility}
- **{Component 2}**: {Purpose and responsibility}

**Interactions**:
- {Component 1} → {Component 2}: {Description of interaction}

### 3.3: API Contracts

**Technology**: {REST/OpenAPI | GraphQL | gRPC | CLISPEC}

**Location**: [{api-spec-file}]({path/to/api-spec})

**Endpoints Overview**:
- `{METHOD} {/path}` - {Description}

### 3.4: Interactions & Sequences

<!-- TODO: Add sequence diagram (draw.io, Mermaid, or ASCII) -->
```mermaid
```

**Use cases**: FDD ID from PRD.

**Actors**: FDD ID from PRD.

### 3.5: Database schemas & tables

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

### 3.6: Topology (optional)

Physical view, files, pods, containers, DC, virtual machines, etc.

**ID**: `fdd-{module-name}-topology-{slug}`

### 3.7: Tech stack (optional)

**ID**: `fdd-{module-name}-tech-{slug}`

## 4. Additional Context

**ID**: `fdd-{module-name}-design-context-{slug}`

<!-- TODO: Add any additional technical context, architect notes, rationale, etc. -->
<!-- This section is optional and not validated by FDD -->
