# FDD-Driven Templates

Lightweight document templates for [Flow-Driven Development](https://github.com/cyberfabric/FDD) **before** full FDD framework integration.

## Purpose

These templates allow teams to start documenting projects in a structured, FDD-compatible way without requiring the full FDD toolchain. When FDD is later connected to the project, these documents will be **fully compatible** and can be used directly with the framework's validation and traceability features.

**This is the first stage of FDD integration.**

## Templates

| Template | Purpose | Layer |
|----------|---------|-------|
| [PRD.md](./PRD.md) | Product Requirements Document — vision, actors, capabilities, use cases, FR, NFR | Foundation |
| [DESIGN.md](./DESIGN.md) | Technical Design — architecture, principles, constraints, domain model, API contracts | System-level |
| [ADR.md](./ADR.md) | Architecture Decision Record — capture decisions, options, trade-offs, consequences | Cross-cutting |
| [FEATURE.md](./FEATURE.md) | Feature Specification — requirements, flows and algorithms (in FDL format), states  | Feature-level |

## Document Placement

Documents should be placed **inside the module folder** following this structure:

```
{module}/
├── PRD.md                      # Product requirements
├── DESIGN.md                   # Technical design
├── ADR/                        # Architecture Decision Records
│   ├── 0001-{fdd-id}.md       # ADR with sequential prefix
│   ├── 0002-{fdd-id}.md
│   └── ...
└── features/                   # Feature specifications
    ├── {FEATURE-NAME}.md
    └── ...
```

### ADR Naming Convention

ADR files MUST use the prefix `NNNN-{fdd-id}.md`:
- `0001-fdd-todo-app-adr-local-storage.md`
- `0002-fdd-todo-app-adr-optimistic-ui.md`

### Feature Files

Feature files are placed in `features/` folder with descriptive names:
- `features/CORE.md`
- `features/LOGIC.md`
- `features/AUTH.md`

## FDD ID Convention

All artifacts use stable FDD IDs for traceability:

```
fdd-{module-name}-{kind}-{slug}
```

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
| `context` | Additional context |
| `db-table` | Database table |
| `topology` | Topology |
| `tech` | Tech stack |

**Examples**:
- `fdd-todo-app-actor-user` — Actor ID
- `fdd-todo-app-fr-create-task` — Functional Requirement ID
- `fdd-todo-app-adr-local-storage` — ADR ID
- `fdd-todo-app-feature-core` — Feature ID

## Example

See [examples/todo-app/](./examples/todo-app/) for a complete example using a universally understood Todo App theme.

## FDD Compatibility

When full FDD framework is connected:

1. **Validation** — `fdd validate` will check document structure and cross-references
2. **Traceability** — IDs will be linked across PRD → DESIGN → ADR → FEATURE → code
3. **Deterministic gates** — CI/CD can enforce document quality before code changes

For more details on FDD taxonomy and artifact relationships, see [`TAXONOMY.md`](https://github.com/cyberfabric/FDD/blob/main/guides/TAXONOMY.md).
