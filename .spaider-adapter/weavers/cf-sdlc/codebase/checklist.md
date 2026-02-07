# Spaider SDLC Code Checklist (Weaver-Specific)

ALWAYS open and follow `docs/checklists/CODING.md` FIRST

**Artifact**: Code Implementation (Spaider SDLC)
**Version**: 1.0
**Purpose**: Weaver-specific checks that require Spaider SDLC artifacts (PRD/DESIGN/DECOMPOSITION/SPEC/ADR) and/or Spaider traceability.

---

## Table of Contents

1. [Traceability Preconditions](#traceability-preconditions)
2. [Semantic Alignment (SEM)](#semantic-alignment-sem)

---

## Traceability Preconditions

Before running the SDLC-specific checks:

- [ ] Determine traceability mode from `artifacts.json` for the relevant system/artifact: `FULL` vs `DOCS-ONLY`
- [ ] If `FULL`: identify the design source(s) to trace (Spec DESIGN is preferred)
- [ ] If `DOCS-ONLY`: skip marker-based requirements and validate semantics against provided design sources

---

## Semantic Alignment (SEM)

These checks are **Spaider SDLC-specific** because they require Spaider artifacts (Spec Design, Overall Design, ADRs, PRD/DESIGN coverage) and/or Spaider markers.

### SEM-CODE-001: Resolve Design Sources
**Severity**: HIGH

- [ ] Resolve Spec Design via `@spaider-*` markers using the `spaider where-defined` or `spaider where-used` skill
- [ ] If no `@spaider-*` markers exist, ask the user to provide the Spec Design location before proceeding
- [ ] If the user is unsure, search the repository for candidate spec designs and present options for user selection
- [ ] Resolve Overall Design by following references from the Spec Design (or ask the user for the design path)

### SEM-CODE-002: Spec Context Semantics
**Severity**: HIGH

- [ ] Confirm code behavior aligns with the Spec Overview, Purpose, and key assumptions
- [ ] Verify all referenced actors are represented by actual interfaces, entrypoints, or roles in code
- [ ] Ensure referenced ADRs and related specs do not conflict with current implementation choices

### SEM-CODE-003: Spec Flows Semantics
**Severity**: HIGH

- [ ] Verify each implemented flow follows the ordered steps, triggers, and outcomes in Actor Flows
- [ ] Confirm conditionals, branching, and return paths match the flow logic
- [ ] Validate all flow steps marked with IDs are implemented and traceable

### SEM-CODE-004: Algorithms Semantics
**Severity**: HIGH

- [ ] Validate algorithm steps match the Spec Design algorithms (inputs, rules, outputs)
- [ ] Ensure data transformations and calculations match the described business rules
- [ ] Confirm loop/iteration behavior and validation rules align with algorithm steps

### SEM-CODE-005: State Semantics
**Severity**: HIGH

- [ ] Confirm state transitions match the Spec Design state machine
- [ ] Verify triggers and guards for transitions match defined conditions
- [ ] Ensure invalid transitions are prevented or handled explicitly

### SEM-CODE-006: Definition of Done Semantics
**Severity**: HIGH

- [ ] Verify each requirement in Definition of Done is implemented and testable
- [ ] Confirm implementation details (API, DB, domain entities) match the requirement section
- [ ] Validate requirement mappings to flows and algorithms are satisfied
- [ ] Ensure PRD coverage (FR/NFR) is preserved in implementation outcomes
- [ ] Ensure Design coverage (principles, constraints, components, sequences, db tables) is satisfied

### SEM-CODE-007: Overall Design Consistency
**Severity**: HIGH

- [ ] Confirm architecture vision and system boundaries are respected
- [ ] Validate architecture drivers (FR/NFR) are still satisfied by implementation
- [ ] Verify ADR decisions are reflected in code choices or explicitly overridden
- [ ] Confirm principles and constraints are enforced in implementation
- [ ] Validate domain model entities and invariants are respected by code
- [ ] Confirm component responsibilities, boundaries, and dependencies match the component model
- [ ] Validate API contracts and integration boundaries are honored
- [ ] Verify interactions and sequences are implemented as described
- [ ] Ensure database schemas, constraints, and access patterns align with design
- [ ] Confirm topology and tech stack choices are not contradicted
- [ ] Document any deviation with a rationale and approval

