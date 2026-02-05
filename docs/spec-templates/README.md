# Spec Templates

Lightweight specification templates for software projects. Uses Spider-style IDs (`spd-...`) for cross-document traceability.

## Purpose

These templates provide a structured way to document product requirements, technical design, architecture decisions, and feature specifications. They work standalone and can be enhanced later with deterministic validation tooling if desired.

## Templates

| Template | Purpose | Layer |
|----------|---------|-------|
| [PRD.md](./PRD.md) | Product Requirements Document — vision, actors, capabilities, use cases, FR, NFR | Foundation |
| [DESIGN.md](./DESIGN.md) | Technical Design — architecture, principles, constraints, domain model, API contracts | System-level |
| [ADR.md](./ADR.md) | Architecture Decision Record — capture decisions, options, trade-offs, consequences | Cross-cutting |
| [FEATURE.md](./FEATURE.md) | Feature Specification — flows, algorithms, states, requirements (FDL format) | Feature-level |

## Document Structure

Quick reference for what goes where:

### PRD.md
1. **Overview** — Purpose, target users, problems solved, success criteria, capabilities
2. **Actors** — Human actors, system actors
3. **Functional Requirements** — What the system must do (WHAT, not HOW)
4. **Use Cases** — Actor interactions with preconditions, flow, postconditions
5. **Non-functional Requirements** — Quality attributes (performance, security, etc.)
6. **Additional Context** — Market notes, assumptions, stakeholder feedback

### DESIGN.md
1. **Architecture Overview** — Vision, drivers, layers
2. **Principles & Constraints** — Design principles, technical constraints
3. **Technical Architecture** — Domain model, components, API contracts, sequences, DB schemas, topology, tech stack
4. **Additional Context** — Architect notes, rationale

### ADR.md
- **Context and Problem Statement** — What problem are we solving?
- **Decision Drivers** — Key factors influencing the decision
- **Considered Options** — Alternatives evaluated
- **Decision Outcome** — Chosen option with consequences
- **Related Design Elements** — Linked actors and requirements

### FEATURE.md
1. **Feature Context** — Overview, purpose, actors, references
2. **Actor Flows (FDL)** — User-facing interactions step by step
3. **Algorithms (FDL)** — Internal functions and procedures
4. **States (FDL)** — State machines for entities
5. **Requirements** — Implementation tasks with phases
6. **Additional Context** — Performance notes, UX considerations

---

### About ADR Files

Architecture Decision Records capture **why** a technical decision was made, not just what was decided. Each ADR documents the context, problem statement, considered options, and the chosen solution with its trade-offs. This creates an institutional memory that prevents re-debating settled decisions and helps new team members understand the rationale behind the architecture.

ADRs are immutable once accepted — if a decision changes, a new ADR supersedes the old one. This preserves the historical context and evolution of the system's architecture over time.

### About Feature Files

Feature files bridge the gap between high-level requirements (PRD) and implementation. Each feature describes **what the system does** in enough detail for a developer or AI agent to implement it without ambiguity. Features contain Actor Flows (user-facing interactions), Algorithms (internal logic), States (state machines), and Requirements (implementation tasks).

Unlike PRD which answers "what do we need?", Feature files answer "how exactly does it work?" — step by step, with precise inputs, outputs, conditions, and error handling. This makes them directly translatable to code and testable against acceptance criteria.

**FDL pseudo-code is optional:**
- ✅ **Use** for early-stage projects, complex domains, onboarding new team members, or when precise behavior must be communicated
- ⏭️ **Skip** for mature teams or simple features — avoid documentation overhead when everyone already understands the flow

## Document Placement

Documents should be placed **inside the module folder** following this structure:

```
{module-or-system}/
├── PRD.md                      # Product requirements
├── DESIGN.md                   # Technical design
├── ADR/                        # Architecture Decision Records
│   ├── 0001-{id}.md            # ADR with sequential prefix
│   ├── 0002-{id}.md
│   └── ...
└── features/                   # Feature specifications
    ├── 0001-{id}.md            # Feature with sequential prefix
    ├── 0002-{id}.md
    └── ...
```

### ADR & Feature Naming Convention

Both ADR and Feature files MUST use the prefix `NNNN-{id}.md`:

**ADRs**:
- `ADR/0001-spd-todo-app-adr-local-storage.md`
- `ADR/0002-spd-todo-app-adr-optimistic-ui.md`

**Features**:
- `features/0001-spd-todo-app-feature-core.md`
- `features/0002-spd-todo-app-feature-logic.md`

## ID Convention

IDs enable traceability across all specification artifacts.

### ID Definition

An ID **defines** a unique identifier for a specification element (actor, requirement, feature, etc.). Each ID must be **globally unique** within the scope you choose (system/module).

**Format**:
```
spd-{system}-{kind}-{slug}
```

**Placement**: Use `**ID**: \`spd-...\`` in the artifact where the element is defined.

### ID Reference

An ID **reference** links to an element defined elsewhere. References create traceability between documents — for example, a Feature can reference Actors from PRD, or an ADR can reference Requirements it addresses.

**Placement**: Use backtick notation `` `spd-...` `` when referencing an ID defined in another section or file.

### Validation

IDs must be unique. If/when you connect deterministic tooling, you can validate cross-document consistency (references exist, duplicates do not exist).

### Kind Reference

| Kind | Description |
|------|-------------|
| `actor` | Actor (human or system) |
| `fr` | Functional requirement |
| `nfr` | Non-functional requirement |
| `usecase` | Use case |
| `adr` | Architecture decision record |
| `feature` | Feature |
| `flow` | Actor flow (within feature) |
| `algo` | Algorithm (within feature) |
| `state` | State machine (within feature) |
| `req` | Feature requirement |
| `principle` | Design principle |
| `constraint` | Constraint |
| `prdcontext` | PRD additional context |
| `designcontext` | DESIGN additional context |
| `featurecontext` | FEATURE additional context |
| `dbtable` | Database table |
| `topology` | Topology |
| `tech` | Tech stack |

**Examples**:
- `spd-todo-app-actor-user` — Actor ID
- `spd-todo-app-fr-create-task` — Functional Requirement ID
- `spd-todo-app-adr-local-storage` — ADR ID
- `spd-todo-app-feature-core` — Feature ID

## Example

See [examples/todo-app/](./examples/todo-app/) for a complete example using a universally understood Todo App theme.

## Tooling (Optional)

If/when you decide to adopt deterministic validation (e.g., Spider templates + artifact registry), these IDs can become the foundation for automated cross-artifact traceability (PRD → DESIGN → ADR → FEATURE → code).
