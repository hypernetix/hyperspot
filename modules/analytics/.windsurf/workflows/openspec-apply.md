# OpenSpec: Apply Changes

Implement the tasks from an approved change proposal.

## Steps

1. **Select change to implement**
   - List available changes from `openspec/changes/`
   - Ask user which change to implement
   - Load the proposal and tasks

2. **Review the plan**
   - Read `proposal.md` for context
   - Read `tasks.md` for implementation steps
   - Read spec deltas in `specs/` for requirements
   - Confirm understanding with user

3. **Implement each task**
   - Work through tasks in order from `tasks.md`
   - Reference the spec deltas for detailed requirements
   - Follow project guidelines from `@/guidelines/NEW_MODULE.md`
   - Use `SecurityCtx` in all handlers
   - Keep domain logic transport-agnostic

4. **Mark tasks as complete**
   - Update checkboxes in `tasks.md` as you complete them
   - `- [x] Completed task`

5. **Test the implementation**
   - Write tests for new functionality
   - Verify against spec requirements
   - Run existing tests to ensure no regressions

6. **Update documentation**
   - Add rustdoc comments
   - Update README if needed

7. **Report completion**
   - Show what was implemented
   - Highlight any deviations from the plan
   - Note any issues or blockers

## Prerequisites

- Change proposal must exist in `openspec/changes/<change-name>/`
- Proposal must be approved by user

## Next Steps

After implementation is complete:
- Run `/openspec-archive` to merge spec changes and archive
