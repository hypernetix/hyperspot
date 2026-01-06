# Module Development Workflow

This document describes the module development workflow.

It combines AI-assisted design with [OpenSpec-driven](https://openspec.dev/) implementation tracking.

> **Document Audience:** Human developers. AI assistance generates documentation and code; developers perform workflow steps, decision-making, and validation.

## Index

- [Quick Links](#quick-links)
- [Workflow Overview](#workflow-overview)
- [Quick Reference](#quick-reference)
- [AI Assistant](#ai-assistant)
- [Step 1: Design \& Planning](#step-1-design--planning)
- [Step 2: Implementation (OpenSpec)](#step-2-implementation-openspec)
- [Step 3: Verification \& Completion](#step-3-verification--completion)
- [Adding New Feature](#adding-new-feature)
- [Best Practices](#best-practices)
- [References](#references)

## Quick Links

- [Module Development Reference](./REFERENCE.md) — Terminology, document templates, directory structure
- [FAQ](./FAQ.md) — Common questions and rationale
- [Prompt Templates](./prompts/) — AI prompts for each step
- [ROADMAP](./ROADMAP.md) — Future improvements of the workflow

---

## Workflow Overview

### New Module Development

```
┌──────────────────────────────────────────────────────────────────┐
│ Step 1: Design & Planning                                        |
│  └─ 1.1: Create DESIGN.md (module architecture)                  │
│  └─ 1.2: Create Features (FEATURE.md files with requirements)    │
│  └─ 1.3: Create IMPLEMENTATION_PLAN.md (phases → features → reqs)│
│  └─ 1.4: Validate design documents                               │
│  └─ 1.5: Create PR for design review                             │
└──────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ ITERATE: For each requirement (or batch) to implement           │
│                                                                 │
│ Step 2: Implementation (OpenSpec)                               │
│  └─ 2.1: Create OpenSpec change proposal                        │
│  └─ 2.2: Review and refine proposal                             │
│  └─ 2.3: Verify specs vs. requirements                          │
│  └─ 2.4: Implement code + unit tests                            │
│  └─ 2.5: Generate E2E tests (if applicable)                     │
│                              ↓                                  │
│ Step 3: Verification & Completion                               │
│  └─ 3.1: Verify code vs. specs & requirements                   │
│  └─ 3.2: Verify code vs. design                                 │
│       → Issues found? Fix and return to Step 2.4                │
│  └─ 3.3: Archive the OpenSpec change                            │
│  └─ 3.4: Create PR for feature                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Adding New Feature

```
┌─────────────────────────────────────────────────────────────────┐
│ Create FEATURE.md for the new feature                           │
│  └─ Use create_feature.md prompt                                │
│  └─ Update DESIGN.md if architecture changes                    │
│  └─ Update IMPLEMENTATION_PLAN.md with new feature              │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ Steps 2-3: Implementation + Verification (same as above)        │
└─────────────────────────────────────────────────────────────────┘
```

---

## Quick Reference

| Term | Format | Example |
|------|---------|---------|
| Implementation Phase | `#{module}/P{N}` | `#oagw/P1` |
| Feature | `{feature-name}/` | `request-forward/` |
| Global Requirement | `#{name}` | `#tenant-isolation` |
| Module Requirement | `#{module}/{name}` | `#oagw/request-forward` |

For detailed formats and templates, see [Module Development Reference](./REFERENCE.md).

---

## AI Assistant

### LLM Model Tiers

It's recommended to use different models for different steps of the workflow for the economic efficiency. Higher tier models produce better results but cost more. Use lower tiers where the task complexity allows to optimize costs.

| Tier | Use Case | Examples |
|------|----------|----------|
| **High Reasoning** | Complex design, architecture decisions, verification analysis | Claude Opus 4.5, GPT-5.2, Gemini 3 Pro (High) |
| **Mid Reasoning** | Code generation, implementation, standard patterns | Claude Sonnet 4.5, Gemini 3 Pro (Low) |
| **Low Reasoning** | Simple refactoring, formatting, small bugfixes | Claude Haiku 4.5, Gemini 3 Flash |

### Prompt Templates

All design steps use prompt templates from [`prompts/`](./prompts/).

1. Open the prompt template file
2. Replace `{module}` with lowercase module name (e.g., `oagw`, `types_registry`)
3. Choose appropriate model tier for the step
4. Submit to AI assistant with module context
5. Review and refine the generated output

---

## Step 1: Design & Planning

> **Recommended Model Tier:** High Reasoning

### Step 1.1: Create DESIGN.md

**Prompt:** [`create_design.md`](./prompts/create_design.md)

**Prerequisites:**
- Review [`guidelines/NEW_MODULE.md`](../../guidelines/NEW_MODULE.md) for module development standards
- Review [`MODKIT_UNIFIED_SYSTEM.md`](../MODKIT_UNIFIED_SYSTEM.md) for ModKit framework patterns
- Review global requirements in [`REQUIREMENTS.md`](../REQUIREMENTS.md)
- Clarify module purpose, scope, and key use cases

**Output:** `DESIGN.md` with architecture, components, phases, and integration points

**Tips:**
- Focus on **what** and **why**, not implementation details
- Define implementation phases (`#{module}/P1`, `#{module}/P2`...)
- Mark open questions for design review

---

### Step 1.2: Create Features

**Prompt:** [`create_feature.md`](./prompts/create_feature.md)

For each planned feature, create a `features/{feature-name}/FEATURE.md` file containing:
- Feature description and scope
- Requirements (using RFC 2119 language: SHALL/SHOULD/MAY)
- Phase reference and implementation approach

**Output:** `features/{feature-name}/FEATURE.md` for each feature

**Notes:**
- Each feature usually maps 1:1 to an OpenSpec specification
- Requirements use flat IDs: `#{module}/{requirement-name}`
- Reference global requirements — don't duplicate them
- Repeat this step for each feature in the module

---

### Step 1.3: Create IMPLEMENTATION_PLAN.md

**Prompt:** [`create_implementation_plan.md`](./prompts/create_implementation_plan.md)

**Prerequisites:**
- DESIGN.md exists (from Step 1.1)
- FEATURE.md files exist (from Step 1.2)

**Output:** `IMPLEMENTATION_PLAN.md` with nested checkboxes: phases → features → requirements

**Tips:**
- Organize by phases from DESIGN.md
- Each feature checkbox has requirement sub-items
- This becomes your progress tracker

---

### Step 1.4: Validate Design Documents

**Prompt:** [`validate_design_docs.md`](./prompts/validate_design_docs.md)

**What Gets Validated:**

| Category | Checks |
|----------|--------|
| **Format** | DESIGN.md sections, FEATURE.md structure, RFC 2119 language |
| **Consistency** | Phases match across docs, requirement phases reference valid phases |
| **Cross-References** | All requirements exist, all features have requirements |
| **Completeness** | All phases have features, all requirements have rationale |

**Pass Criteria:** Zero errors required to proceed. Address warnings where appropriate.

---

### Step 1.5: Create PR for Design Review

1. **Verify** Step 1.4 completed with zero errors
2. **Commit docs and create PR** with title: `docs({module}): add design documents`
3. **PR description** should include: module purpose, features summary, phases overview, open questions

**Review Checklist:**
- [ ] Architecture aligns with ModKit patterns
- [ ] Requirements are clear, testable, use RFC 2119 language
- [ ] Implementation plan covers all requirements
- [ ] Phases are independently implementable
- [ ] No conflicts with existing modules

---

## Step 2: Implementation (OpenSpec)

> **Recommended Model Tier:** Mid Reasoning

For each feature in IMPLEMENTATION_PLAN.md, use OpenSpec to implement incrementally.

### Prerequisites: Module OpenSpec Setup

Before creating OpenSpec proposals for a module, ensure the module has OpenSpec initialized:

1. **Check if module has openspec directory:**
   ```bash
   ls modules/{module}/openspec/
   ```

2. **If not initialized, run from module directory:**
   ```bash
   cd modules/{module}
   openspec init
   ```

**Notes:**
1. All `openspec` commands for a module must be run from that module's directory:
```bash
cd modules/{module}
openspec list
openspec validate --strict
```
2. Only root level OpenSpec slash commands should be used. They extended with a special intructions to run commands in the module directory. 

---

### Step 2.1: Create OpenSpec Change Proposal

Create a change proposal for the next unchecked feature in IMPLEMENTATION_PLAN.md.

**Command:** `/openspec:proposal` (or equivalent AI command)

**Input:** Feature name and scope from IMPLEMENTATION_PLAN.md, reference to the module's design documents. Module name must be specified explicitly.

**Output:**
-   `modules/{module}/openspec/changes/{change-name}/proposal.md`
-   `modules/{module}/openspec/changes/{change-name}/tasks.md`
-   `modules/{module}/openspec/changes/{change-name}/design.md` (optional)
-   `modules/{module}/openspec/changes/{change-name}/specs/{capability}/spec.md`

See [OpenSpec documentation](https://github.com/Fission-AI/OpenSpec/blob/main/README.md) for details.

---

### Step 2.2: Review and Refine Proposal

1. **Review** the generated proposal for alignment with the module's design documents
2. **Refine** specs — ensure scenarios cover success, error, and edge cases
3. **Validate** tasks — ensure they're granular and actionable (< 1 hour each)
4. **Run** `openspec validate --strict` before proceeding

See [OpenSpec documentation](https://github.com/Fission-AI/OpenSpec/blob/main/README.md) for validation.

---

### Step 2.3: Verify Specs vs. Requirements

**Prompt:** [`verify_specs_vs_requirements.md`](./prompts/verify_specs_vs_requirements.md)

Before implementation, validate that the proposed specs align with requirements:

| Check | Description |
|-------|-------------|
| **Coverage** | All referenced requirements exist in REQUIREMENTS.md |
| **Completeness** | Proposed scenarios cover the feature scope |
| **Consistency** | Scenarios use correct requirement IDs |

**Pass Criteria:** Zero errors required to proceed to implementation.

---

### Step 2.4: Implement the Feature

**Command:** `/openspec:apply` (or equivalent AI command)

**Input:** Approved OpenSpec change proposal. Module name must be specified explicitly.

**Output:** Code changes + unit tests in module source

---

### Step 2.5: Generate E2E Tests (if applicable)

Generate E2E tests when the feature:
- Exposes REST endpoints
- Has complex multi-step workflows
- Integrates with external systems

---

## Step 3: Verification & Completion

> **Recommended Model Tier:** High Reasoning

Before creating a PR, verify that implementation matches documentation. Issues found should be fixed before proceeding.

**Output:** Reports in `modules/{module}/docs/verification/{change-name}/`

---

### Step 3.1: Verify Code vs. Specs & Requirements

**Prompt:** [`verify_code_vs_specs_and_requirements.md`](./prompts/verify_code_vs_specs_and_requirements.md)

Verify that implementation matches OpenSpec scenarios and requirements:

| Check | Description |
|-------|-------------|
| **Scenario Implementation** | All scenarios have corresponding code |
| **Requirements Coverage** | All completed requirements have working code |
| **E2E Test Coverage** | Scenarios are covered by E2E tests |
| **Security Requirements** | `#tenant-isolation`, `#rbac` are enforced in code |

**Output:** `modules/{module}/docs/verification/{change-name}/code_vs_specs_and_requirements.md`

**Issues Found?** Fix issues and return to [Step 2.4](#step-24-implement-the-feature), then re-verify.

---

### Step 3.2: Verify Code vs. Design

**Prompt:** [`verify_code_vs_design.md`](./prompts/verify_code_vs_design.md)

Verify that code structure follows DESIGN.md architecture:

| Check | Description |
|-------|-------------|
| **Layer Structure** | Code matches DESIGN.md layers (contract, api, domain, infra) |
| **Components** | All described components exist in code |
| **Integration Points** | Dependencies and routes match design |

**Output:** `modules/{module}/docs/verification/{change-name}/code_vs_design.md`

**Issues Found?** Fix issues and return to [Step 2.4](#step-24-implement-the-feature), then re-verify.

---

### Step 3.3: Archive the OpenSpec Change

After verification passes, archive the completed change:

**Command:** `/openspec:archive` (or equivalent AI command)

This:
- Moves change to `modules/{module}/openspec/changes/archive/`
- Updates `modules/{module}/openspec/specs/` with verified scenarios
- Check off the feature in IMPLEMENTATION_PLAN.md

See [OpenSpec documentation](https://github.com/Fission-AI/OpenSpec/blob/main/README.md) for archiving.

---

### Step 3.4: Create PR for Feature

After verification passes and change is archived, create a PR that includes code, specs, and verification reports.

1. **Commit** code changes, updated specs, and verification reports
2. **Create PR** with title: `feat({module}): {feature-name}`
3. **PR description** should include:
   - Feature summary and requirement references
   - Scenarios implemented
   - Link to verification reports

**PR includes:**
- `modules/{module}/src/` changes
- `modules/{module}/openspec/specs/` updates
- `modules/{module}/docs/verification/{change-name}/*.md` reports

---

## Adding New Feature

To add a new feature to an existing module:

### Create FEATURE.md

1. **Create** `features/{feature-name}/FEATURE.md` using [`create_feature.md`](./prompts/create_feature.md) prompt
2. **Define requirements** using RFC 2119 language (SHALL/SHOULD/MAY)
3. **Reference** global requirements — don't duplicate them

### Update Design Documents

1. **Update DESIGN.md** if the feature changes architecture or adds new components
2. **Update IMPLEMENTATION_PLAN.md** to add the new feature and its requirements
3. **Validate** using [Step 1.4](#step-14-validate-design-documents)

### Implementation & Verification

Follow [Step 2: Implementation](#step-2-implementation-openspec) and [Step 3: Verification & Completion](#step-3-verification--completion).

After PR is merged, update `modules/{module}/docs/CHANGELOG.md` following [Keep A Changelog](https://keepachangelog.com/) format:

```markdown
## [Unreleased]

### Added
- Pagination support for entity listing — Implements #typereg/pagination

### Fixed
- Race condition in concurrent updates — Fixes #typereg/concurrent-updates
```

---

## Best Practices

1. **Complete design first** — Finish DESIGN.md, REQUIREMENTS.md, IMPLEMENTATION_PLAN.md before starting OpenSpec changes
2. **One feature per change** — Keep OpenSpec changes focused and reviewable
3. **Validate early** — Run `openspec validate --strict` before implementing
4. **Update status continuously** — Check off tasks as you complete them
5. **Reference requirements** — Every scenario should reference the requirement it verifies
6. **Keep IMPLEMENTATION_PLAN.md current** — It's your source of truth for what to work on next

---

## References

- [Module Development Reference](./REFERENCE.md) — ID formats, document templates, directory structure
- [Module Development Roadmap](./ROADMAP.md) — Future improvements of the workflow
- [FAQ](./FAQ.md) — Common questions and rationale
- [OpenSpec Documentation](https://github.com/Fission-AI/OpenSpec/blob/main/README.md)
- [Keep A Changelog](https://keepachangelog.com/en/1.0.0/)
- [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119) — Requirement keywords (MUST/SHALL/SHOULD/MAY)
- [guidelines/NEW_MODULE.md](../../guidelines/NEW_MODULE.md) — Module development standards
- [MODKIT_UNIFIED_SYSTEM.md](../MODKIT_UNIFIED_SYSTEM.md) — ModKit framework patterns
