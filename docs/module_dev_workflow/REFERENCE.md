# Module Development Reference

This document provides reference material for module development, including ID formats, document templates, and directory structure.

For the workflow steps, see [Module Development Workflow](./README.md).

## Index

- [Module Development Reference](#module-development-reference)
  - [Index](#index)
  - [Terms and ID Formats](#terms-and-id-formats)
    - [Implementation Phases](#implementation-phases)
    - [Requirements](#requirements)
    - [Scenarios](#scenarios)
  - [Document Formats](#document-formats)
    - [DESIGN.md](#designmd)
    - [REQUIREMENTS.md](#requirementsmd)
    - [IMPLEMENTATION\_PLAN.md](#implementation_planmd)
    - [CHANGELOG.md](#changelogmd)
  - [Directory Structure](#directory-structure)
  - [OpenSpec Specifications](#openspec-specifications)

---

## Terms and ID Formats

| Term | Format | Description | Example |
|------|--------|-------------|---------|
| **Implementation Phase** | `{MODULE}-P{N}` | Incremental delivery milestone. Phases group related features for staged rollout. | `OAGW-P1`, `TYPEREG-P2` |
| **Module Requirement** | `{MODULE}-REQ{N}` | Module-specific requirement defining what the system SHALL do. | `OAGW-REQ01`, `TYPEREG-REQ1` |
| **Global Requirement** | `REQ{N}` | Project-wide requirement from `docs/REQUIREMENTS.md`. | `REQ1` (tenant isolation) |
| **Scenario** | *(heading text)* | Concrete use case with WHEN/THEN/AND structure that verifies a requirement. Lives in OpenSpec specs. | "Forward request to upstream" |

### Implementation Phases

Phases are delivery milestones for incremental development.

**Example phases for Outbound API Gateway module:**
- `OAGW-P1`: Core Functionality
- `OAGW-P2`: Request Forwarding with Advanced Features
- `OAGW-P3`: Monitoring and Analytics

**Notes:**
- Simple modules may not need phases on the first iteration
- Phases are defined in `modules/{module}/docs/DESIGN.md`
- Phases may be referenced in IMPLEMENTATION_PLAN.md and requirements

### Requirements

Requirements define **what the system SHALL do**. They use [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119) language:

| Keyword | Meaning |
|---------|---------|
| **MUST/SHALL** | Mandatory |
| **SHOULD** | Recommended |
| **MAY** | Optional |

**Two levels:**
1. **Global Requirements** — project-wide, in `docs/REQUIREMENTS.md` (e.g., `REQ1`, `REQ2`)
2. **Module Requirements** — module-specific, in `modules/{module}/docs/REQUIREMENTS.md` (e.g., `OAGW-REQ01`)

**Common global requirements:**
- `REQ1`: Tenant Isolation
- `REQ2`: Role Based Access Control
- `REQ3`: Logging
- `REQ4`: Error Handling and API Responses
- `REQ5`: Traceability

### Scenarios

Scenarios are **concrete use cases** that verify requirements using WHEN/THEN/AND format:

```markdown
#### Forward request to upstream
Verifies: OAGW-REQ01, REQ1, REQ2, REQ5
- **WHEN** client sends POST /gateway/forward with valid auth
- **THEN** system validates access by role (REQ2) and tenant (REQ1)
- **AND** request is forwarded to configured upstream
- **AND** response is returned within timeout
- **AND** trace ID is included in headers (REQ5)
```

- Scenarios don't have IDs — the heading is the name
- Each scenario should reference the requirement(s) it verifies
- Create separate scenarios for success, error, and edge cases

---

## Document Formats

### DESIGN.md

**Location:** `modules/{module}/docs/DESIGN.md`

**Purpose:** Documents the module's architecture, components, and implementation approach.

**Structure:**
```markdown
# {Module Name} - Design

## Overview
[Brief module description and purpose]

## Architecture
[High-level architecture diagram and description]

## Components
[Key components and their responsibilities — include requirement IDs]

## Data Flow
[How data flows through the module — include requirement IDs where applicable]

## Integration Points
[How this module integrates with other modules]

## Implementation Phases

### Phase {MODULE}-P1: [Phase Name]
[Description of what's included in this phase — reference requirements]

### Phase {MODULE}-P2: [Phase Name]
[Description of what's included in this phase — reference requirements]

## Technical Decisions
[Key architectural and technical decisions]
```

**Cross-References:** After requirements are defined, add requirement IDs (`{MODULE}-REQ{N}`, `REQ{N}`) to:
- Implementation Phases (which requirements each phase delivers)
- Components (which requirements each component supports)
- Data Flow steps (where applicable)

**When to Update:**
- During initial module design (iteratively with REQUIREMENTS.md)
- When adding features that change architecture or add new components
- When modifying data flow or integration points

---

### REQUIREMENTS.md

**Location:** `modules/{module}/docs/REQUIREMENTS.md`

**Purpose:** Defines what the module SHALL, SHOULD, and MAY do using RFC 2119 language.

**Structure:**
```markdown
# {Module Name} - Requirements

## {MODULE}-REQ{N}: [Requirement Name]

The system SHALL [requirement description].

**Details:**
- [Specific detail 1]
- [Specific detail 2]

**References:** REQ{X}, REQ{Y} (global requirements)

**Phase:** {MODULE}-P{N}

**Rationale:** [Why this requirement exists]
```

**Iteration Signals:** During the iterative design process, watch for:
- Requirement has no clear component owner → Add/clarify component in DESIGN.md
- Component has no requirements → Remove component or identify missing requirements
- Phase boundary feels wrong → Adjust phases based on requirement dependencies

**When to Update:**
- During initial module design (iteratively with DESIGN.md)
- When adding new system capabilities
- When modifying existing capabilities

---

### IMPLEMENTATION_PLAN.md

**Location:** `modules/{module}/docs/IMPLEMENTATION_PLAN.md`

**Purpose:** Trackable checklist of **features** to implement, organized by phase. Features are high-level deliverables, not granular tasks.

**Structure (phased):**
```markdown
# {Module Name} - Implementation Plan

## Phase {MODULE}-P1: [Phase Name]

**Goal:** [What this phase achieves]

- [ ] [Feature 1 name] ({MODULE}-REQ{X})
      Scope: [brief description of what this feature involves]
- [ ] [Feature 2 name] ({MODULE}-REQ{Y}, REQ{Z})
      Scope: [brief description]
- [x] [Completed feature] ({MODULE}-REQ{A})

## Phase {MODULE}-P2: [Phase Name]

**Goal:** [What this phase achieves]

- [ ] [Feature name] ({MODULE}-REQ{B}, REQ{C})
      Scope: [brief description]
```

**Structure (simple module, no phases):**
```markdown
# {Module Name} - Implementation Plan

- [ ] [Feature Name] ({MODULE}-REQ{X})
      Scope: contract traits, domain service, REST endpoint
- [ ] [Feature Name] ({MODULE}-REQ{Y}, REQ{Z})
      Scope: authorization checks, tenant isolation
```

**When to Update:**
- During initial module design (create full plan)
- When adding new features (add unchecked items)
- As features are completed (check off items)

**IMPLEMENTATION_PLAN vs OpenSpec Tasks:**

| Aspect | IMPLEMENTATION_PLAN.md | OpenSpec tasks.md |
|--------|------------------------|-------------------|
| **Granularity** | Features (high-level) | Tasks (granular, actionable) |
| **Purpose** | Track what to build | Track how to build each feature |
| **Lifecycle** | Lives throughout module development | Created per feature, archived after |
| **Example** | "Request Forwarding" | "Define ForwardRequest DTO", "Implement handler" |

---

### CHANGELOG.md

**Location:** `modules/{module}/docs/CHANGELOG.md`

**Purpose:** Track module evolution following [Keep A Changelog](https://keepachangelog.com/en/1.0.0/) format.

**Structure:**
```markdown
# {Module Name} - Change Log

All notable changes to this module will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added
- [New features] - Implements {MODULE}-REQ{X}

### Changed
- [Changes to existing functionality] - Updates {MODULE}-REQ{Y}

### Deprecated
- [Soon-to-be removed features]

### Removed
- [Removed features]

### Fixed
- [Bug fixes] - Fixes {MODULE}-REQ{Z}

### Security
- [Security fixes/improvements]
```

**Categories:**
- **Added**: New features
- **Changed**: Changes to existing functionality
- **Deprecated**: Soon-to-be removed features
- **Removed**: Removed features
- **Fixed**: Bug fixes
- **Security**: Security fixes or improvements

**When to Update:**
- After completing each feature implementation
- Add to `[Unreleased]` section
- Reference requirement IDs

---

## Directory Structure

```
hyperspot/
├── docs/
│   ├── REQUIREMENTS.md                    # Global requirements (REQ1, REQ2...)
│   └── module_dev_workflow/               # Module development workflow
│       ├── README.md                      # Workflow steps (main doc)
│       ├── REFERENCE.md                   # This document
│       ├── ROADMAP.md                     # Future improvements
│       └── prompts/                       # AI prompt templates
│           ├── create_design_and_requirements.md
│           ├── create_implementation_plan.md
│           └── validate_design_docs.md
│
├── modules/{module}/
│   ├── src/                               # Code
│   ├── docs/
│   │   ├── DESIGN.md                      # Architecture + phases ({MODULE}-P1, {MODULE}-P2...)
│   │   ├── IMPLEMENTATION_PLAN.md         # Feature checklist (already implemented and planned) divided by phases
│   │   ├── REQUIREMENTS.md                # Module requirements ({MODULE}-REQ1, {MODULE}-REQ2...)
│   │   └── CHANGELOG.md                   # Change history with requirement references
│   └── openspec/                          # Module-specific OpenSpec
│       ├── AGENTS.md                      # AI instructions (from openspec init)
│       ├── project.md                     # Module context
│       ├── specs/                         # Current module specs
│       │   └── {capability}/
│       │       └── spec.md                # Implemented scenarios for module requirements referenced in docs/REQUIREMENTS.md
│       └── changes/
│           ├── {change-name}/             # Active changes (change is a feature to implement from the IMPLEMENTATION_PLAN.md)
│           │   ├── proposal.md
│           │   ├── tasks.md
│           │   ├── design.md              # (optional)
│           │   └── specs/
│           │       └── {capability}/
│           │           └── spec.md        # Deltas
│           └── archive/                   # Completed changes (change is an implemented feature from the IMPLEMENTATION_PLAN.md)
```

---

## OpenSpec Specifications

**Location:** `modules/{module}/openspec/specs/{capability}/spec.md`

**Purpose:** Document the current state of implemented features with verified scenarios. Represents the **source of truth** for what's built and working.

**Structure:**
```markdown
# {Module Name} / {Capability}

## Requirement: [Requirement Name]
[Brief requirement description using SHALL/SHOULD/MAY and reference to {MODULE}-REQ{X}]

### Scenario: [Scenario name]
- **WHEN** [action/trigger]
- **THEN** [expected result]
- **AND** [additional verification]

### Scenario: [Another scenario]
- **WHEN** [action]
- **THEN** [result]
```

**When to Update:**
- Automatically updated when archiving OpenSpec changes
- Specs reflect only **implemented and verified** functionality
