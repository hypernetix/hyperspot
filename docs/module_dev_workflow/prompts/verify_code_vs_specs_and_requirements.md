# Prompt: Verify Code vs. Specs & Requirements

> **See [Module Development Workflow](../README.md) for workflow details and [Reference](../REFERENCE.md) for ID formats.**

## Variables

Replace in the prompt below:
- `{module}` - lowercase module name (e.g., `oagw`, `types_registry`)
- `{change-name}` - the OpenSpec change name being verified (e.g., `request-forwarding`)

---

## Prompt template

```
Verify that implementation matches OpenSpec specs and requirements for module `{module}`.

**Files to analyze:**
- `modules/{module}/src/` (implementation code)
- `modules/{module}/openspec/specs/**/*.md` (verified scenarios)
- `modules/{module}/docs/features/*/FEATURE.md` (module requirements)
- `docs/REQUIREMENTS.md` (global requirements)
- E2E tests (if applicable)

**Verification checks:**

1. **Scenario Implementation:**
   - Each scenario in specs has corresponding implementation
   - WHEN conditions trigger correct behavior
   - THEN/AND expectations are met by code

2. **Requirements Coverage:**
   - All requirements marked [x] in IMPLEMENTATION_PLAN.md have working code
   - SHALL requirements are fully implemented
   - SHOULD/MAY requirements are implemented or documented as future work

3. **E2E Test Coverage:**
   - Each scenario has corresponding E2E test
   - Tests verify success and error cases

4. **Security Requirements:**
   - `#tenant-isolation` is enforced (tenant ID checked in all data access)
   - `#rbac` is enforced (role/permission checks in endpoints)
   - Other global security requirements are implemented

**Output format:**
Write report to `modules/{module}/docs/verification/{change-name}/code_vs_specs_and_requirements.md`:

# Verification Report: Code vs. Specs & Requirements

**Module:** `{module}`
**Timestamp:** {current date/time}
**Status:** ✅ Passed | ⚠️ Warnings | ❌ Issues Found

## Summary

| Metric | Count |
|--------|-------|
| Total Scenarios | {N} |
| Implemented Scenarios | {N} |
| Missing Implementations | {N} |
| E2E Test Coverage | {N}% |

## Issues

### ❌ Missing Implementation
**Scenario:** {scenario name}
**Location:** {spec file link}
**Description:** {what's missing}

---

### ⚠️ Partial Implementation
**Requirement:** #{module}/{name}
**Description:** {what's incomplete}

---

## Recommendations

- {actionable next steps}
- {suggested OpenSpec changes to create}
```
