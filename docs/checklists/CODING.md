---
spider: true
type: requirement
name: Code Quality Expert Checklist
version: 1.0
purpose: Generic (weaver-agnostic) quality checklist for code changes and reviews
---

# Code Quality Expert Checklist (Generic)

## Overview

Generic (weaver-agnostic) checklist for reviewing code changes. Use it to produce an issues-only report and to drive consistent, high-signal code reviews across any language and project.

**Scope**: Any codebase (language-agnostic). This file contains only generic code-quality criteria.

**Output contract**:
- Produce an **issues-only** report using the formats in [Reporting](#reporting).
- For each issue: include checklist ID, severity, location, evidence, why it matters, and a concrete fix.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Applicability Context](#applicability-context)
3. [Severity Dictionary](#severity-dictionary)
4. [Review Scope Selection](#review-scope-selection)
5. **MUST HAVE**
   - [Engineering Best Practices (ENG)](#engineering-best-practices-eng)
   - [Code Quality (QUAL)](#code-quality-qual)
   - [Error Handling (ERR)](#error-handling-err)
   - [Security (SEC)](#security-sec)
   - [Performance (PERF)](#performance-perf)
   - [Observability (OBS)](#observability-obs)
   - [Testing (TEST)](#testing-test)
6. **MUST NOT HAVE**
   - [No Incomplete Work Markers](#qual-code-no-001-no-incomplete-work-markers)
   - [No Placeholder Implementations](#qual-code-no-002-no-placeholder-implementations)
   - [No Silent Failures](#err-code-no-001-no-silent-failures)
   - [No Unsafe Panic Patterns](#err-code-no-002-no-unsafe-panic-patterns)
   - [No Ignored Tests](#test-code-no-001-no-ignored-tests)
   - [No Hardcoded Secrets](#sec-code-no-001-no-hardcoded-secrets)
   - [No Dangerous Patterns](#sec-code-no-002-no-dangerous-patterns)
7. [Validation Summary](#validation-summary)
8. [Conflict Resolution](#conflict-resolution)
9. [Reporting](#reporting)

---

## Prerequisites

Before starting the review, confirm:

- [ ] I understand this checklist validates CODE implementations
- [ ] I will follow the Applicability Context rules below
- [ ] I selected a Review Mode and will check all items required by that mode
- [ ] I will document any violations found
- [ ] I will provide specific feedback for each failed check
- [ ] I will complete the Final Checklist and provide a review report

---

## Applicability Context

Before evaluating each checklist item, the expert MUST:

1. **Understand the code's domain** — What kind of code is this? (e.g., business logic, infrastructure, tests, utilities, configuration)

2. **Determine applicability for each requirement** — Not all checklist items apply to all code:
   - Test code may have different error handling requirements
   - Configuration code may not need complex algorithms
   - Utility code may not need full observability

3. **Require explicit handling** — For each checklist item:
   - If applicable: The code MUST satisfy it
   - If not applicable: Document why (in review notes)
   - If missing without explanation: Report as violation

4. **Use explicit evaluation states** — For each checklist item in your report, use one of:
   - **FAIL**: Applicable and violated
   - **PASS**: Applicable and satisfied
   - **N/A**: Not applicable, with a written rationale
   - **NOT REVIEWED**: Not checked due to selected Review Mode scope

5. **Never skip silently** — The expert MUST NOT skip a requirement just because it seems irrelevant

**Key principle**: Code quality is about fitness for purpose, not checkbox compliance

---

## Severity Dictionary

- **CRITICAL**: Unsafe/broken/security vulnerability; blocks merge.
- **HIGH**: Major quality issue; should be fixed before merge.
- **MEDIUM**: Meaningful improvement; fix when feasible.
- **LOW**: Minor improvement; optional.

---

## Review Scope Selection

Select review depth based on change size and risk:

### Review Modes

| Change Size | Risk Level | Review Mode | Items to Check |
|-------------|------------|-------------|----------------|
| <50 LOC, single file | Low | **Quick** | Critical-tier only |
| 50-200 LOC, few files | Medium | **Standard** | Critical + High-tier |
| >200 LOC or architectural | High | **Full** | All items |

### Quick Review (Critical-Tier Only)

For small, low-risk changes, check only these items:

**MUST CHECK** (blocking):
- [ ] SEC-CODE-001: Injection Prevention
- [ ] SEC-CODE-002: Authentication & Authorization
- [ ] SEC-CODE-003: Data Protection
- [ ] SEC-CODE-NO-001: No Hardcoded Secrets
- [ ] SEC-CODE-NO-002: No Dangerous Patterns
- [ ] ERR-CODE-001: Explicit Error Handling
- [ ] ERR-CODE-003: Input Validation
- [ ] ERR-CODE-NO-001: No Silent Failures
- [ ] QUAL-CODE-NO-002: No Placeholder Implementations

**SPOT CHECK** (sample):
- [ ] ENG-CODE-001: TDD — at least one test for new behavior
- [ ] QUAL-CODE-001: Readability — naming is clear

Note: `Quick review: {N} LOC, checked Critical-tier items; other items = NOT REVIEWED`

### Standard Review (Critical + High-Tier)

For medium-sized changes, check all CRITICAL and HIGH severity items:

**All Quick Review items PLUS**:
- [ ] All HIGH severity items from MUST HAVE sections
- [ ] All items from MUST NOT HAVE sections

### Full Review

For large or architectural changes, check ALL items in this checklist.

### Triage Priority Within Full Review

When time-constrained during full review, prioritize in this order:

1. **First pass**: All CRITICAL items (security, error handling)
2. **Second pass**: All HIGH items (quality, testing, SOLID)
3. **Third pass**: All MEDIUM items (DRY, OCP, performance)
4. **Fourth pass**: All LOW items (metrics, tracing)

---

# MUST HAVE

---

## Engineering Best Practices (ENG)

### ENG-CODE-001: Test-Driven Development (TDD)
**Severity**: HIGH

- [ ] New behavior has corresponding tests
- [ ] Tests were written before or alongside implementation
- [ ] Tests fail when implementation is removed
- [ ] Tests verify expected outcomes (not just "no crash")
- [ ] Test names describe the behavior being tested
- [ ] Tests are independent and can run in any order

### ENG-CODE-002: Single Responsibility Principle (SRP)
**Severity**: HIGH

- [ ] Each module/class/function has one reason to change
- [ ] Functions do one thing and do it well
- [ ] Classes have a single, clear purpose
- [ ] No "god objects" or "kitchen sink" modules
- [ ] Responsibilities are separated by concern (UI, business logic, data access)

### ENG-CODE-003: Open/Closed Principle (OCP)
**Severity**: MEDIUM

- [ ] Behavior extended via composition/configuration, not modification
- [ ] New functionality doesn't require changing existing code
- [ ] Extension points are clear and intentional
- [ ] No modification of working code to add unrelated specs

### ENG-CODE-004: Liskov Substitution Principle (LSP)
**Severity**: HIGH

- [ ] Implementations honor interface contracts
- [ ] Subtypes are substitutable for their base types
- [ ] No surprising behavior when using polymorphism
- [ ] Preconditions are not strengthened in subtypes
- [ ] Postconditions are not weakened in subtypes

### ENG-CODE-005: Interface Segregation Principle (ISP)
**Severity**: MEDIUM

- [ ] Interfaces are small and purpose-driven
- [ ] No "fat" interfaces with methods clients don't use
- [ ] Clients depend only on what they need
- [ ] Role interfaces preferred over header interfaces

### ENG-CODE-006: Dependency Inversion Principle (DIP)
**Severity**: HIGH

- [ ] High-level modules don't depend on low-level modules
- [ ] Both depend on abstractions
- [ ] Dependencies are injectable
- [ ] Core logic is testable without heavy integration setup
- [ ] External dependencies are behind interfaces

### ENG-CODE-007: Don't Repeat Yourself (DRY)
**Severity**: MEDIUM

- [ ] No copy-paste duplication
- [ ] Shared logic extracted with clear ownership
- [ ] Duplication removed only when patterns are clear (rule of three)
- [ ] Constants defined once, not scattered
- [ ] Common patterns abstracted appropriately

### ENG-CODE-008: Keep It Simple, Stupid (KISS)
**Severity**: HIGH

- [ ] Simplest correct solution chosen
- [ ] No unnecessary complexity
- [ ] Code is readable without extensive documentation
- [ ] Clever tricks avoided in favor of clarity
- [ ] Standard patterns preferred over novel approaches

### ENG-CODE-009: You Aren't Gonna Need It (YAGNI)
**Severity**: HIGH

- [ ] No speculative specs
- [ ] No unused abstractions
- [ ] No configuration for hypothetical scenarios
- [ ] No extension points without current use cases
- [ ] Specs added only when needed, not "just in case"

### ENG-CODE-010: Refactoring Discipline
**Severity**: MEDIUM

- [ ] Refactoring done only after tests pass
- [ ] Behavior unchanged during refactoring
- [ ] Structure improved without adding specs
- [ ] Small, incremental refactoring steps
- [ ] No mixing refactoring with spec work in same commit

---

## Code Quality (QUAL)

### QUAL-CODE-001: Readability
**Severity**: HIGH

- [ ] Clear, descriptive naming (variables, functions, classes)
- [ ] Consistent naming conventions
- [ ] Code reads like well-written prose
- [ ] Complex logic has explanatory comments
- [ ] No misleading names or abbreviations

### QUAL-CODE-002: Maintainability
**Severity**: HIGH

- [ ] Code is easy to modify
- [ ] Changes are localized (no ripple effects)
- [ ] Dependencies are explicit and minimal
- [ ] No hidden coupling between modules
- [ ] Clear module boundaries

### QUAL-CODE-003: Testability
**Severity**: HIGH

- [ ] Core logic testable without external dependencies
- [ ] Dependencies injectable for testing
- [ ] Side effects isolated and mockable
- [ ] Deterministic behavior (no random/time dependencies in logic)
- [ ] Observable outcomes (not just internal state)

---

## Error Handling (ERR)

### ERR-CODE-001: Explicit Error Handling
**Severity**: CRITICAL

- [ ] Errors fail explicitly, not silently
- [ ] Error conditions identified and handled
- [ ] No swallowed exceptions
- [ ] Error messages are clear and actionable
- [ ] Stack traces available for debugging (not in production UI)

### ERR-CODE-002: Graceful Degradation
**Severity**: HIGH

- [ ] Partial failures handled appropriately
- [ ] Recovery actions documented
- [ ] Fallback behavior defined
- [ ] User-facing errors are friendly
- [ ] System-facing errors are detailed

### ERR-CODE-003: Input Validation
**Severity**: CRITICAL

- [ ] All external inputs validated at system boundaries
- [ ] Validation rules clear and consistent
- [ ] Invalid input rejected early
- [ ] Validation errors are specific and helpful
- [ ] No trusting of internal code (validate at boundaries only)

---

## Security (SEC)

### SEC-CODE-001: Injection Prevention
**Severity**: CRITICAL

- [ ] No string concatenation for queries (parameterized queries)
- [ ] No command injection vulnerabilities
- [ ] No XSS vulnerabilities (output encoding)
- [ ] No path traversal vulnerabilities
- [ ] User input never used directly in dangerous contexts

### SEC-CODE-002: Authentication & Authorization
**Severity**: CRITICAL

- [ ] Authentication checks present where required by the design/spec (routes, handlers, entrypoints)
- [ ] Authorization checks present for operations that mutate or access protected resources (as defined by design/spec)
- [ ] No privilege escalation vulnerabilities
- [ ] Session management secure
- [ ] Credentials not hardcoded

### SEC-CODE-003: Data Protection
**Severity**: CRITICAL

- [ ] Sensitive data not logged
- [ ] PII handled appropriately
- [ ] Secrets not in code
- [ ] Encryption used where required
- [ ] Secure transmission for sensitive data

---

## Performance (PERF)

### PERF-CODE-001: Efficiency
**Severity**: MEDIUM

- [ ] No obvious performance anti-patterns
- [ ] N+1 query patterns avoided
- [ ] Unnecessary allocations avoided
- [ ] Resources cleaned up properly
- [ ] Appropriate data structures chosen

### PERF-CODE-002: Scalability
**Severity**: MEDIUM

- [ ] Algorithm complexity appropriate for data size
- [ ] No blocking operations in hot paths
- [ ] Caching used where beneficial
- [ ] Batch operations used where appropriate
- [ ] Pagination implemented for large datasets

---

## Observability (OBS)

### OBS-CODE-001: Logging
**Severity**: MEDIUM

- [ ] Meaningful events logged at integration boundaries
- [ ] Log levels used appropriately (DEBUG, INFO, WARN, ERROR)
- [ ] No secrets in logs
- [ ] Correlation IDs propagated
- [ ] Sufficient context for debugging

### OBS-CODE-002: Metrics & Tracing
**Severity**: LOW (when applicable)

- [ ] Key operations have metrics
- [ ] Tracing integrated where beneficial
- [ ] Health checks implemented
- [ ] Alertable conditions identified
- [ ] Performance baselines established

**Applicability note**: Mark this item **N/A** unless the service is long-running or has operational SLO/SLA requirements.

---

## Testing (TEST)

### TEST-CODE-001: Test Coverage
**Severity**: HIGH

- [ ] All public APIs have tests
- [ ] Happy path tested
- [ ] Error paths tested
- [ ] Edge cases tested
- [ ] Boundary conditions tested

### TEST-CODE-002: Test Quality
**Severity**: HIGH

- [ ] Tests are fast
- [ ] Tests are reliable (no flaky tests)
- [ ] Tests are independent
- [ ] Tests are readable
- [ ] Tests have clear assertions

### TEST-CODE-003: Test Completeness
**Severity**: MEDIUM

- [ ] Unit tests for business logic
- [ ] Integration tests for external dependencies
- [ ] E2E tests for critical paths (when applicable)
- [ ] Tests cover regression scenarios
- [ ] Tests document expected behavior

---

# MUST NOT HAVE

---

## QUAL-CODE-NO-001: No Incomplete Work Markers
**Severity**: HIGH

**What to check**:
- [ ] No TODO in production code (without ticket reference)
- [ ] No FIXME in production code
- [ ] No XXX markers
- [ ] No HACK markers
- [ ] No "temporary" solutions that became permanent

**Action**: Either complete the work or create a tracked issue

---

## QUAL-CODE-NO-002: No Placeholder Implementations
**Severity**: CRITICAL

**What to check**:
- [ ] No `unimplemented!()` / `todo!()` in business logic
- [ ] No `throw new NotImplementedException()` in production paths
- [ ] No `pass` with `# TODO` in Python
- [ ] No empty catch blocks
- [ ] No stub methods that do nothing

**Action**: Implement or remove

---

## ERR-CODE-NO-001: No Silent Failures
**Severity**: CRITICAL

**What to check**:
- [ ] No empty catch blocks
- [ ] No swallowed exceptions
- [ ] No ignored return values for fallible operations
- [ ] No `_ = might_fail()` patterns without handling
- [ ] No `try: ... except: pass` patterns

**Action**: Handle or propagate errors explicitly

---

## ERR-CODE-NO-002: No Unsafe Panic Patterns
**Severity**: HIGH

**What to check**:
- [ ] No bare `unwrap()` on Results/Options in production code
- [ ] No bare `panic!()` in production code
- [ ] No `expect()` without meaningful message
- [ ] No force-unwrapping optionals without guards
- [ ] No assertions in production code paths

**Action**: Use proper error handling

---

## TEST-CODE-NO-001: No Ignored Tests
**Severity**: MEDIUM

**What to check**:
- [ ] No `#[ignore]` without documented reason
- [ ] No `@Disabled` without documented reason
- [ ] No `skip` markers without explanation
- [ ] No commented-out tests
- [ ] No placeholder tests (`assert!(true)`, `assertTrue(true)`)

**Action**: Fix or remove the test

---

## SEC-CODE-NO-001: No Hardcoded Secrets
**Severity**: CRITICAL

**What to check**:
- [ ] No API keys in code
- [ ] No passwords in code
- [ ] No tokens in code
- [ ] No connection strings with credentials
- [ ] No private keys in code

**Action**: Use environment variables or secret management

---

## SEC-CODE-NO-002: No Dangerous Patterns
**Severity**: CRITICAL

**What to check**:
- [ ] No `eval()` with user input
- [ ] No `exec()` with user input
- [ ] No `system()` with user input
- [ ] No `innerHTML` with user input
- [ ] No SQL string concatenation

**Action**: Use safe alternatives

---

# Validation Summary

## Final Checklist

Confirm before reporting results:

- [ ] I checked ALL items in MUST HAVE sections
- [ ] I verified ALL items in MUST NOT HAVE sections
- [ ] I ran all tests and they pass
- [ ] I ran linters and they pass
- [ ] I documented all violations found
- [ ] I provided specific feedback for each failed check
- [ ] All critical issues have been reported

### Build & Test Verification

- [ ] Code compiles without errors
- [ ] Code compiles without warnings (or warnings are acceptable)
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] All E2E tests pass (if applicable)
- [ ] Linter passes
- [ ] Coverage meets requirements (if specified)

---

## Conflict Resolution

When checklist items appear to conflict, use this resolution order:

### Priority Order (highest to lowest)

1. **Security (SEC)** — Always takes precedence
2. **Error Handling (ERR)** — Explicit failures over silent issues
3. **Correctness (QUAL, TEST)** — Working code over elegant code
4. **Maintainability (ENG, QUAL)** — Clarity over cleverness
5. **Performance (PERF)** — Only after correctness established
6. **Observability (OBS)** — Useful but not at expense of security

### Common Conflicts

| Conflict | Resolution |
|----------|------------|
| DRY vs KISS | Prefer KISS — duplication is better than wrong abstraction |
| YAGNI vs OCP | Prefer YAGNI — don't add extension points "just in case" |
| Performance vs Readability | Prefer readability; optimize only with evidence |
| Test coverage vs Test speed | Prefer coverage for critical paths; speed for CI |
| Verbose errors vs User experience | Detailed logs, friendly user messages |

### When Uncertain

1. Ask: "What failure mode is worse?"
2. Security/data loss > inconvenience > performance
3. Document the trade-off decision in code comments

---

## Reporting Readiness Checklist

- [ ] I will report every identified issue (no omissions)
- [ ] I will report only issues (no "everything looks good" sections)
- [ ] Each reported issue will include Evidence (code location/quote)
- [ ] Each reported issue will include Why it matters (impact)
- [ ] Each reported issue will include a Proposal (concrete fix)
- [ ] I will avoid vague statements and use precise, verifiable language

---

## Reporting

Report **only** problems (do not list what is OK).

### Format Selection

| Review Mode | Report Format |
|-------------|---------------|
| Quick | Compact (table) |
| Standard | Compact or Full |
| Full | Full (detailed) |

### Compact Format (for Quick/Standard reviews)

```markdown
## Code Review: {scope}

| # | ID | Sev | Location | Issue | Fix |
|---|-----|-----|----------|-------|-----|
| 1 | SEC-001 | CRIT | file.py:42 | SQL injection | Use parameterized query |
| 2 | ERR-001 | HIGH | file.py:87 | Silent catch | Add logging |

**Review mode**: Quick ({N} LOC)
```

### Full Format (for detailed reviews)

For each issue include:

- **Issue**: What is wrong
- **Location**: File path and line number(s)
- **Evidence**: Code snippet or description
- **Why it matters**: Impact (risk, maintainability, security)
- **Proposal**: Concrete fix

Full output format:

```markdown
## Code Review Report (Issues Only)

### 1. {Short issue title}

**Checklist Item**: `{CHECKLIST-ID}` — {Checklist item title}

**Severity**: CRITICAL|HIGH|MEDIUM|LOW

#### Location

`{file_path}:{line_number}`

#### Issue

{What is wrong}

#### Evidence

```{language}
{code snippet}
```

#### Why It Matters

{Impact: risk, maintainability, security, performance}

#### Proposal

{Concrete fix: what to change}

---

### 2. {Short issue title}

...
```

---

## Reporting Commitment

- [ ] I reported all issues I found
- [ ] I used the exact report format defined in this checklist
- [ ] I included evidence and impact for each issue
- [ ] I proposed concrete fixes for each issue
- [ ] I did not hide or omit known problems
- [ ] I am ready to iterate on the proposals and re-review after changes
