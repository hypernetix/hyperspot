# Prompt: Create Implementation Plan

> **See [Module Development Workflow](../README.md) for complete workflow details and [Reference](../REFERENCE.md) for IMPLEMENTATION_PLAN format.**

## Variables

Replace in the prompt below:
- `{module_name}` - snake_case (e.g., `types_registry`, `oagw`)

---

## Prompt template

```
Read `modules/{module_name}/docs/DESIGN.md` and all `modules/{module_name}/docs/features/*/FEATURE.md` files, then create `modules/{module_name}/docs/IMPLEMENTATION_PLAN.md` following the template in docs/module_dev_workflow/REFERENCE.md (IMPLEMENTATION_PLAN.md section).

## Requirements

- Organize by implementation phases from DESIGN.md
- Use nested checkbox structure: Phase → Feature → Requirements
- Format:
  ```
  ## Phase #{module}/P1: [Phase Name]
  
  **Goal:** [What this phase achieves]
  
  - [ ] **{feature-name}** — [Brief description]
    - [ ] #{module}/{req-1}: [Requirement title]
    - [ ] #{module}/{req-2}: [Requirement title]
  ```
- Each feature checkbox has its requirements as sub-items
- Group features by their phase (from FEATURE.md Phase field)
- Ensure all requirements from all FEATURE.md files are included

Output IMPLEMENTATION_PLAN.md.
```

