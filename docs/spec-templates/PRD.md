# PRD — {Module/Feature Name}

<!--
=============================================================================
PRODUCT REQUIREMENTS DOCUMENT (PRD)
=============================================================================
PURPOSE: Define WHAT the system must do and WHY — business requirements,
functional capabilities, and quality attributes.

SCOPE:
  ✓ Business goals and success criteria
  ✓ Actors (users, systems) that interact with this module
  ✓ Functional requirements (WHAT, not HOW)
  ✓ Non-functional requirements (quality attributes, SLOs)
  ✓ Scope boundaries (in/out of scope)
  ✓ Assumptions, dependencies, risks

NOT IN THIS DOCUMENT (see other templates):
  ✗ Stakeholder needs (managed at project/task level by steering committee)
  ✗ Technical architecture, design decisions → DESIGN.md
  ✗ Why a specific technical approach was chosen → ADR/
  ✗ Detailed implementation flows, algorithms → features/

STANDARDS ALIGNMENT:
  - IEEE 830 / ISO/IEC/IEEE 29148:2018 (requirements specification)
  - IEEE 1233 (system requirements)
  - ISO/IEC 15288 / 12207 (requirements definition)

REQUIREMENT LANGUAGE:
  - Use "MUST" or "SHALL" for mandatory requirements (implicit default)
  - Do not use "SHOULD" or "MAY" — use priority p2/p3 instead
  - Be specific and clear; no fluff, bloat, duplication, or emoji
=============================================================================
-->

## 1. Overview

### 1.1 Purpose

{1-2 sentences: What is this system/module and what problem does it solve?}

### 1.2 Background / Problem Statement

{2-3 paragraphs: Context, current pain points, why this capability is needed now.}

### 1.3 Goals (Business Outcomes)

- {Goal 1: measurable business outcome}
- {Goal 2: measurable business outcome}

### 1.4 Glossary

| Term | Definition |
|------|------------|
| {Term} | {Definition} |

## 2. Actors

> **Note**: Stakeholder needs are managed at project/task level by steering committee. Document **actors** (users, systems) that interact with this module.

### 2.1 Human Actors

#### {Actor Name}

**ID**: `fdd-{module}-actor-{slug}`

**Role**: {Description of what this actor does and their relationship to the system.}
**Needs**: {What this actor needs from the system.}

### 2.2 System Actors

#### {System Actor Name}

**ID**: `fdd-{module}-actor-{slug}`

**Role**: {Description of what this system actor does (external service, scheduler, etc.)}

## 3. Operational Concept & Environment

> **Note**: Project-wide runtime, OS, architecture, lifecycle policy, and integration patterns defined in root PRD. Document only module-specific deviations here. **Delete this section if no special constraints.**

### 3.1 Module-Specific Environment Constraints

{Only if this module has constraints beyond project defaults:}

- {Constraint 1, e.g., "Requires GPU acceleration for X"}
- {Constraint 2, e.g., "Incompatible with async runtime due to Y"}
- {Constraint 3, e.g., "Requires external dependency: Z library v2.0+"}

## 4. Scope

### 4.1 In Scope

- {Capability or feature that IS included}
- {Another capability}

### 4.2 Out of Scope

- {Capability explicitly NOT included in this PRD}
- {Future consideration not addressed now}

## 5. Functional Requirements

> **Testing strategy**: All requirements verified via automated tests (unit, integration, e2e) targeting 90%+ code coverage unless otherwise specified. Document verification method only for non-test approaches (analysis, inspection, demonstration).

Functional requirements define WHAT the system must do. Group by feature area or priority tier.

### 5.1 {Feature Area / Priority Tier}

#### {Requirement Name}

- [ ] `p1` - **ID**: `fdd-{module}-req-{slug}`

The system **MUST** {do something specific and verifiable}.

**Rationale**: {Why this requirement exists — business value or stakeholder need.}
**Actors**: `fdd-{module}-actor-{slug}`
**Verification Method** (optional): {Only if non-standard: analysis | inspection | demonstration | specialized test approach}
**Acceptance Evidence** (optional): {Only if non-obvious: specific test suite path, analysis report, review checklist}

## 6. Non-Functional Requirements

> **Global baselines**: Project-wide NFRs (performance, security, reliability, scalability) defined in root PRD and [guidelines/](../guidelines/). Document only module-specific NFRs here: **exclusions** from defaults or **standalone** requirements.
>
> **Testing strategy**: NFRs verified via automated benchmarks, security scans, and monitoring unless otherwise specified.

### 6.1 Module-Specific NFRs

{Only include this section if there are NFRs that deviate from or extend project defaults.}

#### {NFR Name}

- [ ] `p1` - **ID**: `fdd-{module}-req-{slug}`

The system **MUST** {measurable NFR with specific thresholds, e.g., "respond within 50ms at p95" (stricter than project default)}.

**Threshold**: {Quantitative target with units and conditions}
**Rationale**: {Why this module needs different/additional NFR}
**Verification Method** (optional): {Only if non-standard approach needed}
**Architecture Allocation**: See DESIGN.md § NFR Allocation for how this is realized

### 6.2 NFR Exclusions

{Document any project-default NFRs that do NOT apply to this module}

- {Default NFR name}: {Reason for exclusion}

## 7. Public Library Interfaces

Define the public API surface, versioning/compatibility guarantees, and integration contracts provided by this library.

### 7.1 Public API Surface

#### {Interface Name}

- [ ] `p1` - **ID**: `fdd-{module}-interface-{slug}`

**Type**: {Rust module/trait/struct | REST API | CLI | Protocol | Data format}
**Stability**: {stable | unstable | experimental}
**Description**: {What this interface provides}
**Breaking Change Policy**: {e.g., Major version bump required}

### 7.2 External Integration Contracts

Contracts this library expects from external systems or provides to downstream clients.

#### {Contract Name}

- [ ] `p2` - **ID**: `fdd-{module}-contract-{slug}`

**Direction**: {provided by library | required from client}
**Protocol/Format**: {e.g., HTTP/REST, gRPC, JSON Schema}
**Compatibility**: {Backward/forward compatibility guarantees}

## 8. Use Cases

Optional: Include when interaction flows add clarity beyond requirement statements.

#### UC: {Use Case Name}

- [ ] `p2` - **ID**: `fdd-{module}-req-{slug}`

**Actor**: `fdd-{module}-actor-{slug}`

**Preconditions**:
- {Required state before execution}

**Main Flow**:
1. {Actor action or system response}
2. {Next step}

**Postconditions**:
- {State after successful completion}

**Alternative Flows**:
- **{Condition}**: {What happens instead}

## 9. Acceptance Criteria

Business-level acceptance criteria for the PRD as a whole.

- [ ] {Testable criterion that validates a key business outcome}
- [ ] {Another testable criterion}

## 10. Dependencies

| Dependency | Description | Criticality |
|------------|-------------|-------------|
| {Service/System} | {What it provides} | {p1/p2/p3} |

## 11. Assumptions

- {Assumption about environment, users, or dependent systems}

## 12. Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| {Risk description} | {Potential impact} | {Mitigation strategy} |

## 13. Open Questions

Unresolved questions that need answers before or during implementation.

- {Question about scope, approach, or edge case}

## 14. Traceability

Links to related specification artifacts.

- **Design**: [DESIGN.md](./DESIGN.md)
- **ADRs**: [ADR/](./ADR/)
- **Features**: [features/](./features/)
