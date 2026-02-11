# Artifact Checklists

**Version**: 1.0
**Purpose**: Comprehensive expert-driven quality checklists for common software delivery artifacts

---

## Overview

This directory contains detailed checklists organized by **expertise domain** for common software artifacts. Each checklist is designed to be used by domain experts during artifact review.

---

## Expertise Domains

| Domain | Abbreviation | Focus |
|--------|--------------|-------|
| Architecture | `ARCH` | System design, patterns, modularity, scalability |
| Performance | `PERF` | Efficiency, latency, throughput, resource optimization |
| Security | `SEC` | Authentication, authorization, data protection, vulnerabilities |
| Reliability | `REL` | Fault tolerance, error handling, recovery, resilience |
| Usability | `UX` | User experience, accessibility, clarity, discoverability |
| Maintainability | `MAINT` | Code quality, documentation, testability, technical debt |
| Compliance | `COMPL` | Regulations, standards, audits, certifications |
| Data | `DATA` | Data modeling, integrity, privacy, governance |
| Integration | `INT` | APIs, third-party systems, interoperability |
| Operations | `OPS` | Deployment, monitoring, observability, DevOps |
| Testing | `TEST` | Test coverage, quality assurance, validation strategies |
| Business | `BIZ` | Requirements alignment, value delivery, stakeholder needs |

---

## Artifact Checklists

| Artifact | Checklist File |
|----------|----------------|
| PRD | `PRD.md` |
| DESIGN | `DESIGN.md` |
| ADR | `ADR.md` |
| FEATURE | `FEATURE.md` |
| CODING | `CODING.md` |

---

## Checklist Structure

Each checklist contains:

### MUST HAVE Sections
Requirements that **must be present** in the artifact, organized by expertise domain.

### MUST NOT HAVE Sections
Content that **must not appear** in the artifact, with guidance on where that content should be placed instead.

---

## Severity Dictionary

Use only these levels:

| Level | When to use | Expected action |
|-------|-------------|-----------------|
| **CRITICAL** | Missing/incorrect content makes the artifact unsafe, misleading, non-compliant, or not usable for downstream work. Typically blocks implementation or causes rework. | Must fix before continuing. |
| **HIGH** | Major quality gap that significantly increases risk/cost or makes important decisions ambiguous, but work can still proceed with caution. | Fix before approval/release. |
| **MEDIUM** | Meaningful improvement that increases clarity, testability, or maintainability; not usually blocking. | Fix when feasible; track if deferred. |
| **LOW** | Minor improvement, style/format polish, or optional enhancement. | Optional; address opportunistically. |

Rules:

- **CRITICAL**: Impacts correctness, safety, compliance, or makes requirements/design unverifiable.
- **HIGH**: Creates ambiguity in key flows, responsibilities, interfaces, or acceptance criteria.
- **MEDIUM**: Improves completeness/consistency; reduces future rework.
- **LOW**: Cosmetic or optional.

---

## Agent Prompts

Example prompts for using checklists with an AI agent:

### Full Review

```
Review @docs/my-module/PRD.md against @docs/checklists/PRD.md checklist.
Output findings in table format: Domain | Item | Severity | Finding | Recommendation
```

### Domain-Specific Review

```
Review @docs/my-module/DESIGN.md for Security (SEC) items only.
Use @docs/checklists/DESIGN.md as reference.
```

### Critical Issues Only

```
Scan @docs/my-module/FEATURE.md against @docs/checklists/FEATURE.md.
Report only CRITICAL and HIGH severity issues.
```

### Review with Auto-Fix

```
Review @docs/my-module/ADR.md against @docs/checklists/ADR.md.
For each finding, propose a concrete fix. Apply fixes directly if severity is MEDIUM or lower.
```

### Batch Review

```
Review all artifacts in @docs/my-module/ against corresponding checklists in @docs/checklists/.
Summarize by artifact, then by domain.
```

### Pre-Commit Check

```
I'm about to commit changes to @docs/my-module/PRD.md.
Quick check against @docs/checklists/PRD.md — any CRITICAL issues?
```

### Generate Missing Content

```
Based on @docs/checklists/DESIGN.md, identify missing sections in @docs/my-module/DESIGN.md.
Generate draft content for missing CRITICAL items.
```

---

## PR Review Integration

These checklists are integrated with the Cypilot PR review workflow. When reviewing PRs:

- **PRD PRs**: Use `PRD.md` — covers requirements completeness, testability, traceability, and industry alignment
- **Design PRs**: Use `DESIGN.md` — covers architecture, trade-offs, API contracts, security, and antipatterns
- **ADR PRs**: Use `ADR.md` — covers decision significance, alternatives analysis, and overlap detection
- **Code PRs**: Use `CODING.md` — covers Rust correctness, architecture (ModKit/SDK pattern), security (secure ORM), clippy/dylint compliance, testing, performance, etc.

The checklist is auto-selected by the `/cypilot-pr-review` workflow based on the PR content. Configuration is in `.cypilot/.cypilot-config.json` under the `pr.prompts` key.

Example prompts:

```
cypilot review PR 123
cypilot review PR #59 with CODE checklist
review PR 42
```

---

## Cross-References

- Existing project docs (architecture docs, API specs, runbooks)
- Source code and tests
- Cypilot PR review config: `.cypilot-adapter/pr-review.json`
- PR review workflow: `.cypilot/workflows/pr-review.md`
- PR status workflow: `.cypilot/workflows/pr-status.md`
- PR review prompts: `.cypilot/prompts/pr/`
- PR review script: `.cypilot/skills/scripts/pr.py`
- PR review templates: `.cypilot/templates/pr/`
- PR review documentation: `docs/pr-review/`
