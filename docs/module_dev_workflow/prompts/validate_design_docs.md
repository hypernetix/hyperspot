# Prompt: Validate Design Documents

> **See [Module Development Workflow](../README.md) for validation criteria and [Reference](../REFERENCE.md) for ID formats and document structures.**

## Variables

Replace in the prompt below:
- `{module_name}` - snake_case (e.g., `types_registry`, `oagw`)

---

## Prompt template

```
Validate design documents for `modules/{module_name}/docs/`:

**Files to validate:**
- DESIGN.md
- features/*/FEATURE.md (all feature files)
- IMPLEMENTATION_PLAN.md
- docs/REQUIREMENTS.md (global requirements reference)

**Validation categories:**

1. **Format Validation:**
   - DESIGN.md has required sections (Overview, Architecture, Implementation Phases)
   - Each FEATURE.md: has Overview, Requirements, Implementation Approach sections
   - Requirements use proper header format `#{module}/{name}`, RFC 2119 keywords (SHALL/SHOULD/MAY), Phase and Rationale fields
   - IMPLEMENTATION_PLAN.md: nested checkbox format (phases → features → requirements)
   - ID formats: `#{name}` (global), `#{module}/{name}` (module), `#{module}/P{N}` (phases)

2. **Consistency Validation:**
   - Phases in IMPLEMENTATION_PLAN.md match DESIGN.md
   - Requirements reference valid phases from DESIGN.md
   - Feature names in IMPLEMENTATION_PLAN match feature directories

3. **Cross-Reference Validation:**
   - All requirement references in IMPLEMENTATION_PLAN.md exist in FEATURE.md files or docs/REQUIREMENTS.md
   - All requirements in FEATURE.md files are referenced in IMPLEMENTATION_PLAN.md
   - Global requirement references (#name) exist in docs/REQUIREMENTS.md

4. **Completeness Validation:**
   - Each phase has at least one feature
   - Each feature has at least one requirement
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

