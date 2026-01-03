# Active Changes

This directory contains active change proposals that are in progress.

## Structure

Each change has its own folder:
```
changes/
└── <change-name>/
    ├── proposal.md     # Why and what
    ├── tasks.md        # Implementation checklist
    ├── design.md       # Technical decisions (optional)
    └── specs/          # Spec deltas
        └── api/
            └── spec.md
```

## Workflow

1. **Create proposal**: `/openspec-proposal`
   - Agent creates folder structure
   - Generates proposal, tasks, and spec deltas
   - User reviews and approves

2. **Implement**: `/openspec-apply`
   - Agent implements tasks following specs
   - Marks tasks complete
   - Tests implementation

3. **Archive**: `/openspec-archive`
   - Merges spec deltas into `../specs/`
   - Moves change to `../archive/`

## Change States

- **Active**: In this directory, being worked on
- **Archived**: In `../archive/`, completed and merged

## Commands

```bash
# List active changes
openspec list

# View change details
openspec show <change-name>

# Validate change structure
openspec validate <change-name>
```
