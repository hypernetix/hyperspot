# UPSTREAM_REQS — {Target Module Name}

<!--
=============================================================================
UPSTREAM REQUIREMENTS DOCUMENT
=============================================================================
PURPOSE: Optional document to capture technical requirements IMPOSED
ON this module BY upstream modules (dependencies, consumers, surrounding systems).

CONTEXT: When a module is being designed or its PRD is still emerging,
upstream modules may need specific capabilities from it. This document
captures those requirements from the perspective of upstream modules.

SCOPE:
  ✓ Requirements FROM upstream modules TO this module
  ✓ Public interfaces this module must expose
  ✓ Functional requirements imposed by upstream consumers
  ✓ Non-functional requirements from upstream dependencies
  ✓ Integration contracts this module must fulfill

STRUCTURE:
  - Each upstream module has its own section
  - Requirements are grouped by upstream module source
  - Clear ownership: each requirement states which module needs it

RELATIONSHIP TO OTHER DOCS:
  - PRD.md: Defines what THIS module does (internal perspective)
  - UPSTREAM_REQS.md: Defines what OTHERS need from THIS module (external perspective)
  - DESIGN.md: Technical implementation of both sets of requirements

USE CASES:
  - Early-stage development: upstream needs known before full PRD exists
  - API-first design: consumers define interface requirements
  - Integration planning: capture cross-module dependencies
  - Incremental development: build features driven by actual consumer needs

STANDARDS ALIGNMENT:
  - IEEE 830 / ISO/IEC/IEEE 29148:2018 (requirements specification)
  - ISO/IEC 15288 / 12207 (interface requirements)

REQUIREMENT LANGUAGE:
  - Use "MUST" or "SHALL" for mandatory requirements (implicit default)
  - Do not use "SHOULD" or "MAY" — use priority p2/p3 instead
  - Be specific and clear; no fluff, bloat, duplication, or emoji
=============================================================================
-->

# UPSTREAM: {name of the upstream module from which requirements are imposed}

## System Actor Definition

**ID**: `fdd-{target-module}-upstream-actor-{slug}`

**Role**: {Description of how the upstream module interacts with target module}
**Integration Pattern**: {e.g., REST API client, SDK consumer, Event subscriber, Direct library import}

## Functional Requirements

Requirements that the upstream module needs the target module to fulfill.

### {Requirement Name}

- [ ] `p1` - **ID**: `fdd-{target-module}-upstream-req-{slug}`

The target module **MUST** {specific capability or behavior needed by upstream}.

**Rationale**: {Why the upstream module needs this capability}
**Use Case**: {How the upstream module will use this capability}
**Integration Point**: {e.g., REST endpoint, SDK method, Event topic}

### {Another Requirement}

- [ ] `p2` - **ID**: `fdd-{target-module}-upstream-req-{slug}`

The target module **MUST** {another specific requirement}.

**Rationale**: {Why this is needed}
**Use Case**: {Usage scenario}

## Non-Functional Requirements

NFRs that the upstream module requires from the target module.

### {NFR Name}

- [ ] `p1` - **ID**: `fdd-{target-module}-upstream-nfr-{slug}`

The target module **MUST** {measurable NFR with specific thresholds}.

**Threshold**: {Quantitative target with units, e.g., "respond within 100ms at p95"}
**Rationale**: {Why this NFR is required by upstream}
**Impact on Upstream**: {What happens if this NFR is not met}

## Public Interface Requirements

Interfaces that must be exposed to the upstream module.

### {Interface Name}

- [ ] `p1` - **ID**: `fdd-{target-module}-upstream-interface-{slug}`

**Type**: {REST API | gRPC | SDK method | Event | Data format}
**Stability Required**: {stable | unstable acceptable}
**Contract**: {Detailed interface contract or link to OpenAPI spec}

**Description**: {What this interface must provide}
**Input**: {Expected input parameters/format}
**Output**: {Expected output/response}
**Error Handling**: {Required error scenarios and responses}

---

# UPSTREAM: {name of the upstream module from which requirements are imposed}

## System Actor Definition
...

## Functional Requirements
...

### {Requirement Name}
...

### {Another Requirement}
...

## Non-Functional Requirements
...

### {NFR Name}
...

## Public Interface Requirements
...

### {Interface Name}
...
