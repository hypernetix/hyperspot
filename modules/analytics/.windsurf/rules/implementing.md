---
trigger: model_decision
description: When implementing code or writing Rust files
---
# Implementation Following OpenSpec

## Before Writing Code
- Check if proposal exists in `openspec/changes/`
- Read proposal.md for context
- Read tasks.md for checklist
- Read spec deltas in `specs/` for requirements

## While Implementing
- Mark tasks complete in tasks.md: `- [x]`
- Reference spec deltas for detailed requirements
- Follow `@/guidelines/NEW_MODULE.md` patterns
- Use `SecurityCtx` in all handlers
- Keep domain logic in domain/
- Keep API logic in api/rest/

## After Implementation
- Write tests (unit + integration)
- Update documentation
- Verify against spec requirements
- Report completion to user