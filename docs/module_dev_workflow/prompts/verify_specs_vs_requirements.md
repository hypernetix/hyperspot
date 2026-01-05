# Prompt: Verify Specs vs. Requirements

> **See [Module Development Workflow](../README.md) for workflow details and [Reference](../REFERENCE.md) for ID formats.**

## Variables

Replace in the prompt below:
- `{module}` - lowercase module name (e.g., `oagw`, `types_registry`)
- `{change-name}` - the OpenSpec change name being verified (e.g., `request-forwarding`)

---

## Prompt template

```
Verify alignment between OpenSpec specs and requirements for module `{module}`.

**Files to verify:**
- `modules/{module}/openspec/changes/{change-name}/specs/**/*.md` (proposed scenarios)
- `modules/{module}/docs/REQUIREMENTS.md` (module requirements)
- `docs/REQUIREMENTS.md` (global requirements)

**Verification checks:**

1. **Coverage:**
   - All `Verifies: #module/name` references point to existing requirements
   - No typos or non-existent requirement IDs

2. **Completeness:**
   - Proposed scenarios cover the feature scope from the proposal
   - Success, error, and edge cases are considered

3. **Consistency:**
   - Requirement IDs use correct format: `#{module}/{name}`
   - Global requirements use format: `#{name}`

**Output format:**
Write report to `modules/{module}/docs/verification/{change-name}/specs_vs_requirements.md`:

# Verification Report: Specs vs. Requirements

**Module:** `{module}`
**Change:** `{change-name}`
**Timestamp:** {current date/time}
**Status:** ✅ Passed | ⚠️ Warnings | ❌ Issues Found

## Summary

| Metric | Count |
|--------|-------|
| Total Requirements Referenced | {N} |
| Valid References | {N} |
| Missing Requirements | {N} |
| Orphaned Scenarios | {N} |

## Issues

### ❌ Missing Requirement
**Scenario:** {scenario name}
**Reference:** `#{module}/{name}`
**Description:** Referenced requirement does not exist

---

### ⚠️ Missing Error Case
**Requirement:** `#{module}/{name}`
**Description:** No error scenario defined for this requirement

---

## Recommendations

- {actionable next steps}
- {missing scenarios to add}
```
