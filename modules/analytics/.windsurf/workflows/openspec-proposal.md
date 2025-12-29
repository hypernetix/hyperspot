# OpenSpec: Create Proposal

Create a new change proposal in the OpenSpec workflow.

## Steps

1. **Understand the requirement**
   - Ask clarifying questions if the requirement is unclear
   - Identify which specs will be affected

2. **Create change folder structure**
   - Create `openspec/changes/<change-name>/`
   - Generate `proposal.md` - explain WHY and WHAT changes
   - Generate `tasks.md` - implementation checklist
   - Create `specs/` subfolder with delta files showing changes

3. **Write proposal.md**
   - Clear description of the change
   - Motivation and context
   - Scope and affected components
   - Dependencies and prerequisites

4. **Write tasks.md**
   - Break down implementation into concrete tasks
   - Use checkboxes for tracking: `- [ ] Task description`
   - Order tasks logically
   - Reference relevant specs

5. **Create spec deltas**
   - In `openspec/changes/<change-name>/specs/`
   - Mirror the structure of `openspec/specs/`
   - Show ONLY the additions/changes using delta format:
     ```
     # Add: New Section
     Content to add

     # Modify: Existing Section
     - Old line
     + New line

     # Remove: Section to Delete
     ```

6. **Present to user for review**
   - Show the proposal structure
   - Explain key changes
   - **WAIT for user approval** before proceeding

## Output

A complete change proposal in:
```
openspec/changes/<change-name>/
├── proposal.md
├── tasks.md
└── specs/
    └── [affected-specs]/
        └── spec.md (delta)
```

## Next Steps

After user approval:
- Run `/openspec-apply` to implement the tasks
- Run `/openspec-archive` when complete to merge changes
