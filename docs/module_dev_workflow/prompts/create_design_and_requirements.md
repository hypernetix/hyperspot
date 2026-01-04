# Prompt: Create Module Design and Requirements

> **See [Module Development Workflow](../README.md) for complete workflow details and [Reference](../REFERENCE.md) for document templates.**

## Variables

Replace in the prompt below:
- `{module_name}` - snake_case (e.g., `types_registry`, `oagw`)
- `{MODULE}` - UPPERCASE prefix (e.g., `TYPEREG`, `OAGW`)

---

## Prompt template

```
Take module description (below) and create `modules/{module_name}/docs/DESIGN.md` and `modules/{module_name}/docs/REQUIREMENTS.md` through an iterative design process.

## Iterative Process

Design and requirements evolve together. Follow this iterative approach:

1. **Draft Initial Design** - Create DESIGN.md with:
   - Overview, Architecture, Components, Data Flow
   - Integration Points, Implementation Phases, Technical Decisions
   - Use phase IDs: #{module}/P1, #{module}/P2, etc.

2. **Extract Requirements** - From the design, identify:
   - What the system SHALL/SHOULD/MAY do
   - Create REQUIREMENTS.md with `#{module}/{name}` format
   - Reference global requirements (`#tenant-isolation`, `#rbac`...) from docs/REQUIREMENTS.md

3. **Refine Design from Requirements** - Requirements often reveal:
   - Missing components or data flows
   - Unclear integration points
   - Phase boundary adjustments
   - Update DESIGN.md accordingly

4. **Cross-Reference** - Ensure alignment:
   - Add requirement IDs to DESIGN.md phases, components, data flows
   - Verify each requirement maps to a phase
   - Check that design decisions support all requirements

5. **Iterate** - Repeat steps 2-4 until:
   - Both documents are internally consistent
   - All capabilities have corresponding requirements
   - All requirements are supported by the design

## Document Standards

**DESIGN.md:**
- Follow guidelines/NEW_MODULE.md for module standards
- Follow docs/MODKIT_UNIFIED_SYSTEM.md for ModKit patterns
- Follow layering in architecture (api, service, storage, etc.)
- Sections: Overview, Architecture, Components, Data Flow, Integration Points, Implementation Phases, Technical Decisions, Dependencies, Open Questions
- Focus on "what" and "why", not implementation details

**REQUIREMENTS.md:**
- Use RFC 2119 language: SHALL (mandatory), SHOULD (recommended), MAY (optional)
- Each requirement: ID, statement, Details, References, Phase, Rationale
- Atomic (one capability each) and testable
- Reference global requirements - do NOT duplicate them

## Output

Both files with cross-references:
- DESIGN.md with requirement IDs in phases/components
- REQUIREMENTS.md with phase references and rationale

Module description:
[INSERT YOUR MODULE DESCRIPTION HERE]
```