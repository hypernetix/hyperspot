# Module Development Roadmap

This document tracks planned enhancements to the module development workflow.

For the current workflow, see [Module Development Workflow](./README.md).

## Index

- [1. Verification Automation](#1-verification-automation)
  - [Current Implementation](#current-implementation)
  - [Future: Automation Scripts](#future-automation-scripts)
  - [Future: CI/CD Integration](#future-cicd-integration)
  - [Future: Periodic Global Verification](#future-periodic-global-verification)
- [2. Migration to Agent Skills](#2-migration-to-agent-skills)

---

## 1. Verification Automation

> **Status:** Verification is implemented via AI prompts in Steps 2.3, 3.1, and 3.2. This section tracks future automation work.

### Current Implementation

Verification is currently performed using AI prompts:
- **Step 2.3:** [`verify_specs_vs_requirements.md`](./prompts/verify_specs_vs_requirements.md)
- **Step 3.1:** [`verify_code_vs_specs_and_requirements.md`](./prompts/verify_code_vs_specs_and_requirements.md)
- **Step 3.2:** [`verify_code_vs_design.md`](./prompts/verify_code_vs_design.md)

Reports are stored in `modules/{module}/docs/verification/{change-name}/`.

### Future: Automation Scripts

Create scripts to automate structural verification checks that don't require AI:

**Potential scripts (Python or Rust):**

| Script | Purpose |
|--------|---------|
| `verify-requirement-refs` | Check all `#module/name` references point to existing requirements |
| `verify-scenario-coverage` | Ensure all completed requirements have scenarios |
| `verify-id-formats` | Validate ID format consistency (`#{module}/P{N}`, `#{module}/{name}`) |
| `verify-cross-refs` | Check IMPLEMENTATION_PLAN.md ↔ REQUIREMENTS.md alignment |

**Location:** `tools/verify/`

**Integration:**
```bash
# Run all verification scripts
make verify

# Run specific check
make verify-requirement-refs
```

### Future: CI/CD Integration

- Add verification to PR checks
- Configurable severity thresholds (fail on errors, warn on warnings)
- Auto-generate verification reports on PR creation

### Future: Periodic Global Verification

Run verification across the entire module (not just per-change) to catch drift over time:

**Scope:**
- Verify ALL specs match ALL requirements across the module
- Verify ALL code matches ALL archived specs
- Detect orphaned requirements (no scenarios), orphaned specs (no code), undocumented code

**Potential triggers:**
- Scheduled (weekly/monthly)
- Before major releases
- On-demand via `make verify-all`

**Reports:**
- `modules/{module}/docs/verification/specs_vs_requirements.md`
- `modules/{module}/docs/verification/code_vs_specs.md`
- `modules/{module}/docs/verification/code_vs_design.md`

**Use cases:**
- Catch drift from incremental changes
- Validate module consistency before 1.0 release
- Audit existing modules for documentation gaps

---

## 2. Migration to Agent Skills

Consider migrating from `./prompts/` templates to [Agent Skills](https://agentskills.io/home) for more dynamic, reusable AI workflows.

### Open Questions

- Agent Skills are currently supported by Claude Code, Codex CLI, GitHub Copilot, and Cursor (nightly). What about other tools (Windsurf, Antigravity, etc.)?

---

*This roadmap will be updated as improvements are planned, designed, and implemented.*
