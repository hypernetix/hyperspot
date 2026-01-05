# Prompt: Verify Code vs. Design

> **See [Module Development Workflow](../README.md) for workflow details and [Reference](../REFERENCE.md) for ID formats.**

## Variables

Replace in the prompt below:
- `{module}` - lowercase module name (e.g., `oagw`, `types_registry`)
- `{change-name}` - the OpenSpec change name being verified (e.g., `request-forwarding`)

---

## Prompt template

```
Verify that code structure follows DESIGN.md architecture for module `{module}`.

**Files to analyze:**
- `modules/{module}/src/` (implementation code)
- `modules/{module}/docs/DESIGN.md` (architecture document)

**Verification checks:**

1. **Layer Structure:**
   - Code organization matches DESIGN.md layers (contract, api, domain, infra)
   - No layer violations (e.g., domain importing from infra)
   - Public API matches contract definitions

2. **Components:**
   - All components described in DESIGN.md exist in code
   - No undocumented major components
   - Component responsibilities match design

3. **Integration Points:**
   - Dependencies listed in DESIGN.md are present
   - REST routes match described endpoints
   - External integrations follow design patterns

4. **Data Flow:**
   - Data flow implementation follows DESIGN.md diagrams
   - Request/response transformations match design

**Output format:**
Write report to `modules/{module}/docs/verification/{change-name}/code_vs_design.md`:

# Verification Report: Code vs. Design

**Module:** `{module}`
**Timestamp:** {current date/time}
**Status:** ✅ Aligned | ⚠️ Minor Deviations | ❌ Significant Drift

## Summary

| Aspect | Status |
|--------|--------|
| Layer Structure | ✅/⚠️/❌ |
| Components | ✅/⚠️/❌ |
| Integration Points | ✅/⚠️/❌ |
| Data Flow | ✅/⚠️/❌ |

## Issues

### ❌ Missing Component
**Expected:** {component from DESIGN.md}
**Status:** Not found in code
**Action:** Implement or update DESIGN.md

---

### ⚠️ Undocumented Component
**Found:** {component in code}
**Status:** Not in DESIGN.md
**Action:** Add to DESIGN.md or remove if unnecessary

---

### ❌ Layer Violation
**File:** {file path}
**Issue:** {description of violation}
**Action:** Refactor to respect layer boundaries

---

## Recommendations

- {actionable next steps}
- {DESIGN.md updates needed}
- {code refactoring suggestions}
```
