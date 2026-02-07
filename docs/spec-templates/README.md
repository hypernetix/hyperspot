# Spec Templates

Industry-standard specification templates incorporating best practices from IEEE, ISO, and modern software development methodologies. Compatible with [Cypilot-Driven Development](https://github.com/cyberfabric/Cypilot) for validation and traceability.

## Purpose

These templates provide a structured way to document product requirements, technical design, architecture decisions, and feature specifications. They incorporate proven practices from:

- **IEEE 830 / ISO/IEC/IEEE 29148:2018** — Requirements specification, key reference
- **IEEE 1233** — System requirements
- **IEEE 1016-2009** — Software design description
- **IEEE 42010** — Architecture description
- **ISO/IEC 15288 / 12207** — Systems and software life cycle processes
- **MADR** — Markdown Any Decision Records

Templates work standalone or can be enhanced with Cypilot annotations (`cpt-id`) for cross-document validation and traceability.

**Cypilot integration is optional** — templates are useful on their own for clear, consistent documentation.

## Governance & Process

**Governance**: Steering committee with vote/veto authority on major requirements and architecture decisions. Escalation paths documented separately.

**Global Guidelines**: Project-wide standards defined in root PRD, DESIGN, and [guidelines/](../../guidelines/) covering architecture, security, performance, operations, runtime environments, testing strategy.

**Module-level specs**: Document only **deviations** or **extensions** from global standards. Avoid duplicating project-wide requirements.

**Testing Strategy**: All requirements verified via automated tests (unit, integration, e2e, security, performance) targeting 90%+ code coverage. Document verification method only for non-test approaches (analysis, inspection, demonstration).

**Document Format**: All specification documents **MUST be Markdown**. Use standard Markdown syntax. Mermaid diagrams supported for architecture visualizations.

## Templates

| Template | Purpose | Layer |
|----------|---------|-------|
| [PRD.md](./PRD.md) | Product Requirements Document — vision, actors, capabilities, use cases, FR, NFR | Foundation |
| [UPSTREAM_REQS.md](./UPSTREAM_REQS.md) | Upstream Requirements — technical requirements FROM other modules TO this module | Integration |
| [DESIGN.md](./DESIGN.md) | Technical Design — architecture, principles, constraints, domain model, API contracts | System-level |
| [DECOMPOSITION.md](./DECOMPOSITION.md) | Decomposition — break down features into implementation units with traceability | Feature-level |
| [ADR.md](./ADR.md) | Architecture Decision Record — capture decisions, options, trade-offs, consequences | Cross-cutting |
| [FEATURE.md](./FEATURE.md) | Feature Specification — flows, algorithms, states, requirements (CDSL format) | Feature-level |

## Document Structure

Quick reference for what goes where:

### PRD.md
1. **Overview** — Purpose, background, goals, glossary
2. **Actors** — Human and system actors that interact with this module (stakeholders managed at project/task level)
3. **Operational Concept & Environment** (optional) — Module-specific environment constraints (delete if none)
4. **Scope** — In/out of scope boundaries
5. **Functional Requirements** — What the system MUST do (WHAT, not HOW) with rationale; verification method optional (default: automated tests 90%+ coverage)
6. **Non-Functional Requirements** — Module-specific NFRs only (exclusions/extensions from project defaults)
7. **Public Library Interfaces** — Public API surface, stability guarantees, integration contracts
8. **Use Cases** — Optional interaction flows
9. **Acceptance Criteria** — Business-level validation
10. **Dependencies** — External factors
11. **Assumptions** — Environment, users, dependent systems
12. **Risks** — Risks and mitigation strategies
13. **Open Questions** — Unresolved questions
14. **Traceability** — Links to DESIGN, ADR, features

Standards alignment:
  - IEEE 830 / ISO/IEC/IEEE 29148:2018 (requirements specification)
  - IEEE 1233 (system requirements)
  - ISO/IEC 15288 / 12207 (requirements definition)

Note: Stakeholder needs (per ISO 29148) managed at project/task level by steering committee.

### UPSTREAM_REQS.md
**Per upstream module** (repeat for each upstream module):
1. **UPSTREAM: {upstream module name}** — H1 section for each upstream module
2. **System Actor Definition** — How the upstream module interacts with target module
3. **Functional Requirements** — Capabilities the target module must provide
4. **Non-Functional Requirements** — NFRs imposed by upstream module
5. **Public Interface Requirements** — Interfaces that must be exposed

Standards alignment:
  - IEEE 830 / ISO/IEC/IEEE 29148:2018 (requirements specification)
  - ISO/IEC 15288 / 12207 (interface requirements)

Note: Optional document. UPSTREAM_REQS captures what OTHER modules need FROM this module. Each upstream module gets its own H1 section. Use when upstream requirements are known before the module's PRD is complete, or for API-first design.

### DESIGN.md
1. **Architecture Overview** — Vision, drivers (functional + NFR allocation table), layers
2. **Principles & Constraints** — Design principles and constraints with ADR links
3. **Technical Architecture** — Domain model, components, API contracts, external interfaces & protocols, sequences, DB schemas, topology, tech stack
4. **Additional Context** — Extra context that helps implementers
5. **Traceability** — Links to PRD, ADR, features

Standards alignment:
  - IEEE 1016-2009 (Software Design Description)
  - IEEE 42010 (Architecture Description — viewpoints, views, concerns)
  - ISO/IEC 15288 / 12207 (Architecture & Design Definition processes)

### DECOMPOSITION.md

1. **Overview** — Feature context, scope, related artifacts
2. **Requirements Coverage** — Map PRD requirements to implementation units
3. **Implementation Units** — Break down into phases/tasks with dependencies
4. **Traceability** — Links to PRD, DESIGN, features

### ADR/*.md
- **Context and Problem Statement** — What problem are we solving?
- **Decides For Requirements** — Explicit links to requirements/design elements this decision addresses
- **Decision Drivers** — Forces/concerns that matter
- **Considered Options** — Alternatives evaluated
- **Decision Outcome** — Chosen option with rationale
- **Consequences** — Trade-offs accepted
- **Confirmation** — How compliance will be confirmed
- **Pros and Cons of the Options** — Detailed analysis
- **More Information** — Links/evidence
- **Traceability** — Links to PRD/DESIGN

Standards alignment:
  - MADR (Markdown Any Decision Records)
  - IEEE 42010 (architecture decisions as first-class elements)
  - ISO/IEC 15288 / 12207 (decision analysis process)

### features/*.md
1. **Feature Context** — Overview, purpose, actors, PRD requirement references
2. **Actor Flows (CDSL)** — User-facing interactions step by step
3. **Algorithms (CDSL)** — Internal functions and procedures
4. **States (CDSL)** — State machines for entities (optional)
5. **Implementation Requirements** — Specific tasks to build
6. **Acceptance Criteria** — Feature-level validation

---

### About ADR Files (ADR/*.md)

Architecture Decision Records capture **why** a technical decision was made, not just what was decided. Each ADR documents the context, problem statement, considered options, and the chosen solution with its trade-offs. This creates an institutional memory that prevents re-debating settled decisions and helps new team members understand the rationale behind the architecture.

Use ADRs only when there was a meaningful discussion/debate and the rationale needs to be preserved as a historical decision record. Use ADRs for real decision dilemmas and final decision state. Decision history is in git; keep one ADR per decision.

### About Upstream Requirements (UPSTREAM_REQS.md)

Upstream Requirements documents are optional and capture technical requirements **imposed ON** a module **BY** other modules (consumers, dependencies, surrounding systems). This creates an external perspective complementing the internal perspective of PRD.md.

**Key differences from PRD:**
- **PRD.md**: "What does THIS module do?" (internal goals and capabilities)
- **UPSTREAM_REQS.md**: "What do OTHER modules need FROM this module?" (external obligations)

**When to use:**
- **Early-stage development**: Upstream needs are known before the module's full PRD exists
- **API-first design**: Consumers define interface requirements before implementation
- **Integration planning**: Capture cross-module dependencies explicitly
- **Incremental development**: Build features driven by actual consumer needs rather than speculation

**Structure benefits:**
- Requirements grouped by source module for clear ownership
- Each upstream module has its own section with actors, FRs, NFRs, and interfaces
- Cross-module requirements section for shared needs
- Integration contract matrix for visibility across all consumers

A single UPSTREAM_REQS.md can contain requirements from multiple upstream modules, making it a central integration point for the target module.

### About Feature Files (features/*.md)

Feature files bridge the gap between high-level requirements (PRD) and implementation. Each feature describes **what the system does** in enough detail for a developer or AI agent to implement it without ambiguity. Features contain Actor Flows (user-facing interactions), Algorithms (internal logic), States (state machines), and Requirements (implementation tasks).

Unlike PRD which answers "what do we need?", Feature files answer "how exactly does it work?" — step by step, with precise inputs, outputs, conditions, and error handling. This makes them directly translatable to code and testable against acceptance criteria.

**CDSL pseudo-code is optional:**
- **Use** for early-stage projects, complex domains, onboarding new team members, or when precise behavior must be communicated
- **Skip** for mature teams or simple features — avoid documentation overhead when everyone already understands the flow

## Document Placement

Documents should be placed **inside appropriate subsystem or module folder** following this structure:

```
docs/arch/common/ or docs/arch/{subsystem}/ or {module}/
├── PRD.md                     # Product requirements
├── DESIGN.md                  # Technical design
├── ADR/                       # Architecture Decision Records
│   ├── 0001-{cpt-id}.md       # ADR with sequential prefix
│   ├── 0002-{cpt-id}.md
│   └── ...
└── features/                  # Feature specifications
    ├── 0001-{cpt-id}.md       # Feature with sequential prefix
    ├── 0002-{cpt-id}.md
    └── ...
```

### ADR & Feature Naming Convention

Both ADR and Feature files MUST use the prefix `NNNN-{cpt-id}.md`:

**ADRs**:
- `ADR/0001-cpt-todo-app-adr-local-storage.md`
- `ADR/0002-cpt-todo-app-adr-optimistic-ui.md`

**Features**:
- `features/0001-cpt-todo-app-feature-core.md`
- `features/0002-cpt-todo-app-feature-logic.md`

## Cypilot ID Convention

Cypilot IDs enable traceability across all specification artifacts.

### Cypilot ID Definition

An Cypilot ID **defines** a unique identifier for a specification element (actor, requirement, feature, etc.). Each ID must be **globally unique** within the module, subsystem or global project depending on where it's defined

**Format**:
```
cpt-{system}-{kind}-{slug}
```

**Placement**: Use `**ID**: \`cpt-...\`` in the artifact where the element is defined.

### ID Scope

An Cypilot ID covers the **markdown section where it's defined and all its subsections**:

- ID on `#` (H1) → covers the entire document
- ID on `##` (H2) → covers that section and all H3/H4/... within it
- ID on `####` (H4) → covers section H4 and nested content ...

This allows flexible granularity — define IDs at whatever level makes sense for traceability.

### Implementation Status and Priority

Requirements and design elements can include **implementation status** and **priority** directly in the ID line. This bridges the gap between specifications and actual implementation — specs come first, and we need visibility into what's implemented and in what order.

**Format**:
```
- [status] `priority` - **ID**: `cpt-{system}-{kind}-{slug}`
```

| Element | Values | Description |
|---------|--------|-------------|
| Status | `[ ]` / `[x]` | Implementation checkbox — unchecked = pending, checked = done |
| Priority | `p1` `p2` `p3` `p4` | Relative priority — p1 highest |

**Examples**:
```markdown
#### User roles requirement

- [ ] `p1` - **ID**: `cpt-auth-req-user-roles`

The system must support 1-N user roles associated with the user...
```

```markdown
#### Response time

- [x] `p2` - **ID**: `cpt-api-req-response-time`

API responses must complete within 200ms at p95...
```

> **Note**: Implementation status and priority are **informative only** — they don't replace your issue tracking system. Keep them simple. The value is having spec-to-implementation traceability directly in version-controlled documentation, reducing uncertainty between what's specified and what's actually built.

### Cypilot ID Reference

An Cypilot ID **reference** links to an element defined elsewhere. References create traceability between documents — for example, a Feature can reference Actors from PRD, or an ADR can reference Requirements it addresses.

**Placement**: Use backtick notation `` `cpt-...` `` when referencing an ID defined in another section or file.

### Validation

Cypilot IDs must be unique. When Cypilot tooling is connected, `cypilot validate` will:
- Check that all referenced IDs exist
- Detect duplicate definitions
- Verify cross-document consistency

### Kind Reference

These are **suggested** kind names for common artifact types. Cypilot does not enforce specific kind values — use whatever naming makes sense for your project. The important thing is consistency within your codebase.

| Kind | Description |
|------|-------------|
| `actor` | Stakeholder or system actor |
| `req` | Requirement (functional or non-functional) |
| `feature` | Feature specification |
| `adr` | Architecture decision record |
| `design` | Design element (component, API, schema, etc.) |
| `flow` | Actor flow / use case flow |
| `algo` | Algorithm / internal procedure |
| `state` | State machine |

**Examples**:
- `cpt-todo-app-actor-user` — Actor ID
- `cpt-todo-app-req-create-task` — Requirement ID
- `cpt-todo-app-req-response-time` — NFR (still uses `req`)
- `cpt-todo-app-adr-local-storage` — ADR ID
- `cpt-todo-app-feature-core` — Feature ID

> **Note**: You can use any slug that fits your domain. For example, `cpt-billing-uc-checkout` or `cpt-auth-nfr-token-expiry` are equally valid if your team prefers more specific kinds. Cypilot validation only checks that referenced IDs exist — it does not validate kind names.

## Example

See [examples/todo-app/](./examples/todo-app/) for a complete example using a universally understood Todo App theme.

## Cypilot Compatibility

When full Cypilot framework is connected:

1. **Validation** — `cypilot validate` will check document structure and cross-references
2. **Traceability** — IDs will be linked across PRD → DESIGN → ADR → FEATURE → code
3. **Deterministic gates** — CI/CD can enforce document quality before code changes

For more details on Cypilot taxonomy and artifact relationships, see [`TAXONOMY.md`](https://github.com/cyberfabric/Cypilot/blob/main/guides/TAXONOMY.md).
