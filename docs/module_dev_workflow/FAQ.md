# Module Development Workflow FAQ

This document answers common questions about the module development workflow, providing rationale and decision-making guidance.

## Index

- [General Questions](#general-questions)
  - [Why both Design Docs AND OpenSpec?](#why-both-design-docs-and-openspec)
  - [Is OpenSpec a strict requirement?](#is-openspec-a-strict-requirement-for-this-workflow)
- [Design Phase](#design-phase)
  - [Why are DESIGN.md and REQUIREMENTS.md created together?](#why-are-designmd-and-requirementsmd-created-together)
  - [Why keep them as separate files?](#why-keep-them-as-separate-files)
  - [Why use implementation phases?](#why-use-implementation-phases-in-designmd)
  - [Reference vs. duplicate global requirements?](#should-i-reference-global-requirements-req1-req2-or-duplicate-them-in-module-requirementsmd)
  - [When to create new vs. reference existing requirements?](#when-should-i-create-a-new-requirement-vs-referencing-an-existing-one)
- [Workflow Questions](#workflow-questions)
  - [What if my design is wrong during implementation?](#what-if-i-discover-during-implementation-that-my-design-is-wrong)
- [References](#references)

---

## General Questions

### Why both Design Docs AND OpenSpec?

**Design docs are not suitable for OpenSpec format:**
- **Design is exploratory** — You need to iterate on architecture, components, and data flow before committing to implementation
- **High-level thinking** — Design docs capture the "why" and "how" at a conceptual level, not detailed scenarios
- **Free-form flexibility** — Markdown allows diagrams, prose explanations, and open questions that don't fit spec format
- **AI collaboration** — Free-form documents work better for AI-assisted design discussions and brainstorming

**OpenSpec is for implementation tracking:**
- **Concrete behavior** — Specs document exact inputs, outputs, and edge cases that are verified by code
- **Feature-by-feature** — Each change focuses on a single, implementable unit of work
- **Living documentation** — Specs evolve with the code and are kept in sync through archiving
- **Validation** — Structured format enables automated validation and consistency checks

**Clear workflow phases:**
1. **Design Phase** → Use DESIGN.md, REQUIREMENTS.md (free-form, exploratory)
2. **Implementation Phase** → Use OpenSpec changes (structured, verified)
3. **Archive** → Specs become the verified documentation of what actually works

**TL;DR:** Design docs answer "what should we build and why?"; OpenSpec specs answer "what did we build and how does it behave?"

### Is OpenSpec a strict requirement for this workflow?

**No, OpenSpec is a tool choice, not a requirement.**

**Current state:**
- OpenSpec provides a practical way to do feature-by-feature spec-driven development
- It has CLI tooling for validation, listing changes, and archiving
- It works well enough for our current needs

**Future flexibility:**
- We may switch to another spec-driven development (SDD) tool as we gain more experience
- We may implement our own tool tailored to Hyperspot's specific needs
- The workflow principles (design → implementation → verification) remain the same regardless of tooling

**What matters:**
- **Feature-by-feature implementation** — Breaking work into small, reviewable chunks
- **Spec-driven development** — Documenting expected behavior before/during implementation
- **Verification** — Ensuring specs match actual code behavior

**TL;DR:** OpenSpec is our current implementation of spec-driven development, but the workflow is tool-agnostic. We can replace it when we find or build something better.

---

## Design Phase

### Why are DESIGN.md and REQUIREMENTS.md created together?

**They inform each other:**
- Defining requirements often reveals missing design elements
- Design decisions create new requirements
- Phase boundaries depend on requirement dependencies
- Trying to separate them linearly leads to rework

**Iterative process:**
```
Draft Design → Extract Requirements → Refine Design → Cross-Reference → Iterate
```

**Benefits of iteration:**
- Catches inconsistencies early (before implementation)
- Requirements and design stay aligned throughout
- Reduces back-and-forth during implementation
- Both documents are ready for review together

### Why keep them as separate files?

**Separation of concerns:**
- **DESIGN.md** focuses on architecture, components, and technical decisions (the "how")
- **REQUIREMENTS.md** focuses on capabilities and acceptance criteria (the "what")

**Benefits:**
- Requirements can be referenced independently in specs and tests
- Architecture can evolve without rewriting requirements
- Clear traceability from requirement → design → implementation
- Different stakeholders can focus on relevant documents

**TL;DR:** Created together (iteratively), stored separately (for different purposes).

### Why use implementation phases in DESIGN.md?

**Incremental delivery:**
- Phases break large modules into independently shippable units
- Each phase delivers working, testable functionality
- Earlier phases can be used while later phases are in development

**Risk management:**
- Validates core architecture early
- Allows pivoting if design assumptions prove incorrect
- Reduces scope creep by clearly defining phase boundaries

### Should I reference global requirements (REQ1, REQ2) or duplicate them in module REQUIREMENTS.md?

**Always reference, never duplicate.**

**Why:**
- Single source of truth for cross-cutting concerns
- Changes to global requirements automatically apply to all modules
- Reduces maintenance burden and inconsistencies

### When should I create a new requirement vs. referencing an existing one?

**Create a new module requirement when:**
- The capability is specific to this module
- It needs module-specific acceptance criteria
- It will be referenced by multiple scenarios within the module

**Reference a global requirement when:**
- The capability applies to all modules (e.g., error handling, logging, monitoring)
- It's defined in `docs/REQUIREMENTS.md`

**Reference another module's requirement when:**
- You depend on functionality from that module
- Example: `"SHALL integrate with Type Registry (references TYPEREG-REQ5)"`

---

## Workflow Questions

### What if I discover during implementation that my design is wrong?

**Depends on the severity:**

**Minor refinement (e.g., renaming a function, adjusting internal structure):**
- Update the OpenSpec proposal or spec delta
- No need to update DESIGN.md unless architecture changed

**Significant change (e.g., different data flow, additional component):**
1. Pause implementation
2. Update DESIGN.md and REQUIREMENTS.md
3. Validate docs
4. Get design review if needed
5. Resume implementation with updated OpenSpec change

---

## References

- [Module Development Reference](./REFERENCE.md) — ID formats, templates, directory structure
- [OpenSpec Documentation](https://github.com/Fission-AI/OpenSpec/blob/main/README.md)
- [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119) — Requirement keywords
