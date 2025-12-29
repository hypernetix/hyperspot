# OpenSpec: Archive Change

Archive a completed change and merge spec updates.

## Steps

1. **Verify completion**
   - Confirm all tasks in `tasks.md` are checked off
   - Verify tests are passing
   - Confirm code is working as expected
   - Ask user for final approval

2. **Merge spec deltas**
   - Read spec deltas from `openspec/changes/<change-name>/specs/`
   - Apply changes to corresponding files in `openspec/specs/`
   - Create new spec files if they don't exist
   - For existing specs:
     - Apply "Add" sections
     - Apply "Modify" sections
     - Remove "Remove" sections

3. **Move to archive**
   - Create `openspec/archive/<change-name>/`
   - Move entire change folder to archive
   - Preserve all files (proposal, tasks, spec deltas)

4. **Update tracking**
   - Remove from active changes list
   - Document in archive

5. **Clean up**
   - Verify `openspec/changes/<change-name>/` is removed
   - Verify `openspec/archive/<change-name>/` exists
   - Verify specs in `openspec/specs/` are updated

6. **Report completion**
   - Confirm archival
   - Show updated specs
   - Summarize what was implemented

## Prerequisites

- Change must be fully implemented
- All tasks must be complete
- User must approve archival

## Result

The change is archived and spec updates are merged into the source of truth.

```
openspec/
├── specs/           # Updated with changes
├── changes/         # Change removed
└── archive/
    └── <change-name>/  # Archived
```
