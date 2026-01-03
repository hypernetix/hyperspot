# OpenSpec Specifications

This directory contains the **source of truth** specifications for the Analytics module.

## Structure

```
specs/
└── api/
    └── spec.md     # API specifications
```

## Guidelines

- Specifications are written in Markdown
- Each spec describes current implemented behavior
- Specs are updated only when changes are archived
- Never edit specs directly - use OpenSpec workflow
- Changes happen via `openspec/changes/` → approved → archived → merged here

## Creating New Specs

Use OpenSpec workflow:
1. Run `/openspec-proposal` to create a change
2. Add spec in `openspec/changes/<change-name>/specs/`
3. Get approval
4. Implement with `/openspec-apply`
5. Archive with `/openspec-archive` to merge here

## Spec Format

Use clear Markdown structure:
- Headers for sections
- Lists for requirements
- Code blocks for examples
- Tables for data structures

Example:
```markdown
# Feature Name

## Overview
Brief description

## Requirements
- Requirement 1
- Requirement 2

## API Endpoints
### GET /resource
Description and behavior
```
