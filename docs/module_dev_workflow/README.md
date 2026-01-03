# Module Development Workflow

This document describes the module development workflow:
- **New modules** — design and implement from scratch
- **Existing modules** — add new features post-1.0 release

It combines AI-assisted design with [OpenSpec-driven](https://openspec.dev/) implementation tracking.

> **Intended Audience:** Human developers. AI assistance generates documentation and code; developers perform workflow steps, decision-making, and validation.

## Index

- [Workflow Overview](#workflow-overview)
- [Quick Reference](#quick-reference)
- [Using Prompt Templates](#using-prompt-templates)
- [Step 1: Design & Planning](#step-1-design--planning)
  - [1.1: Create DESIGN.md and REQUIREMENTS.md](#step-11-create-designmd-and-requirementsmd-iterative)
  - [1.2: Create IMPLEMENTATION_PLAN.md](#step-12-create-implementation_planmd)
  - [1.3: Validate Design Documents](#step-13-validate-design-documents)
  - [1.4: Create PR for Design Review](#step-14-create-pr-for-design-review)
- [Step 2: Implementation (OpenSpec)](#step-2-implementation-openspec)
  - [2.1: Create OpenSpec Change Proposal](#step-21-create-openspec-change-proposal)
  - [2.2: Review and Refine Proposal](#step-22-review-and-refine-proposal)
  - [2.3: Implement the Feature](#step-23-implement-the-feature)
  - [2.4: Generate E2E Tests](#step-24-write-e2e-tests-if-applicable)
  - [2.5: Archive the OpenSpec Change](#step-25-archive-the-openspec-change)
- [Adding Features to Existing Modules](#adding-features-to-existing-modules)
- [Best Practices](#best-practices)
- [References](#references)

## Quick Links

- [Module Development Reference](./REFERENCE.md) — ID formats, document templates, directory structure
- [FAQ](./FAQ.md) — Common questions and rationale
- [Prompt Templates](./prompts/) — AI prompts for each step
- [ROADMAP](./ROADMAP.md) — Future improvements of the workflow

---

## Workflow Overview

### New Module Development

```
┌─────────────────────────────────────────────────────────────────┐
│ Step 1: Design & Planning                                       │
│  └─ 1.1: Create DESIGN.md and REQUIREMENTS.md (iterative)       │
│  └─ 1.2: Create IMPLEMENTATION_PLAN.md                          │
│  └─ 1.3: Validate design documents                              │
│  └─ 1.4: Create PR for design review                            │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ Step 2: Implementation (OpenSpec)                               │
│  └─ For each feature in IMPLEMENTATION_PLAN:                    │
│      1. Create OpenSpec change proposal                         │
│      2. Review and refine proposal, specs and tasks             │
│      3. Implement code + unit tests                             │
│      4. Generate E2E tests (if applicable)                         │
│      5. Archive change → specs updated                          │
└─────────────────────────────────────────────────────────────────┘
```

### Adding Features to Existing Module

```
┌─────────────────────────────────────────────────────────────────┐
│ Step 1: Update Design Documents                                 │
│  └─ Review existing docs, update as needed, validate            │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ Step 2: Implementation (OpenSpec) — same as new module          │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ Step 3: Update CHANGELOG.md                                     │
└─────────────────────────────────────────────────────────────────┘
```

---

## Quick Reference

| Term | Format | Example |
|------|--------|---------|
| Implementation Phase | `{MODULE}-P{N}` | `OAGW-P1` |
| Module Requirement | `{MODULE}-REQ{N}` | `OAGW-REQ01` |
| Global Requirement | `REQ{N}` | `REQ1` |

For detailed formats and templates, see [Module Development Reference](./REFERENCE.md).

---

## Using Prompt Templates

All design steps use prompt templates from [`prompts/`](./prompts/):

1. Open the prompt template file
2. Replace `{module_name}` (snake_case) and `{MODULE}` (UPPERCASE prefix)
3. Submit to AI assistant with module context
4. Review and refine the generated output

| Step | Prompt File | Output |
|------|-------------|--------|
| 1.1 | [`create_design_and_requirements.md`](./prompts/create_design_and_requirements.md) | `DESIGN.md` + `REQUIREMENTS.md` |
| 1.2 | [`create_implementation_plan.md`](./prompts/create_implementation_plan.md) | `IMPLEMENTATION_PLAN.md` |
| 1.3 | [`validate_design_docs.md`](./prompts/validate_design_docs.md) | Validation report |

---

## Step 1: Design & Planning

### Step 1.1: Create DESIGN.md and REQUIREMENTS.md (Iterative)

**Prompt:** [`create_design_and_requirements.md`](./prompts/create_design_and_requirements.md)

**Prerequisites:**
- Review [`guidelines/NEW_MODULE.md`](../../guidelines/NEW_MODULE.md) for module development standards
- Review [`MODKIT_UNIFIED_SYSTEM.md`](../MODKIT_UNIFIED_SYSTEM.md) for ModKit framework patterns
- Review global requirements in [`REQUIREMENTS.md`](../REQUIREMENTS.md)
- Clarify module purpose, scope, and key use cases

**Output:** Both `DESIGN.md` and `REQUIREMENTS.md` with cross-references

**Iterative Process:**

Design and requirements evolve together through iteration:

```
┌──────────────────────────────────────────────────────────────┐
│  Draft Design → Extract Requirements → Refine Design → ...  │
│       ↑                                        │             │
│       └────────────────────────────────────────┘             │
│                    Iterate until consistent                  │
└──────────────────────────────────────────────────────────────┘
```

1. **Draft Initial Design** — Architecture, components, data flow, phases
2. **Extract Requirements** — What the system SHALL/SHOULD/MAY do
3. **Refine Design** — Requirements often reveal missing components or unclear flows
4. **Cross-Reference** — Add requirement IDs to design, verify alignment
5. **Iterate** — Repeat until both documents are internally consistent

**Tips:**
- Focus on **what** and **why**, not implementation details
- Requirements must be atomic and testable
- Reference global requirements (REQ1, REQ2...) — don't duplicate
- Use RFC 2119 language: SHALL (mandatory), SHOULD (recommended), MAY (optional)
- Mark open questions for design review

---

### Step 1.2: Create IMPLEMENTATION_PLAN.md

**Prompt:** [`create_implementation_plan.md`](./prompts/create_implementation_plan.md)

**Prerequisites:**
- DESIGN.md and REQUIREMENTS.md exist with cross-references (from Step 1.1)

**Tips:**
- Each checkbox = 1 feature (maps to 1 OpenSpec change)
- Include requirement references: `({MODULE}-REQ{X}, REQ{Y})`
- Add scope hints: `Scope: contract, domain service, REST endpoint`
- Organize by phases if defined in DESIGN.md

---

### Step 1.3: Validate Design Documents

**Prompt:** [`validate_design_docs.md`](./prompts/validate_design_docs.md)

**What Gets Validated:**

| Category | Checks |
|----------|--------|
| **Format** | Required sections, ID formats, checkbox syntax, RFC 2119 language |
| **Consistency** | Phases match across docs, requirement phases reference valid phases |
| **Cross-References** | All referenced requirements exist, all features reference requirements |
| **Completeness** | All phases have features, scope hints present, rationale present |

**Pass Criteria:** Zero errors required to proceed. Address warnings where appropriate.

---

### Step 1.4: Create PR for Design Review

1. **Verify** Step 1.3 completed with zero errors
2. **Commit docs and create PR** with title: `docs({module}): add design documents`
3. **PR description** should include: module purpose, key requirements summary, phases overview, open questions

**Review Checklist:**
- [ ] Architecture aligns with ModKit patterns
- [ ] Requirements are clear, testable, use RFC 2119 language
- [ ] Implementation plan covers all requirements
- [ ] Phases are independently implementable
- [ ] No conflicts with existing modules

---

## Step 2: Implementation (OpenSpec)

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

1. **Review** the generated proposal for alignment with requirements and design (if generated) for alignment with the module's design documents.
2. **Refine** specs — ensure scenarios cover success, error, and edge cases
3. **Validate** tasks — ensure they're granular and actionable (< 1 hour each)
4. **Run** `openspec validate --strict` before implementing

See [OpenSpec documentation](https://github.com/Fission-AI/OpenSpec/blob/main/README.md) for validation.

---

### Step 2.3: Implement the Feature

**Command:** `/openspec:apply` (or equivalent AI command)

**Input:** Approved OpenSpec change proposal. Module name must be specified explicitly.

**Output:** Code changes + unit tests in module source

---

### Step 2.4: Generate E2E Tests (if applicable)

Generate E2E tests when the feature:
- Exposes REST endpoints
- Has complex multi-step workflows
- Integrates with external systems

---

### Step 2.5: Archive the OpenSpec Change

After implementation is complete and tests pass:

**Command:** `/openspec:archive` (or equivalent AI command)

This:
- Moves change to `modules/{module}/openspec/changes/archive/`
- Updates `modules/{module}/openspec/specs/` with verified scenarios
- Check off the feature in IMPLEMENTATION_PLAN.md

See [OpenSpec documentation](https://github.com/Fission-AI/OpenSpec/blob/main/README.md) for archiving.

---

## Adding Features to Existing Modules

For modules with existing design documentation:

### Step 1: Update Design Documents

1. **Review** existing DESIGN.md, REQUIREMENTS.md, IMPLEMENTATION_PLAN.md
2. **Add requirement** to REQUIREMENTS.md if introducing new capability
3. **Update DESIGN.md** if architecture, components, or data flow changes
4. **Add feature** to IMPLEMENTATION_PLAN.md with requirement references
5. **Validate** using [Step 1.3](#step-13-validate-design-documents)

### Step 2: Implementation

Follow [Step 2: Implementation (OpenSpec)](#step-2-implementation-openspec) above.

### Step 3: Update CHANGELOG.md

Add entry to `modules/{module}/docs/CHANGELOG.md` following [Keep A Changelog](https://keepachangelog.com/) format.

Reference requirement IDs in entries:
```markdown
## [Unreleased]

### Added
- Pagination support for entity listing - Implements TYPEREG-REQ9, references REQ1

### Fixed
- Race condition in concurrent updates - Fixes TYPEREG-REQ11
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
