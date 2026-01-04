# Prompt: Create Implementation Plan

> **See [Module Development Workflow](../README.md) for complete workflow details and [Reference](../REFERENCE.md) for IMPLEMENTATION_PLAN format and feature granularity guidelines.**

## Variables

Replace in the prompt below:
- `{module_name}` - snake_case (e.g., `types_registry`, `oagw`)
- `{MODULE}` - UPPERCASE prefix (e.g., `TYPEREG`, `OAGW`)

---

## Prompt template

```
Read `modules/{module_name}/docs/DESIGN.md` and `modules/{module_name}/docs/REQUIREMENTS.md` and create `modules/{module_name}/docs/IMPLEMENTATION_PLAN.md` following the template in docs/module_dev_workflow/REFERENCE.md (IMPLEMENTATION_PLAN.md section).

Requirements:
- Organize features by implementation phases from DESIGN.md (or as flat list for simple modules)
- Each feature must:
  - Have checkbox format: `- [ ] Feature Name (#module/req-name)`
  - Reference at least one requirement (module or global)
  - Include scope hint on next line (6-space indent): `Scope: layers/components involved`
- Feature granularity: 1-5 days of work, implements 1-3 related requirements
- Good feature names: "Entity Registration", "Pagination Support", "Request Forwarding"
- Good scope hints: "contract traits, domain service, REST endpoint, database storage"
- Ensure all requirements from REQUIREMENTS.md are covered by at least one feature

Output IMPLEMENTATION_PLAN.md.
```
