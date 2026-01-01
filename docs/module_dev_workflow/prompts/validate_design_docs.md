# Prompt: Validate Design Documents

> **See [Module Development Workflow](../README.md) for validation criteria and [Reference](../REFERENCE.md) for ID formats and document structures.**

## Variables

Replace in the prompt below:
- `{module_name}` - snake_case (e.g., `types_registry`, `oagw`)
- `{MODULE}` - UPPERCASE prefix (e.g., `TYPEREG`, `OAGW`)

---

## Prompt template

```
Validate design documents for `modules/{module_name}/docs/`:

**Files to validate:**
- DESIGN.md
- REQUIREMENTS.md
- IMPLEMENTATION_PLAN.md
- docs/REQUIREMENTS.md (global requirements reference)

**Validation categories:**

1. **Format Validation:**
   - DESIGN.md has required sections (Overview, Architecture, Implementation Phases)
   - REQUIREMENTS.md: each requirement has proper header format `{MODULE}-REQ{N}`, uses RFC 2119 keywords (SHALL/SHOULD/MAY), has Phase and Rationale fields
   - IMPLEMENTATION_PLAN.md: features use checkbox format, have requirement references and Scope hints
   - ID formats: {MODULE}-REQ{N}, {MODULE}-P{N}, REQ{N}

2. **Consistency Validation:**
   - Phases in IMPLEMENTATION_PLAN.md match DESIGN.md
   - Requirements reference valid phases from DESIGN.md

3. **Cross-Reference Validation:**
   - All requirement references in IMPLEMENTATION_PLAN.md exist in REQUIREMENTS.md or docs/REQUIREMENTS.md
   - All requirements in REQUIREMENTS.md are referenced by at least one feature
   - All features reference at least one requirement

4. **Completeness Validation:**
   - Each phase has features
   - Each feature has scope hint
   - Each requirement has rationale

**Output format:**
## Validation Results
### Errors (must fix)
- [ERROR] {location} - {description}

### Warnings (should fix)
- [WARNING] {location} - {description}

### Summary
- Errors: {count}
- Warnings: {count}
- Status: {PASSED | FAILED}

ERROR = blocks proceeding to PR
WARNING = should review

If PASSED (0 errors), ready for Step 1.5 (Create PR for Design Review).
```
