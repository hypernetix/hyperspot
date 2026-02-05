# DECOMPOSITION Expert Checklist

**Artifact**: Design Decomposition (DECOMPOSITION)
**Version**: 2.0
**Last Updated**: 2025-02-03
**Purpose**: Validate quality of design decomposition into implementable work packages

---

## Referenced Standards

This checklist validates decomposition quality based on the following international standards:

| Standard | Domain | Description |
|----------|--------|-------------|
| [IEEE 1016-2009](https://standards.ieee.org/ieee/1016/4502/) | **Design Decomposition** | Software Design Descriptions — Decomposition Description viewpoint (§5.4) |
| [ISO 21511:2018](https://www.iso.org/standard/69702.html) | **Work Breakdown Structure** | WBS for project/programme management — scope decomposition, 100% rule |
| [ISO 10007:2017](https://www.iso.org/standard/70400.html) | **Configuration Management** | Configuration items, product structure, baselines |
| [ISO/IEC/IEEE 42010:2022](https://www.iso.org/standard/74393.html) | **Architecture Description** | Architecture viewpoints, model correspondences, consistency |
| [ISO/IEC/IEEE 29148:2018](https://www.iso.org/standard/72089.html) | **Requirements Traceability** | Bidirectional traceability, verification |

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Applicability Context](#applicability-context)
3. [Severity Dictionary](#severity-dictionary)
4. [Checkpointing](#checkpointing-long-reviews) — for long reviews / context limits
5. **MUST HAVE** (check in priority order):
   - [COV: Coverage](#coverage-cov) — WBS 100% Rule *(ISO 21511)*
   - [EXC: Exclusivity](#exclusivity-exc) — Mutual Exclusivity *(ISO 21511)*
   - [ATTR: Entity Attributes](#entity-attributes-attr) — Design Entity Completeness *(IEEE 1016)*
   - [LEV: Decomposition Levels](#decomposition-levels-lev) — Granularity Consistency
   - [CFG: Configuration Items](#configuration-items-cfg) — CI Selection *(ISO 10007)*
   - [TRC: Traceability](#traceability-trc) — Bidirectional Links *(ISO 29148, ISO 42010)*
   - [DEP: Dependencies](#dependencies-dep) — Dependency Graph Quality
   - [CHK: Checkbox Consistency](#checkbox-consistency-chk) — Status Integrity
   - [DOC: Deliberate Omissions](#deliberate-omissions-doc)
6. **MUST NOT HAVE**:
   - [No Implementation Details](#no-implementation-details)
   - [No Requirements Definitions](#no-requirements-definitions)
   - [No Architecture Decisions](#no-architecture-decisions)
7. [Format Validation](#format-validation)
8. [Validation Summary](#validation-summary)
9. [Reporting](#reporting)

**Review Priority**: COV → EXC → ATTR → TRC → DEP → (others)

---

## Prerequisites

Before starting the review, confirm:

- [ ] I understand this checklist validates DECOMPOSITION artifacts (design breakdown into specs)
- [ ] I have access to the source DESIGN artifact being decomposed
- [ ] I will check ALL items in MUST HAVE sections
- [ ] I will verify ALL items in MUST NOT HAVE sections
- [ ] I will document any violations found
- [ ] I will use the [Reporting](#reporting) format for output

---

## Applicability Context

**Purpose of DECOMPOSITION artifact**: Break down the overall DESIGN into implementable work packages (specs) that can be assigned, tracked, and implemented independently.

**What this checklist tests**: Quality of the decomposition itself — not the quality of requirements, design decisions, security, performance, or other concerns. Those belong in PRD and DESIGN checklists.

**Key principle**: A perfect decomposition has:
1. **100% coverage** — every design element appears in at least one spec
2. **No overlap** — no design element appears in multiple specs without clear reason
3. **Complete attributes** — every spec has identification, purpose, scope, dependencies
4. **Consistent granularity** — specs are at similar abstraction levels
5. **Bidirectional traceability** — can trace both ways between design and specs

---

## Severity Dictionary

- **CRITICAL**: Decomposition is fundamentally broken; cannot proceed to implementation.
- **HIGH**: Significant decomposition gap; should be fixed before implementation starts.
- **MEDIUM**: Decomposition improvement needed; fix when feasible.
- **LOW**: Minor improvement; optional.

---

## Checkpointing (Long Reviews)

### Checkpoint After Each Domain

After completing each expertise domain (COV, EXC, ATTR, etc.), output:
```
✓ {DOMAIN} complete: {N} items checked, {M} issues found
Issues: {list issue IDs}
Remaining: {list unchecked domains}
```

### Minimum Viable Review

If full review impossible, prioritize in this order:
1. **COV-001** (CRITICAL) — WBS 100% Rule
2. **EXC-001** (CRITICAL) — Mutual Exclusivity
3. **ATTR-001** (HIGH) — Entity Identification
4. **TRC-001** (HIGH) — Forward Traceability
5. **DOC-001** (CRITICAL) — Deliberate Omissions

Mark review as "PARTIAL" if not all domains completed.

---

# MUST HAVE

---

## COVERAGE (COV)

> **Standard**: [ISO 21511:2018](https://www.iso.org/standard/69702.html) §4.2 — WBS 100% Rule
>
> "The WBS must include 100% of the work defined by the scope and capture all deliverables."

### COV-001: Design Element Coverage (100% Rule)
**Severity**: CRITICAL
**Ref**: ISO 21511:2018 §4.2 (WBS 100% rule)

- [ ] ALL components from DESIGN are assigned to at least one spec
- [ ] ALL sequences/flows from DESIGN are assigned to at least one spec
- [ ] ALL data entities from DESIGN are assigned to at least one spec
- [ ] ALL design principles from DESIGN are assigned to at least one spec
- [ ] ALL design constraints from DESIGN are assigned to at least one spec
- [ ] No orphaned design elements (elements not in any spec)

**Verification method**: Cross-reference DESIGN IDs with DECOMPOSITION references.

### COV-002: Requirements Coverage Passthrough
**Severity**: HIGH
**Ref**: ISO/IEC/IEEE 29148:2018 §6.5 (Traceability)

- [ ] ALL functional requirements (FR) from PRD are covered by at least one spec
- [ ] ALL non-functional requirements (NFR) from PRD are covered by at least one spec
- [ ] No orphaned requirements (requirements not in any spec)

**Note**: This verifies that the decomposition covers requirements transitively through DESIGN.

### COV-003: Coverage Mapping Completeness
**Severity**: HIGH

- [ ] Each spec explicitly lists "Requirements Covered" with IDs
- [ ] Each spec explicitly lists "Design Components" with IDs
- [ ] Each spec explicitly lists "Sequences" with IDs (or "None")
- [ ] Each spec explicitly lists "Data" with IDs (or "None")
- [ ] No implicit or assumed coverage

---

## EXCLUSIVITY (EXC)

> **Standard**: [ISO 21511:2018](https://www.iso.org/standard/69702.html) §4.2 — Mutual Exclusivity
>
> "Work packages should be mutually exclusive to avoid double-counting and ambiguity."

### EXC-001: Scope Non-Overlap
**Severity**: CRITICAL
**Ref**: ISO 21511:2018 §4.2 (Mutual exclusivity)

- [ ] Specs do not overlap in scope (each deliverable assigned to exactly one spec)
- [ ] No duplicate coverage of the same design element by multiple specs without explicit reason
- [ ] Responsibility for each deliverable is unambiguous
- [ ] No "shared" scope that could cause confusion about ownership

**Verification method**: Check if any design element ID appears in multiple specs' references.

### EXC-002: Boundary Clarity
**Severity**: HIGH

- [ ] Each spec has clear "Scope" boundaries (what's in)
- [ ] Each spec has clear "Out of Scope" boundaries (what's explicitly excluded)
- [ ] Boundaries between adjacent specs are explicit and non-ambiguous
- [ ] Domain entities are assigned to single spec (or clear reason for sharing)

### EXC-003: Dependency vs Overlap Distinction
**Severity**: MEDIUM

- [ ] Dependencies (one spec uses output of another) are clearly distinct from overlaps
- [ ] Shared components are documented as dependencies, not duplicate scope
- [ ] Integration points are explicit

---

## ENTITY ATTRIBUTES (ATTR)

> **Standard**: [IEEE 1016-2009](https://standards.ieee.org/ieee/1016/4502/) §5.4.1 — Decomposition Description Attributes
>
> "Each design entity in decomposition must have: identification, type, purpose, function, subordinates."

### ATTR-001: Entity Identification
**Severity**: HIGH
**Ref**: IEEE 1016-2009 §5.4.1 (identification attribute)

- [ ] Each spec has unique **ID** following naming convention (`spd-{system}-spec-{slug}`)
- [ ] IDs are stable (won't change during implementation)
- [ ] IDs are human-readable and meaningful
- [ ] No duplicate IDs within the decomposition

### ATTR-002: Entity Type
**Severity**: MEDIUM
**Ref**: IEEE 1016-2009 §5.4.1 (type attribute)

- [ ] Each spec has **type** classification implied by priority/status (or explicit type field)
- [ ] Type indicates nature: core, supporting, infrastructure, integration, etc.
- [ ] Types are consistent across similar specs

### ATTR-003: Entity Purpose
**Severity**: HIGH
**Ref**: IEEE 1016-2009 §5.4.1 (purpose attribute)

- [ ] Each spec has clear one-line **Purpose** statement
- [ ] Purpose explains WHY this spec exists
- [ ] Purpose is distinct from other specs' purposes
- [ ] Purpose is implementation-agnostic (describes intent, not approach)

### ATTR-004: Entity Function (Scope)
**Severity**: HIGH
**Ref**: IEEE 1016-2009 §5.4.1 (function attribute)

- [ ] Each spec has concrete **Scope** bullets describing WHAT it does
- [ ] Scope items are actionable and verifiable
- [ ] Scope aligns with Purpose
- [ ] Scope is at appropriate abstraction level (not too detailed, not too vague)

### ATTR-005: Entity Subordinates
**Severity**: MEDIUM
**Ref**: IEEE 1016-2009 §5.4.1 (subordinates attribute)

- [ ] Each spec documents phases/milestones (subordinate decomposition)
- [ ] Or explicitly states "single phase" / no sub-decomposition needed
- [ ] Subordinates represent meaningful implementation milestones
- [ ] Subordinate relationships are hierarchically valid

---

## DECOMPOSITION LEVELS (LEV)

> **Standard**: [ISO 21511:2018](https://www.iso.org/standard/69702.html) §5.2 — Levels of Decomposition

### LEV-001: Granularity Consistency
**Severity**: MEDIUM
**Ref**: ISO 21511:2018 §5.2 (decomposition levels)

- [ ] All specs are at similar abstraction level (consistent granularity)
- [ ] No spec is significantly larger than others (≤3x size difference)
- [ ] No spec is significantly smaller than others (≥1/3x size difference)
- [ ] Size is measured by scope items or estimated effort

### LEV-002: Decomposition Depth
**Severity**: MEDIUM
**Ref**: IEEE 1016-2009 §5.4.2 (decomposition hierarchy)

- [ ] Specs are decomposed to implementable units (not too coarse)
- [ ] Specs are not over-decomposed (not too granular for this artifact level)
- [ ] Hierarchy is clear: DESIGN → DECOMPOSITION → (Spec specs)

### LEV-003: Phase Balance
**Severity**: LOW

- [ ] Phase/milestone counts are roughly balanced across specs
- [ ] No spec has disproportionately many phases (>5x average)
- [ ] No spec has zero phases without explicit reason

---

## CONFIGURATION ITEMS (CFG)

> **Standard**: [ISO 10007:2017](https://www.iso.org/standard/70400.html) §6.2 — Configuration Identification
>
> "Configuration items should be selected using established criteria. Their inter-relationships describe the product structure."

### CFG-001: Configuration Item Boundaries
**Severity**: MEDIUM
**Ref**: ISO 10007:2017 §6.2 (CI selection)

- [ ] Each spec represents a logical configuration item (CI)
- [ ] Spec boundaries align with natural configuration/release boundaries
- [ ] Specs can be versioned and baselined independently (where applicable)

### CFG-002: Change Control Readiness
**Severity**: LOW
**Ref**: ISO 10007:2017 §6.3 (change control)

- [ ] Spec status enables configuration status accounting
- [ ] Changes to specs are trackable (ID versioning pattern documented)
- [ ] Spec structure supports incremental delivery

---

## TRACEABILITY (TRC)

> **Standards**: [ISO/IEC/IEEE 29148:2018](https://www.iso.org/standard/72089.html) §6.5, [ISO/IEC/IEEE 42010:2022](https://www.iso.org/standard/74393.html) §5.6

### TRC-001: Forward Traceability (Design → Specs)
**Severity**: HIGH
**Ref**: ISO/IEC/IEEE 29148:2018 §6.5.2 (forward traceability)

- [ ] Each design element can be traced to implementing spec(s)
- [ ] Traceability links use valid design IDs
- [ ] Coverage is explicit (listed in References sections)

### TRC-002: Backward Traceability (Specs → Design)
**Severity**: HIGH
**Ref**: ISO/IEC/IEEE 29148:2018 §6.5.2 (backward traceability)

- [ ] Each spec traces back to source design elements
- [ ] References to design IDs are valid and resolvable
- [ ] No spec exists without design justification

### TRC-003: Cross-Artifact Consistency
**Severity**: HIGH
**Ref**: ISO/IEC/IEEE 42010:2022 §5.6 (architecture description consistency)

- [ ] Spec IDs and slugs will match Spec Design (SPEC) artifacts
- [ ] References between DECOMPOSITION and SPEC artifacts are planned
- [ ] Any missing spec design is documented as intentional

### TRC-004: Impact Analysis Readiness
**Severity**: MEDIUM
**Ref**: ISO/IEC/IEEE 42010:2022 §5.6 (consistency checking)

- [ ] Dependency graph supports impact analysis (what is affected if X changes)
- [ ] Cross-references support reverse lookup (what depends on X)
- [ ] Changes to design can be traced to affected specs

---

## DEPENDENCIES (DEP)

> **Standard**: [ISO/IEC 25010:2023](https://www.iso.org/standard/78176.html) §4.2.7.2 — Modularity (loose coupling)

### DEP-001: Dependency Graph Quality
**Severity**: CRITICAL
**Ref**: ISO/IEC 25010:2023 §4.2.7.2 (Modularity — loose coupling)

- [ ] All dependencies are explicit (Depends On field)
- [ ] No circular dependencies
- [ ] Dependencies form a valid DAG (Directed Acyclic Graph)
- [ ] Foundation specs have no dependencies
- [ ] Dependency links reference existing specs

### DEP-002: Dependency Minimization
**Severity**: HIGH

- [ ] Specs have minimal dependencies (loose coupling)
- [ ] Specs can be implemented independently (given dependencies)
- [ ] Specs support parallel development where possible

### DEP-003: Implementation Order
**Severity**: MEDIUM

- [ ] Dependencies reflect valid implementation order
- [ ] Foundation/infrastructure specs listed first
- [ ] Spec ordering supports incremental delivery

---

## CHECKBOX CONSISTENCY (CHK)

### CHK-001: Status Integrity
**Severity**: HIGH

- [ ] Overall status `id:status` is `[x]` only when ALL spec `id:spec` blocks are `[x]`
- [ ] Spec `id:spec` is `[x]` only when ALL nested `id-ref` blocks within that spec are `[x]`
- [ ] Priority markers (`p1`-`p9`) are consistent between definitions and references
- [ ] Status emoji matches checkbox state (⏳ for in-progress, ✅ for done)

### CHK-002: Reference Validity
**Severity**: HIGH

- [ ] All `id-ref` IDs resolve to valid definitions in source artifacts (DESIGN, PRD)
- [ ] No orphaned checked references (reference checked but definition unchecked)
- [ ] No duplicate checkboxes for the same ID within a spec block

---

## DELIBERATE OMISSIONS (DOC)

### DOC-001: Explicit Non-Applicability
**Severity**: CRITICAL

- [ ] If a design element is intentionally NOT covered, it is explicitly stated with reasoning
- [ ] If specs intentionally overlap, the reason is documented
- [ ] No silent omissions — reviewer can distinguish "considered and excluded" from "forgot"

---

# MUST NOT HAVE

---

## No Implementation Details
**Severity**: CRITICAL

**What to check**:
- [ ] No code snippets or algorithms
- [ ] No detailed technical specifications (belongs in SPEC artifact)
- [ ] No user flows or state machines (belongs in SPEC artifact)
- [ ] No API request/response schemas (belongs in SPEC artifact)

**Where it belongs**: SPEC (spec design) artifact

---

## No Requirements Definitions
**Severity**: HIGH

**What to check**:
- [ ] No functional requirement definitions (FR-xxx) — should reference PRD
- [ ] No non-functional requirement definitions (NFR-xxx) — should reference PRD
- [ ] No use case definitions — should reference PRD
- [ ] No actor definitions — should reference PRD

**Where it belongs**: PRD artifact

---

## No Architecture Decisions
**Severity**: HIGH

**What to check**:
- [ ] No "why we chose X" explanations (should reference ADR)
- [ ] No technology selection rationales (should reference ADR)
- [ ] No pros/cons analysis (should reference ADR)

**Where it belongs**: ADR artifact

---

# Format Validation

## FMT-001: Spec Entry Format
**Severity**: HIGH

- [ ] Each spec entry has unique title
- [ ] Each spec entry has stable identifier
- [ ] Entries are consistently formatted

## FMT-002: Required Fields Present
**Severity**: HIGH

- [ ] **ID**: Present and follows convention
- [ ] **Purpose**: One-line description
- [ ] **Depends On**: None or spec references
- [ ] **Scope**: Bulleted list
- [ ] **Out of Scope**: Bulleted list (or explicit "None")
- [ ] **Requirements Covered**: ID references
- [ ] **Design Components**: ID references

## FMT-003: Checkbox Syntax
**Severity**: CRITICAL

- [ ] All checkboxes use correct syntax: `[ ]` (unchecked) or `[x]` (checked)
- [ ] Checkbox followed by backtick-enclosed priority: `[ ] \`p1\``
- [ ] Priority followed by dash and backtick-enclosed ID

---

# Validation Summary

## Final Checklist

Confirm before reporting results:

- [ ] I checked ALL items in MUST HAVE sections
- [ ] I verified ALL items in MUST NOT HAVE sections
- [ ] I documented all violations found
- [ ] All critical issues have been reported

## Domain Disposition

For each major checklist category, confirm:

- [ ] COV (Coverage): Addressed or violation reported
- [ ] EXC (Exclusivity): Addressed or violation reported
- [ ] ATTR (Attributes): Addressed or violation reported
- [ ] TRC (Traceability): Addressed or violation reported
- [ ] DEP (Dependencies): Addressed or violation reported

---

## Reporting

Report **only** problems (do not list what is OK).

For each issue include:

- **Issue**: What is wrong
- **Evidence**: Quote or location in artifact
- **Why it matters**: Impact on decomposition quality
- **Proposal**: Concrete fix

```markdown
## Review Report (Issues Only)

### 1. {Short issue title}

**Checklist Item**: `{CHECKLIST-ID}` — {Checklist item title}

**Severity**: CRITICAL|HIGH|MEDIUM|LOW

#### Issue

{What is wrong}

#### Evidence

{Quote or "No mention found"}

#### Why It Matters

{Impact on decomposition quality}

#### Proposal

{Concrete fix}
```
