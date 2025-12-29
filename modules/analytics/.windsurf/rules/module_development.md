---
trigger: always_on
---
# Analytics Module - OpenSpec SDLC

This module uses OpenSpec for all development.

## Prerequisites (Read on Start)
1. `@/AGENTS.md` - Root instructions
2. `@/guidelines/NEW_MODULE.md` - Module patterns
3. [openspec/AGENTS.md](openspec/AGENTS.md) - OpenSpec SDLC workflow

## Critical Rules
- ❌ Never write code without approved proposal
- ❌ Never skip SecurityCtx in handlers
- ✅ Always use Windsurf workflows (`/openspec-*`)
- ✅ Always get user approval after proposal
- ✅ Always follow project guidelines

## When User Requests Feature
1. Run `/openspec-proposal`
2. Create proposal + tasks + spec deltas
3. WAIT for user approval
4. Run `/openspec-apply` to implement
5. Run `/openspec-archive` when complete

## Implementation Guidelines
- Follow `@/guidelines/NEW_MODULE.md`
- Use `SecurityCtx` as first parameter
- Keep domain layer transport-agnostic
- Write tests for everything