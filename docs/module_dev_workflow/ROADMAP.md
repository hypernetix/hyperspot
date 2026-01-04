# Module Development Roadmap

This document tracks planned enhancements to the module development workflow.

For the current workflow, see [Module Development Workflow](./README.md).

## Index

- [1. Verification Flows](#1-verification-flows)
  - [Verification Types](#verification-types)
  - [Verification Output Format](#verification-output-format)
  - [When to Run Verifications](#when-to-run-verifications)
  - [Implementation Plan](#implementation-plan)
- [2. Migration to Agent Skills](#2-migration-to-agent-skills)

---

## 1. Verification Flows

Define automated verification workflows to ensure consistency across documentation, specifications, and code.

### Verification Types

#### a) OpenSpec Specs vs. Design and Requirements

**Purpose:** Verify that OpenSpec scenarios align with DESIGN.md architecture and REQUIREMENTS.md definitions.

**What to verify:**
- All requirements in REQUIREMENTS.md marked as implemented in IMPLEMENTATION_PLAN.md have corresponding scenarios in OpenSpec specs
- Scenarios reference correct requirement IDs
- No orphaned scenarios (scenarios without corresponding requirements)
- Scenario coverage matches implementation phase scope from DESIGN.md

#### b) Source Code vs. OpenSpec Specs and Requirements

**Purpose:** Verify that implementation matches documented specifications and requirements.

**What to verify:**
- All scenarios from OpenSpec specs are actually implemented
- All scenarios from OpenSpec specs are covered by e2e tests
- REST endpoints match OpenSpec scenario descriptions
- Data models in code match models described in requirements
- Error handling matches scenarios (e.g., 404, 400, 500 cases)
- Security requirements (#tenant-isolation, #rbac) are implemented

#### c) Source Code vs. Design

**Purpose:** Verify that code structure follows DESIGN.md architecture.

**What to verify:**
- Module layer structure matches DESIGN.md (contract, api, domain, infra)
- Components described in DESIGN.md exist in code
- Integration points (ClientHub dependencies, REST routes) match design
- Data flow implementation follows DESIGN.md diagrams

### Verification Output Format

Verification reports should be written as Markdown documents with the following structure:

```markdown
# Verification Report: Specs vs. Requirements

**Module:** `types_registry`  
**Type:** Specs vs. Requirements  
**Status:** ❌ Failed  
**Timestamp:** 2025-01-01 12:00:00 UTC

## Summary

| Metric | Count |
|--------|-------|
| Total Requirements | 12 |
| Requirements with Scenarios | 10 |
| Missing Scenarios | 2 |
| Orphaned Scenarios | 0 |

## Issues

### ❌ Error: Missing Scenario

**Requirement:** #typereg/entity-lookup  
**Description:** Requirement #typereg/entity-lookup has no corresponding scenarios  
**Location:** [REQUIREMENTS.md:45](file:///path/to/modules/types_registry/docs/REQUIREMENTS.md#L45)

---

### ⚠️ Warning: Scenario Mismatch

**Scenario:** "List entities with invalid token"  
**Description:** Scenario references non-existent requirement #typereg/invalid-req  
**Location:** [spec.md:120](file:///path/to/openspec/specs/types-registry/spec.md#L120)

---

## Status Legend

- ✅ **Passed** — No issues found
- ⚠️ **Warning** — Non-critical issues that should be addressed
- ❌ **Failed** — Critical issues that must be fixed
```

### When to Run Verifications

| Timing | Verification | Goal |
|--------|--------------|------|
| During Design Review (Step 1.4) | Design self-consistency | Catch design issues before implementation |
| Before Creating OpenSpec Proposal | Requirements vs. Design | Ensure new requirements align with design |
| After Archiving OpenSpec Change | Specs vs. Requirements | Ensure all scenarios are documented and linked |
| Before PR Merge | All verifications (a, b, c) | Comprehensive consistency check |
| CI/CD Pipeline | Code vs. OpenSpec Specs (b) | Continuous verification |

### Implementation Plan

- Create verification scripts in `tools/verify/`
- Integrate with `make verify` command
- Add to CI/CD pipeline with configurable severity thresholds
- Generate reports as Markdown files in `docs/verification/`

---

## 2. Migration to Agent Skills

Consider migrating from `./prompts/` templates to [Agent Skills](https://agentskills.io/home) for more dynamic, reusable AI workflows.

### Open Questions

- Agent Skills are currently supported by Claude Code, Codex CLI, GitHub Copilot, and Cursor (nightly). What about other tools (Windsurf, Antigravity, etc.)?

---

*This roadmap will be updated as improvements are planned, designed, and implemented.*
