# Prompt: Create Module Design

> **See [Module Development Workflow](../README.md) for complete workflow details and [Reference](../REFERENCE.md) for document templates.**

## Variables

Replace in the prompt below:
- `{module_name}` - snake_case (e.g., `types_registry`, `oagw`)

---

## Prompt template

```
Create `modules/{module_name}/docs/DESIGN.md` — the module architecture document.

## Prerequisites

Read and understand:
- guidelines/NEW_MODULE.md — module development standards
- docs/MODKIT_UNIFIED_SYSTEM.md — ModKit framework patterns
- docs/REQUIREMENTS.md — global requirements to reference

## Document Structure

Create DESIGN.md with these sections:

### # {Module Name} - Design

### ## Overview
[Brief module description and purpose — 2-3 paragraphs]

### ## Problem Statement
[What problem this module solves and why it's needed]

### ## Architecture
[High-level architecture diagram and description]
- Layers (api, service, storage, etc.)
- Key components and responsibilities

### ## Components
[Key components — focus on what, not how]
- Component name and responsibility
- Reference global requirements where applicable (#tenant-isolation, #rbac, etc.)

### ## Data Flow
[How data flows through the module]
- Include sequence or flow diagrams if helpful

### ## Integration Points
[How this module integrates with other modules/systems]

### ## Implementation Phases

#### Phase #{module}/P1: [Phase Name]
[Description of what's included — reference features this phase will deliver]

#### Phase #{module}/P2: [Phase Name]
[Description of what's included]

(Add more phases if needed)

### ## Technical Decisions
[Key architectural and technical decisions with rationale]

### ## Open Questions
[Questions to resolve during design review]

## Guidelines

- Focus on **what** and **why**, not implementation details
- Reference global requirements — don't duplicate definitions
- Keep phases independently implementable
- Mark open questions for design review

Output only the DESIGN.md content.

Module description:
[INSERT YOUR MODULE DESCRIPTION HERE]
```
