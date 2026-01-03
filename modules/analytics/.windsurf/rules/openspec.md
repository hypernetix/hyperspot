---
trigger: glob
globs: openspec/**/*
---
# OpenSpec Workflow

When working with files in `openspec/`:

## Always Read First
- [openspec/AGENTS.md](openspec/AGENTS.md) - OpenSpec SDLC instructions
- Relevant workflow from `.windsurf/workflows/openspec-*.md`

## Key Principles
- Specs first, code second
- All changes start with proposal
- Never skip user approval
- Follow 3-phase workflow: proposal → implement → archive

## Current Context
- `openspec/specs/` = source of truth
- `openspec/changes/` = active work
- `openspec/archive/` = completed changes

## Commands Available
- `/openspec-proposal` - create proposal
- `/openspec-apply` - implement tasks
- `/openspec-archive` - archive change

## Before Any Action
Check active changes with: `openspec list`