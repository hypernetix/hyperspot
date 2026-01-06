# Prompt: Create Feature

> **See [Module Development Workflow](../README.md) for complete workflow details and [Reference](../REFERENCE.md) for FEATURE.md template.**

## Variables

Replace in the prompt below:
- `{module_name}` - snake_case (e.g., `types_registry`, `oagw`)
- `{feature_name}` - kebab-case (e.g., `request-forward`, `entity-registration`)

---

## Prompt template

```
Create `modules/{module_name}/docs/features/{feature_name}/FEATURE.md` — a feature definition with requirements.

## Prerequisites

Read and understand:
- modules/{module_name}/docs/DESIGN.md — module architecture and phases
- docs/REQUIREMENTS.md — global requirements to reference (not duplicate)
- docs/module_dev_workflow/REFERENCE.md — FEATURE.md template

## Document Structure

Create FEATURE.md with these sections:

### # {Feature Name}

### ## Overview
[Brief description of what this feature delivers — 1-2 paragraphs]

### ## Requirements

For each requirement in this feature:

#### #{module}/{requirement-name}: [Requirement Title]

The system SHALL/SHOULD/MAY [requirement description].

**Details:**
- [Specific detail 1]
- [Specific detail 2]

**Phase:** #{module}/P{N}
**References:** #global-req-1, #global-req-2 (if applicable)
**Rationale:** [Why this requirement exists]

(Repeat for each requirement)

### ## Implementation Approach
[High-level technical approach — reference DESIGN.md architecture]
- Which components are involved
- Key integration points
- Technical considerations

## Guidelines

- Use RFC 2119 language: SHALL (mandatory), SHOULD (recommended), MAY (optional)
- Requirements must be atomic and testable
- Reference global requirements — don't duplicate them
- Use flat requirement IDs: #{module}/{requirement-name}
- Each requirement references its implementation phase

## Notes

- This feature will map 1:1 to an OpenSpec specification (openspec/specs/{feature_name}/spec.md)
- The feature will be implemented via 1-10 OpenSpec changes
- Each OpenSpec change should implement 1-5 requirements from this feature

Output only the FEATURE.md content.

Feature description:
[INSERT YOUR FEATURE DESCRIPTION HERE]
```
