# [OpenSpec](https://github.com/Fission-AI/OpenSpec) Guideline for Hyperspot

This guide provides the standard process for AI-driven development in the Hyperspot mono-repo using [OpenSpec](https://github.com/Fission-AI/OpenSpec). It is designed to enable parallel development across multiple teams and modules without conflicts.

## Core Concept: Module-Centric Namespaces

To support parallel development in this mono-repo, OpenSpec is organized by **module namespaces**. This ensures that changes to one module do not conflict with others and ownership is clear.

## Directory Structure

All specs and changes MUST follow the module-prefixed convention:

```
openspec/
├── changes/
│   ├── [module-name]-[change-id]/    # e.g., settings-service-add-audit-logs
│   │   ├── proposal.md           # The "Why" and "What"
│   │   ├── tasks.md              # Implementation checklist
│   │   └── specs/                # Delta changes
│   │       └── [module-name]-[capability]/
│   │           └── spec.md
│   └── archive/                  # Completed and merged changes
├── specs/
│   ├── [module-name]-[capability]/   # e.g., settings-service-core
│   │   ├── spec.md               # Current Truth (Requirements & Scenarios)
│   │   └── design.md             # Technical patterns & decisions
│   └── shared/                   # Cross-cutting concerns (requires coordination)
```

## Workflow

### 1. Naming Your Change
Always prefix your change ID with the module name to prevent collisions with syntax `[module-name]-[change-id]`.
- **Correct**: `settings-service-add-retention-policy`
- **Incorrect**: `add-retention-policy` (Ambiguous in a mono-repo)

### 2. Scoping Your Specs
Specs live in `openspec/specs/[module-name]-[capability]`.
- If you are adding a new capability to the Settings Module, create `openspec/specs/settings-service-newcap`.
- **Granularity**: Avoid monolithic specs. Break them down by capability (e.g., `api`, `storage`, `events`).

### 3. Parallel Development Process
1. **Pull Latest**: Ensure you have the latest `openspec/` state.
2. **Create Branch**: `git checkout -b feature/<your-feature-name>`
3. **Scaffold Change**:
   ```bash
   # Create the change directory structure
   mkdir -p openspec/changes/[module-name]-[change-id]/specs/[module-name]-[capability]
   
   # Initialize documents
   touch openspec/changes/[module-name]-[change-id]/{proposal.md,tasks.md}
   ```
4. **Write Proposal**: Define what and why in `proposal.md`.
5. **Write Deltas**: Define requirements in `specs/[module-name]-[capability]/spec.md`.
   - Use `## ADDED Requirements`, `## MODIFIED Requirements`, etc.
   - See `openspec/AGENTS.md` for strict syntax rules.
6. **Validate**: `openspec validate [module-name]-[change-id] --strict`
7. **Implement**: Code the solution based on the spec.
8. **Archive**: After deployment/merge, move to `changes/archive/`.

## DOs and DON'Ts

### DO
- **DO** use the `[module-name]-` prefix for all change IDs and spec directories.
- **DO** keep capabilities focused (e.g., `settings-service-api`, `settings-service-storage`) rather than one giant `settings-service` spec.
- **DO** reference `openspec/AGENTS.md` for the core syntax rules (Requirements MUST have Scenarios).
- **DO** use `openspec validate` frequently to catch syntax errors early.
- **DO** consult `guidelines/NEW_MODULE.md` when creating a new module to ensure the OpenSpec structure is initialized correctly.

### DON'T
- **DON'T** modify files in `openspec/specs/` directly. Always use a Change Proposal in `openspec/changes/`.
- **DON'T** create top-level specs without a module prefix (unless it's truly global/shared like `shared-auth`).
- **DON'T** merge a PR if `openspec validate` fails.
- **DON'T** overwrite another module's specs. Stick to your namespace.

## Integration with ModKit

When creating a new module using `guidelines/NEW_MODULE.md`:

1.  **Initialize Specs**: Immediately create `openspec/specs/[module-name]-core`.
2.  **Define Contract**: Use the spec to define the module's `contract` (public API) before implementation.
3.  **Iterate**: Use Change Proposals for every significant feature addition to the module.

## Example: Settings Service Module Structure

The Settings Service module serves as a reference implementation of the OpenSpec structure. Here's how it's organized:

### Core Specifications
```
openspec/specs/settings-service-core/
├── spec.md         # Core requirements and scenarios
└── design.md       # Technical design decisions
└── tasks.md        # Implementation checklist
```

### Example Change Proposal
```
openspec/changes/settings-service-add-tenant-hierarchy/
├── proposal.md     # Why and what's changing
├── tasks.md        # Implementation checklist
├── design.md       # Technical design decision
└── specs/
    └── settings-service-tenant/
        └── spec.md # Added/MODIFIED tenant hierarchy requirements
```

## Recommended Steps

For developers and AI assistants starting work in this repo:

1.  **Audit Existing Specs**: Run `openspec list --specs` to see if your module already has a namespace (e.g., `settings-service-core`).
2.  **Initialize Missing Namespaces**: If your module is missing, create the initial directory structure:
    ```bash
    mkdir -p openspec/specs/[module-name]-core
    echo "## Requirements" > openspec/specs/[module-name]-core/spec.md
    ```
3.  **Refactor Legacy Specs**: If you see top-level specs without prefixes (e.g., `openspec/specs/auth`), propose a change to move them to `openspec/specs/auth-module-core` to align with the mono-repo structure.
4.  **Always Validate**: Before submitting any code or spec changes, run:
    ```bash
    openspec validate [module-name]-[change-id] --strict
    ```

