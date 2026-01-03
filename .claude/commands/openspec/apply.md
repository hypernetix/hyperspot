---
name: OpenSpec: Apply
description: Implement an approved OpenSpec change and keep tasks in sync.
category: OpenSpec
tags: [openspec, apply]
---
<!-- OPENSPEC:START -->
**Guardrails**
- Favor straightforward, minimal implementations first and add complexity only when it is requested or clearly required.
- Keep changes tightly scoped to the requested outcome.
- Refer to `openspec/AGENTS.md` (located inside the `openspec/` directory—run `ls openspec` or `openspec update` if you don't see it) if you need additional OpenSpec conventions or clarifications.

**Steps**
Track these steps as TODOs and complete them one by one.
1. Read `changes/<id>/proposal.md`, `design.md` (if present), and `tasks.md` to confirm scope and acceptance criteria.
2. Work through tasks sequentially, keeping edits minimal and focused on the requested change.
3. Confirm completion before updating statuses—make sure every item in `tasks.md` is finished.
4. Update the checklist after all work is done so each task is marked `- [x]` and reflects reality.
5. Reference `openspec list` or `openspec show <item>` when additional context is required.

**Reference**
- Use `openspec show <id> --json --deltas-only` if you need additional context from the proposal while implementing.
<!-- OPENSPEC:END -->

**Monorepo Context**
This is a multi-module monorepo. Individual modules in `modules/` may have their own `openspec/` directory (e.g., `modules/<module-name>/openspec/`).
- Before running any `openspec` command, determine if the change targets a specific module or the root project.
- If the change belongs to a module with its own `openspec/` directory, run all `openspec` commands from that module's directory (e.g., `cd modules/<module-name> && openspec show <id>`).
- If targeting the root project or a module without its own openspec, use the root `openspec/` directory.
- When in doubt, check for `openspec/` directories with `ls modules/*/openspec 2>/dev/null` and ask the user which scope applies.
