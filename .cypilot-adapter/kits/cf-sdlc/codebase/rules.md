# CODE Rules

**Target**: Codebase Implementation
**Purpose**: Rules for code generation and validation with Cypilot traceability

---

## Table of Contents

- [CODE Rules](#code-rules)
  - [Table of Contents](#table-of-contents)
  - [Requirements](#requirements)
    - [ID Format](#id-format)
    - [Structural Requirements](#structural-requirements)
    - [Traceability Requirements](#traceability-requirements)
    - [Checkbox Cascade (Code Markers → Upstream Artifacts)](#checkbox-cascade-code-markers--upstream-artifacts)
    - [Versioning Requirements](#versioning-requirements)
    - [Engineering Best Practices (MANDATORY)](#engineering-best-practices-mandatory)
    - [Quality Requirements](#quality-requirements)
  - [Tasks](#tasks)
    - [Phase 1: Setup](#phase-1-setup)
    - [Phase 2: Implementation (Work Packages)](#phase-2-implementation-work-packages)
    - [Phase 3: Cypilot Markers (Traceability Mode ON only)](#phase-3-cypilot-markers-traceability-mode-on-only)
    - [Phase 4: Sync FEATURE (Traceability Mode ON only)](#phase-4-sync-feature-traceability-mode-on-only)
    - [Phase 5: Quality Check](#phase-5-quality-check)
    - [Phase 6: Tag Verification (Traceability Mode ON only)](#phase-6-tag-verification-traceability-mode-on-only)
    - [When Updating Existing Code](#when-updating-existing-code)
  - [Validation](#validation)
    - [Phase 1: Implementation Coverage](#phase-1-implementation-coverage)
    - [Phase 2: Traceability Validation (Mode ON only)](#phase-2-traceability-validation-mode-on-only)
    - [Phase 3: Test Scenarios Validation](#phase-3-test-scenarios-validation)
    - [Phase 4: Build and Lint Validation](#phase-4-build-and-lint-validation)
    - [Phase 5: Test Execution](#phase-5-test-execution)
    - [Phase 6: Code Quality Validation](#phase-6-code-quality-validation)
    - [Phase 7: Code Logic Consistency with Design](#phase-7-code-logic-consistency-with-design)
    - [Traceability Report](#traceability-report)
    - [Quality Report](#quality-report)
    - [PASS/FAIL Criteria](#passfail-criteria)
    - [Phase 8: Semantic Expert Review (Always)](#phase-8-semantic-expert-review-always)
  - [Next Steps](#next-steps)
    - [After Successful Implementation](#after-successful-implementation)
    - [After Validation Issues](#after-validation-issues)
    - [If No Design Exists](#if-no-design-exists)

---

**Dependencies**:
- `checklist.md` — code quality criteria
- `{adapter-dir}/AGENTS.md` — project conventions
- `../rules.md` — common CyberFabric SDLC rules shared by all kinds
- **Source** (one of, in priority order):
  1. FEATURE artifact (implementable unit in this kit)
  2. Other Cypilot artifact — PRD, DESIGN, ADR, DECOMPOSITION
  3. Similar content — user-provided description, spec, or requirements
  4. Prompt only — direct user instructions

**ALWAYS open and follow** `{cypilot_path}/requirements/traceability.md` WHEN Traceability Mode is FULL (marker syntax, validation rules, coverage requirements)

**ALWAYS read** the FEATURE artifact being implemented. The FEATURE contains flows, algorithms, state machines, and requirements that define what code must do (if provided and exists)

**ALWAYS read** the system's DESIGN artifact (if registered in `artifacts.json`) to understand overall architecture, components, principles, and constraints before implementing code. The DESIGN provides essential context for making implementation decisions aligned with system architecture.

---

## Requirements

Agent confirms understanding of requirements:

### ID Format

All Cypilot IDs MUST:

- Use format: `cpt-{hierarchy-prefix}-{kind}-{slug}`
- Match regex: `^cpt-[a-z0-9][a-z0-9-]+$`
- Be wrapped in backticks: `` `cpt-...` ``
- Use only lowercase `a-z`, digits `0-9`, and `-` (kebab-case)

**ID definition** examples:

```text
**ID**: `cpt-...`
- [ ] `p1` - **ID**: `cpt-...`
```

**ID reference** examples:

```text
`cpt-...`
[x] `p1` - `cpt-...`
```

Any inline `` `cpt-...` `` in text is treated as an ID reference.

### Structural Requirements

- [ ] Code implements FEATURE design requirements
- [ ] Code follows project conventions from adapter

### Traceability Requirements

**Reference**: `{cypilot_path}/requirements/traceability.md` for full specification

- [ ] Traceability Mode determined per spec (FULL vs DOCS-ONLY)
- [ ] If Mode ON: markers follow spec syntax (`@cpt-*`, `@cpt-begin`/`@cpt-end`)
- [ ] If Mode ON: all CDSL instructions have corresponding markers `@cpt-begin`/`@cpt-end`
- [ ] If Mode ON: every implemented CDSL instruction (`[x] ... `inst-*``) has a paired `@cpt-begin/.../@cpt-end` block marker in code
- [ ] If Mode ON: no orphaned/stale markers
- [ ] If Mode ON: design checkboxes synced with code
- [ ] If Mode OFF: no Cypilot markers in code

### Checkbox Cascade (Code Markers → Upstream Artifacts)

CODE implementation triggers upstream checkbox updates through markers:

| Code Marker | Upstream Effect |
|-------------|-----------------|
| `@cpt-flow:{cpt-id}:p{N}` | When all pN markers exist → check  in FEATURE |
| `@cpt-algo:{cpt-id}:p{N}` | When all pN markers exist → check  in FEATURE |
| `@cpt-state:{cpt-id}:p{N}` | When all pN markers exist → check in FEATURE |
| `@cpt-req:{cpt-id}:p{N}` | When all pN markers exist + tests pass → check in FEATURE |

**Full Cascade Chain**:

```
CODE markers exist
    ↓
FEATURE: slug:flow/algo/state/req → [x]
    ↓
FEATURE: ALL IDs [x] → [x] in DECOMPOSITION
    ↓
DECOMPOSITION: [x] → (fr, principle, component, etc.) → [x]
    ↓
PRD: fr/nfr [x] when ALL downstream refs [x]
DESIGN: principle/constraint/component/seq/dbtable [x] when ALL refs [x]
```

**When to Update Upstream Checkboxes**:

1. **After implementing CDSL instruction**:
   - Add `@cpt-begin:{cpt-id}:p{N}:inst-{slug}` / `@cpt-end:...` markers
   - Mark corresponding CDSL step `[x]` in FEATURE

**Consistency rule (MANDATORY)**:
- [ ] Never mark an CDSL instruction `[x]` unless the corresponding code block markers exist and wrap non-empty implementation code
- [ ] Never add a code block marker pair unless the corresponding CDSL instruction exists in the design source (add it first if missing)

2. **After completing flow/algo/state/req**:
   - All CDSL steps marked `[x]` → mark as `[x]` in FEATURE

3. **After completing FEATURE**:
   - All `cpt-{system}-feature-{slug}` in FEATURE are `[x]` → mark as `[x]` in DECOMPOSITION
   - Update feature status: `⏳ PLANNED` → `🔄 IN_PROGRESS` → `✅ IMPLEMENTED`

4. **After DECOMPOSITION updated**:
   - Check if all `cpt-{system}-*` in DECOMPOSITION are `[x]`
   - If all refs for a PRD/DESIGN ID are `[x]` → mark that ID as `[x]` in PRD/DESIGN

**Validation Checks**:
- Prompt `cypilot validate` will warn if code marker exists but FEATURE checkbox is `[ ]`
- Prompt `cypilot validate` will warn if FEATURE checkbox is `[x]` but code marker is missing
- Prompt `cypilot validate` will report coverage: N% of FEATURE IDs have code markers

### Versioning Requirements

- [ ] When design ID versioned (`-v2`): update code markers to match
- [ ] Marker format with version: `@cpt-flow:{cpt-id}-v2:p{N}`
- [ ] Migration: update all markers when design version increments
- [ ] Keep old markers commented during transition (optional)

### Engineering Best Practices (MANDATORY)

- [ ] **TDD**: Write failing test first, implement minimal code to pass, then refactor
- [ ] **SOLID**:
  - Single Responsibility: Each module/function focused on one reason to change
  - Open/Closed: Extend behavior via composition/configuration, not editing unrelated logic
  - Liskov Substitution: Implementations honor interface contract and invariants
  - Interface Segregation: Prefer small, purpose-driven interfaces over broad ones
  - Dependency Inversion: Depend on abstractions; inject dependencies for testability
- [ ] **DRY**: Remove duplication by extracting shared logic with clear ownership
- [ ] **KISS**: Prefer simplest correct solution matching design and adapter conventions
- [ ] **YAGNI**: No specs/abstractions not required by current design scope
- [ ] **Refactoring discipline**: Refactor only after tests pass; keep behavior unchanged
- [ ] **Testability**: Structure code so core logic is testable without heavy integration
- [ ] **Error handling**: Fail explicitly with clear errors; never silently ignore failures
- [ ] **Observability**: Log meaningful events at integration boundaries (no secrets)

### Quality Requirements

**Reference**: `checklist.md` for detailed criteria

- [ ] Code passes quality checklist
- [ ] Functions/methods are appropriately sized
- [ ] Error handling is consistent
- [ ] Tests cover implemented requirements

---

## Tasks

Agent executes tasks during generation:

### Phase 1: Setup

**1.1 Resolve Source**

Ask user for implementation source (if not provided):

| Source Type | Traceability | Action |
|-------------|--------------|--------|
| FEATURE artifact (registered) | DOCS-ONLY | Load artifact, extract requirements |
| Other Cypilot artifact (PRD/DESIGN/ADR) | DOCS-ONLY | Load artifact, extract requirements |
| User-provided spec/description | DOCS-ONLY | Use as requirements reference |
| Prompt only | DOCS-ONLY | Implement per user instructions |
| None | — | Suggest: create a FEATURE artifact first |

**1.2 Load Context**

- [ ] Read adapter `AGENTS.md` for code conventions
- [ ] Load source artifact/description
- [ ] Load `checklist.md` for quality guidance
- [ ] Determine Traceability Mode (see Requirements)
- [ ] Plan implementation order (by requirement, flow, or phase)

### Phase 2: Implementation (Work Packages)

Choose implementation order based on FEATURE design:
- One requirement end-to-end, or
- One flow/algo/state section end-to-end, or
- One phase at a time if design defines phases

**For each work package:**

1. Identify exact design items to code (flows/algos/states/requirements/tests)
2. Implement according to adapter conventions
3. **If Traceability Mode ON**: Add instruction-level tags while implementing
4. Run work package validation (tests, build, linters per adapter)
5. **If Traceability Mode ON**: Update FEATURE checkboxes
6. Proceed to next work package

**Partial Implementation Handling**:

If implementation cannot be completed in a single session:

1. **Checkpoint progress**:
   - Note completed work packages with their IDs
   - Note current work package state (which steps done)
   - List remaining work packages
2. **Ensure valid intermediate state**:
   - All completed work packages must pass validation
   - Current work package: either complete or revert to last valid state
   - Do NOT leave partially implemented code without markers
3. **Document resumption point**:
   ```
   Implementation checkpoint:
   - Completed: {list of IDs}
   - In progress: {current ID, steps done}
   - Remaining: {list of IDs}
   - Resume command: /cypilot-generate CODE --continue {feature-id}
   ```
4. **On resume**:
   - Verify checkpoint state still valid (design unchanged)
   - Continue from documented resumption point
   - If design changed: restart affected work packages

### Phase 3: Cypilot Markers (Traceability Mode ON only)

**Reference**: `{cypilot_path}/requirements/traceability.md` for full marker syntax

**Apply markers per spec:**
- Scope markers: `@cpt-{kind}:{cpt-id}:p{N}` at function/class entry
- Block markers: `@cpt-begin:{cpt-id}:p{N}:inst-{local}` / `@cpt-end:...` wrapping CDSL steps

**Quick reference:**
```python
# @cpt-begin:cpt-myapp-feature-auth-flow-login:p1:inst-validate-creds
def validate_credentials(username, password):
    # implementation here
    pass
# @cpt-end:cpt-myapp-feature-auth-flow-login:p1:inst-validate-creds
```

### Phase 4: Sync FEATURE (Traceability Mode ON only)

**After each work package, sync checkboxes:**

1. For each `...:p{N}:inst-{local}` implemented:
   - Locate owning scope entry in the FEATURE artifact by base ID
   - Find matching CDSL step line with `p{N}` and `inst-{local}`
   - Mark checkbox: `- [ ]` → `- [x]`

2. For each requirement ID implemented:
   - First work package for requirement: set `**Status**` to `🔄 IN_PROGRESS`
   - Mark `**Phases**` checkboxes as implemented
   - All phases complete: set `**Status**` to `✅ IMPLEMENTED`

3. For test scenarios:
   - Do NOT mark until test exists and passes

**Consistency rule**: Only mark `[x]` if corresponding code exists and is tagged

### Phase 5: Quality Check

- [ ] Self-review against `checklist.md`
- [ ] **If Traceability Mode ON**: Verify all `cpt-{system}-{req|flow|algo|state|test}-*` IDs have markers
- [ ] **If Traceability Mode ON**: Ensure no orphaned markers
- [ ] Run tests to verify implementation
- [ ] Verify engineering best practices followed

### Phase 6: Tag Verification (Traceability Mode ON only)

**Before finishing implementation:**
- [ ] Search codebase for ALL IDs from DESIGN (flow/algo/state/req/test)
- [ ] Confirm tags exist in files that implement corresponding logic/tests
- [ ] If any DESIGN ID has no code tag → report as gap and/or add tag

### When Updating Existing Code

- [ ] Preserve existing Cypilot markers
- [ ] Add markers for new design elements
- [ ] Remove markers for deleted design elements
- [ ] Update marker IDs if design IDs changed (with migration)

---

## Validation

Validation workflow verifies requirements are met:

### Phase 1: Implementation Coverage

For each ID/scope marked as implemented:

**Verify code exists:**
- [ ] Code files exist and contain implementation
- [ ] Code is not placeholder/stub (no TODO/FIXME/unimplemented!)
- [ ] No unimplemented!() in business logic

### Phase 2: Traceability Validation (Mode ON only)

**Reference**: `{cypilot_path}/requirements/traceability.md` for validation rules

**Deterministic checks** (per spec):
- [ ] Marker format valid
- [ ] All begin/end pairs matched
- [ ] No empty blocks
- [ ] Phase postfix present on all markers

**Coverage checks**:
- [ ] All `cpt-{system}-{req|flow|algo|state|test}-*` IDs have markers
- [ ] No orphaned markers (marker ID not in design)
- [ ] No stale markers (design ID changed/deleted)
- [ ] Design checkboxes synced with code markers

### Phase 3: Test Scenarios Validation

For each test scenario from design:

- [ ] Test file exists (unit/integration/e2e per adapter)
- [ ] Test contains scenario ID in comment for traceability
- [ ] Test is NOT ignored without justification
- [ ] Test actually validates scenario behavior (not placeholder)
- [ ] Test follows adapter testing conventions

### Phase 4: Build and Lint Validation

**Build:**
- [ ] Build succeeds
- [ ] No compilation errors
- [ ] No compiler warnings (or acceptable per adapter)

**Lint:**
- [ ] Linter passes
- [ ] No linter errors
- [ ] No linter warnings (or acceptable per adapter)

### Phase 5: Test Execution

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] All e2e tests pass (if applicable)
- [ ] No ignored tests without justification
- [ ] Coverage meets adapter requirements

### Phase 6: Code Quality Validation

**Check for incomplete work:**
- [ ] No TODO/FIXME/XXX/HACK in domain/service layers
- [ ] No unimplemented!/todo! in business logic
- [ ] No bare unwrap() or panic in production code
- [ ] No ignored tests without documented reason
- [ ] No placeholder tests (assert!(true))

**Engineering best practices:**
- [ ] TDD: New/changed behavior covered by tests
- [ ] SOLID: Responsibilities separated; dependencies injectable
- [ ] DRY: No copy-paste duplication without justification
- [ ] KISS: No unnecessary complexity
- [ ] YAGNI: No speculative abstractions beyond design scope

### Phase 7: Code Logic Consistency with Design

**For each requirement marked IMPLEMENTED:**
- [ ] Read requirement specification
- [ ] Locate implementing code via @cpt-req tags
- [ ] Verify code logic matches requirement (no contradictions)
- [ ] Verify no skipped mandatory steps
- [ ] Verify error handling matches design error specifications

**For each flow marked implemented:**
- [ ] All flow steps executed in correct order
- [ ] No steps bypassed that would change behavior
- [ ] Conditional logic matches design conditions
- [ ] Error paths match design error handling

**For each algorithm marked implemented:**
- [ ] Algorithm logic matches design specification
- [ ] Performance characteristics match design (O(n), O(1), etc.)
- [ ] Edge cases handled as designed
- [ ] No logic shortcuts that violate design constraints

### Traceability Report

**Format**: See `{cypilot_path}/requirements/traceability.md` → Validation Report

### Quality Report

Output format:
```
Code Quality Report
═══════════════════

Build: PASS/FAIL
Lint: PASS/FAIL
Tests: X/Y passed
Coverage: N%

Checklist: PASS/FAIL (N issues)

Issues:
- [SEVERITY] CHECKLIST-ID: Description

Logic Consistency: PASS/FAIL
- CRITICAL divergences: [...]
- MINOR divergences: [...]
```

### PASS/FAIL Criteria

**PASS only if:**
- Build/lint/tests pass per adapter
- Coverage meets adapter requirements
- No CRITICAL divergences between code and design
- If Traceability Mode ON: required tags present and properly paired

### Phase 8: Semantic Expert Review (Always)

Run expert panel review after producing validation output.

**Review Scope Selection**:

| Change Size | Review Mode | Experts |
|-------------|-------------|---------|
| <50 LOC, single concern | Abbreviated | Developer + 1 relevant expert |
| 50-200 LOC, multiple concerns | Standard | Developer, QA, Security, Architect |
| >200 LOC or architectural | Full | All 8 experts |

**Abbreviated Review** (for small, focused changes):
1. Developer reviews code quality and design alignment
2. Select ONE additional expert based on change type:
   - Security changes → Security Expert
   - Performance changes → Performance Engineer
   - Database changes → Database Architect/Data Engineer
   - Infrastructure changes → DevOps Engineer
   - Test changes → QA Engineer
3. Skip remaining experts with note: `Abbreviated review: {N} LOC, single concern`

**Full Expert Panel**:
- Developer, QA Engineer, Security Expert, Performance Engineer
- DevOps Engineer, Architect, Monitoring Engineer
- Database Architect, Data Engineer

**For EACH expert:**
1. Adopt role (write: `Role assumed: {expert}`)
2. Review actual code and tests in validation scope
3. If design artifact available: evaluate design-to-code alignment
4. Identify issues:
   - Contradictions vs design intent
   - Missing behavior (requirements/tests)
   - Unclear intent (naming/structure)
   - Unnecessary complexity (YAGNI, premature abstraction)
   - Missing non-functional concerns (security/perf/observability)
5. Provide concrete proposals:
   - What to remove (dead code, unused abstractions)
   - What to add (tests, error handling, validation)
   - What to rewrite (simpler structure, clearer naming)
6. Propose corrective workflow:
   - If design must change: `feature` or `design` (UPDATE mode)
   - If only code must change: `code` (continue implementation)

**Output format:**
```
### Semantic Expert Review

**Review status**: COMPLETED
**Reviewed artifact**: Code ({scope})

#### Expert: {expert}
- **Role assumed**: {expert}
- **Checklist completed**: YES
- **Findings**:
  - Contradictions: ...
  - Missing behavior: ...
  - Unclear intent: ...
  - Unnecessary complexity: ...
- **Proposed edits**:
  - Remove: "..." → Reason: ...
  - Add: ...
  - Rewrite: "..." → "..."

**Recommended corrective workflow**: {feature | design | code}
```

---

## Next Steps

After code generation/validation, offer these options to user:

### After Successful Implementation

| Condition | Suggested Next Step |
|-----------|---------------------|
| Feature complete | Update feature status to IMPLEMENTED in DECOMPOSITION |
| All features done | `/cypilot-analyze DESIGN` — validate overall design completion |
| New feature needed | Design and add the next FEATURE artifact |
| Want expert review only | `/cypilot-analyze semantic` — semantic validation (skip deterministic) |

### After Validation Issues

| Issue Type | Suggested Next Step |
|------------|---------------------|
| Design mismatch | Update FEATURE artifact and reconcile implementation |
| Missing tests | Continue `/cypilot-generate CODE` — add tests |
| Code quality issues | Continue `/cypilot-generate CODE` — refactor |

### If No Design Exists

| Scenario | Suggested Next Step |
|----------|---------------------|
| Implementing new feature | Create FEATURE design first |
| Implementing from PRD | `/cypilot-generate DESIGN` then `/cypilot-generate DECOMPOSITION` — create design hierarchy |
| Quick prototype | Proceed without traceability, suggest a FEATURE artifact later |
