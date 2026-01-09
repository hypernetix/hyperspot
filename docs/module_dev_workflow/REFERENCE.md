# Module Development Reference

This document provides reference material for module development, including terminology, document templates, and directory structure.

> **Document Audience:** Human developers and AI agents. Provides reference material for understanding terminology and document formats.

## Index

- [Terminology](#terminology)
  - [Implementation Phases](#implementation-phases)
  - [Requirements](#requirements)
  - [Scenarios](#scenarios)
- [Directory Structure](#directory-structure)
- [Document Formats](#document-formats)
  - [DESIGN.md](#designmd)
  - [FEATURE.md](#featuremd)
  - [IMPLEMENTATION\_PLAN.md](#implementation_planmd)
  - [CHANGELOG.md](#changelogmd)
- [OpenSpec Specifications](#openspec-specifications)

---

## Terminology

| Term | Format | Description | Example |
|------|--------|-------------|---------|
| **Implementation Phase** | `#{module}/P{N}` | Incremental delivery milestone. Phases group related features for staged rollout. | `#oagw/P1`, `#typereg/P2` |
| **Global Requirement** | `#{name}` | Project-wide requirement from `docs/REQUIREMENTS.md`. Referenced by module requirements, not directly by scenarios. | `#tenant-isolation`, `#rbac` |
| **Feature** | `{feature-name}/` | Collection of related requirements stored in `docs/features/{feature-name}/FEATURE.md`. Usually maps 1:1 to an OpenSpec specification and implemented via 1-10 OpenSpec changes. | `request-forward/`, `auth/` |
| **Module Requirement** | `#{module}/{name}` | Module-specific requirement defined in a FEATURE.md file. Uses RFC 2119 language (SHALL/SHOULD/MAY). Each OpenSpec change implements 1-5 requirements. | `#oagw/request-forward`, `#typereg/entity-reg` |
| **Scenario** | *(heading text)* | Concrete use case with WHEN/THEN/AND structure that verifies a module requirement. Lives in OpenSpec specs. | "Forward request to upstream" |

### Design vs Implementation Artifacts

```
modules/{module}/
├── docs/                          # DESIGN ARTIFACTS (what to build)
│   ├── DESIGN.md                  # Module architecture + phases
│   ├── IMPLEMENTATION_PLAN.md     # Phase → Feature → Requirement checklist
│   └── features/{feature}/        
│       └── FEATURE.md             # Feature description + requirements
│
└── openspec/                      # IMPLEMENTATION ARTIFACTS (how it's built)
    ├── specs/{feature}/spec.md    # Verified scenarios (references FEATURE.md)
    └── changes/                   # Active implementation work
```

### Feature ↔ OpenSpec Verification Loop

| Design (FEATURE.md) | Implementation (spec.md) |
|---------------------|--------------------------|
| Defines **what** the feature does | Defines **how it's tested** |
| Contains requirements (`#{module}/req`) | Scenarios reference requirements |
| Written before implementation | Updated after implementation verified |

### Relationships

```mermaid
graph TD
    subgraph "Design Artifacts (docs/)"
        OD["Overall Design<br/>(DESIGN.md)"]
        GR["Global Requirement<br/>(#tenant-isolation, #rbac...)"]
        PH["Phase<br/>(#{module}/P{N})"]
        FT["Feature<br/>(features/{name}/FEATURE.md)"]
        MR["Module Requirement<br/>(#{module}/name)"]
    end

    subgraph "Implementation Artifacts (openspec/, src/)"
        SP["OpenSpec Spec<br/>(specs/{feature}/spec.md)"]
        SC["Scenario<br/>(WHEN/THEN/AND)"]
        CH["OpenSpec Change<br/>(changes/{name}/)"]
        CD["Code<br/>(src/)"]
    end

    OD -->|"defines"| PH
    GR -->|"referenced by"| MR
    FT -->|"contains"| MR
    FT -->|"belongs to"| PH
    FT -->|"1:1 maps to"| SP
    CH -->|"implements 1-5"| MR
    FT -->|"via 1-10" | CH
    SC -->|"verifies"| MR
    SP -->|"contains"| SC
    CH -->|"produces"| CD
```

### Implementation Phases

Phases are delivery milestones for incremental development.

**Example phases for Outbound API Gateway module:**
- `#oagw/P1`: Core Functionality
- `#oagw/P2`: Request Forwarding with Advanced Features
- `#oagw/P3`: Monitoring and Analytics

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
1. **Global Requirements** — project-wide, in `docs/REQUIREMENTS.md` (e.g., `#tenant-isolation`, `#rbac`)
2. **Module Requirements** — module-specific, defined in `modules/{module}/docs/features/{feature}/FEATURE.md` files (e.g., `#oagw/request-forward`)

**Common global requirements:**
- `#tenant-isolation`
- `#rbac`
- `#logging`
- `#error-handling`
- `#traceability`

### Features

Features are **collections of related requirements** stored in `docs/features/{feature-name}/FEATURE.md`:

- Each feature contains 1+ module requirements
- Usually maps 1:1 to an OpenSpec specification (`openspec/specs/{feature}/spec.md`)
- Usually implemented via 1-10 OpenSpec changes
- Each OpenSpec change implements 1-5 requirements

**Feature structure:**
```markdown
# {Feature Name}

## Overview
[Brief description of what this feature delivers]

## Requirements

### #{module}/{requirement-name}: [Title]
The system SHALL [requirement description].

**Phase:** #{module}/P{N}
**References:** #global-req (if applicable)

## Implementation Approach
[High-level technical approach]
```

### Scenarios

Scenarios are **concrete use cases** that verify module requirements using WHEN/THEN/AND format:

```markdown
#### Forward request to upstream
Verifies: #oagw/request-forward
- **WHEN** client sends POST /gateway/forward with valid auth
- **THEN** system validates access by role and tenant
- **AND** request is forwarded to configured upstream
- **AND** response is returned within timeout
- **AND** trace ID is included in headers
```

> **Note:** Scenarios reference **module requirements only**. Module requirements reference global requirements in their definition (e.g., `#oagw/request-forward` might reference `#tenant-isolation`, `#rbac`, `#traceability` in its FEATURE.md).

- Scenarios don't have IDs — the heading is the name
- Each scenario should reference the module requirement(s) it verifies
- Create separate scenarios for success, error, and edge cases

---

## Directory Structure

```
hyperspot/
├── docs/
│   ├── REQUIREMENTS.md                    # Global requirements (#tenant-isolation, #rbac...)
│   └── module_dev_workflow/               # Module development workflow
│       ├── README.md                      # Workflow steps (main doc)
│       ├── REFERENCE.md                   # This document
│       ├── ROADMAP.md                     # Future improvements
│       └── prompts/                       # AI prompt templates
│           ├── create_design.md
│           ├── create_feature.md
│           ├── create_implementation_plan.md
│           ├── validate_design_docs.md
│           ├── verify_specs_vs_requirements.md
│           ├── verify_code_vs_specs_and_requirements.md
│           └── verify_code_vs_design.md
│
├── modules/{module}/
│   ├── src/                               # Code
│   ├── docs/
│   │   ├── DESIGN.md                      # Architecture + phases (#{module}/P1, #{module}/P2...)
│   │   ├── IMPLEMENTATION_PLAN.md         # Phase → Feature → Requirement checklist
│   │   ├── CHANGELOG.md                   # Change history with requirement references
│   │   ├── features/                      # Feature definitions
│   │   │   └── {feature-name}/
│   │   │       └── FEATURE.md             # Feature description + requirements
│   │   └── verification/                  # Implementation verification reports
│   │       └── {change-name}/             # Per-change verification reports
│   │           ├── specs_vs_requirements.md
│   │           ├── code_vs_specs_and_requirements.md
│   │           └── code_vs_design.md
│   └── openspec/                          # Module-specific OpenSpec
│       ├── AGENTS.md                      # AI instructions (from openspec init)
│       ├── project.md                     # Module context
│       ├── specs/                         # Current module specs (mirrors features/)
│       │   └── {feature-name}/
│       │       └── spec.md                # Verified scenarios for feature requirements
│       └── changes/
│           ├── {change-name}/             # Active changes (implements requirements from FEATURE.md)
│           │   ├── proposal.md
│           │   ├── tasks.md
│           │   ├── design.md              # (optional)
│           │   └── specs/
│           │       └── {feature-name}/
│           │           └── spec.md        # Deltas
│           └── archive/                   # Completed changes
```

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

### Phase #{module}/P1: [Phase Name]
[Description of what's included in this phase — reference requirements]

### Phase #{module}/P2: [Phase Name]
[Description of what's included in this phase — reference requirements]

## Technical Decisions
[Key architectural and technical decisions]
```

**Cross-References:** Reference requirement IDs (`#{module}/{name}`, `#{name}`) in:
- Implementation Phases (which features/requirements each phase delivers)
- Components (which requirements each component supports)

**When to Update:**
- During initial module design
- When adding features that change architecture or add new components
- When modifying data flow or integration points

---

### FEATURE.md

**Location:** `modules/{module}/docs/features/{feature-name}/FEATURE.md`

**Purpose:** Defines a feature: what it delivers and its requirements (using RFC 2119 language).

**Structure:**
```markdown
# {Feature Name}

## Overview
[Brief description of what this feature delivers]

## Requirements

### #{module}/{requirement-name}: [Title]

The system SHALL [requirement description].

**Details:**
- [Specific detail 1]
- [Specific detail 2]

**Phase:** #{module}/P{N}
**References:** #global-req-1, #global-req-2 (if applicable)
**Rationale:** [Why this requirement exists]

### #{module}/{another-requirement}: [Title]
...

## Implementation Approach
[High-level technical approach — references DESIGN.md architecture]
```

**When to Update:**
- When creating a new feature
- When adding new requirements to an existing feature
- When modifying requirement scope or details

---

### IMPLEMENTATION_PLAN.md

**Location:** `modules/{module}/docs/IMPLEMENTATION_PLAN.md`

**Purpose:** Trackable checklist organized by phase → feature → requirements. Shows implementation progress.

**Structure (phased):**
```markdown
# {Module Name} - Implementation Plan

## Phase #{module}/P1: [Phase Name]

**Goal:** [What this phase achieves]

- [ ] **{feature-name}** — [Brief description]
  - [ ] #{module}/{req-1}: [Requirement title]
  - [ ] #{module}/{req-2}: [Requirement title]
- [x] **{completed-feature}** — [Brief description]
  - [x] #{module}/{req-3}: [Requirement title]

## Phase #{module}/P2: [Phase Name]

**Goal:** [What this phase achieves]

- [ ] **{another-feature}** — [Brief description]
  - [ ] #{module}/{req-4}: [Requirement title]
```

**Structure (simple module, no phases):**
```markdown
# {Module Name} - Implementation Plan

- [ ] **{feature-name}** — [Brief description]
  - [ ] #{module}/{req-1}: [Requirement title]
  - [ ] #{module}/{req-2}: [Requirement title]
```

**When to Update:**
- During initial module design (create full plan from FEATURE.md files)
- When adding new features
- As requirements are implemented (check off items)

**IMPLEMENTATION_PLAN vs OpenSpec Tasks:**

| Aspect | IMPLEMENTATION_PLAN.md | OpenSpec tasks.md |
|--------|------------------------|-------------------|
| **Granularity** | Features → Requirements | Tasks (granular, actionable) |
| **Purpose** | Track what to build | Track how to build |
| **Lifecycle** | Lives throughout module development | Created per change, archived after |
| **Example** | `request-forward` → `#oagw/timeout` | "Implement timeout handler" |

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
- [New features] — Implements #{module}/req-name

### Changed
- [Changes to existing functionality] — Updates #{module}/another-req

### Deprecated
- [Soon-to-be removed features]

### Removed
- [Removed features]

### Fixed
- [Bug fixes] — Fixes #{module}/bug-fix-req

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

## OpenSpec Specifications

**Location:** `modules/{module}/openspec/specs/{capability}/spec.md`

**Purpose:** Document the current state of implemented features with verified scenarios. Represents the **source of truth** for what's built and working.

**Structure:**
```markdown
# {Module Name} / {Capability}

## Requirement: #{module}/{name} - [Requirement Name]
[Brief requirement description using SHALL/SHOULD/MAY]
References: #global-req-1, #global-req-2

### Scenario: [Scenario name]
Verifies: #{module}/{name}
- **WHEN** [action/trigger]
- **THEN** [expected result]
- **AND** [additional verification]

### Scenario: [Another scenario]
Verifies: #{module}/{another-name}
- **WHEN** [action]
- **THEN** [result]
```

**When to Update:**
- Automatically updated when archiving OpenSpec changes
- Specs reflect only **implemented and verified** functionality
