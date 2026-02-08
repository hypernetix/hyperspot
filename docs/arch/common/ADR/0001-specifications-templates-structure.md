---
status: "accepted"
date: 2025-02-08
---

# Use structured spec templates with FDD traceability for project documentation

**ID**: `fdd-common-adr-spec-templates-structure`

## Context and Problem Statement

The project needs a consistent, scalable approach to documenting requirements, architecture, decisions, and feature specifications across all modules and subsystems. Without a standardized structure, documentation becomes fragmented, inconsistent, and difficult to navigate — leading to knowledge loss, redundant debates, and unclear traceability from requirements to implementation.

How should we structure our specification documents to ensure consistency, traceability, and maintainability across the entire project?

## Decision Drivers and Traceability

* **Traceability** — ability to trace requirements through design, decisions, and implementation
* **Consistency** — uniform structure across all modules and subsystems so contributors know where to find and place information
* **Standards alignment** — leverage proven industry standards (IEEE, ISO) rather than inventing ad-hoc formats
* **Low tooling overhead** — documentation should be version-controlled, diffable, and reviewable in standard PR workflows
* **Scalability** — structure must work for a growing number of modules without becoming unwieldy
* **AI-agent compatibility** — structured, predictable formats enable AI assistants to reliably read, generate, and validate specifications

This decision establishes the foundational documentation structure for the project. PRD and DESIGN documents for `docs/arch/common/` will follow once the templates are adopted.

## Considered Options

* Structured spec templates with FDD traceability (Markdown + FDD IDs)
* Lightweight wiki-style documentation (flat Markdown, no templates)
* Formal requirements management tooling (DOORS, Jama, Sphinx)
* Code-first documentation only (Rustdoc / doc-comments, minimal high-level specs)

## Decision Outcome

Chosen option: "Structured spec templates with FDD traceability", because it provides the best balance of consistency, traceability, and low overhead while aligning with industry standards and remaining fully version-controlled.

### Consequences

* Good, because all modules follow the same document structure — reduces onboarding friction and cognitive load
* Good, because FDD IDs enable cross-document traceability from requirements → design → decisions → features → code
* Good, because Markdown-only format integrates naturally with Git workflows (diff, PR review, blame)
* Good, because standards alignment (IEEE 830, IEEE 1016, IEEE 42010, MADR) provides proven structure rather than ad-hoc invention
* Good, because structured templates are predictable for AI agents to consume and generate
* Bad, because template overhead may feel heavy for very small or obvious decisions/features
* Bad, because FDD ID discipline requires diligence — stale or missing IDs degrade traceability value

### Confirmation

* Code/documentation review: PRs introducing new modules must include spec documents following the template structure
* `fdd validate` (when FDD tooling is connected) will verify cross-document ID consistency and detect broken references
* Periodic manual audit of `docs/arch/` and module-level specs to confirm adherence

## Pros and Cons of the Options

### Structured spec templates with FDD traceability

Layered Markdown templates (PRD, UPSTREAM_REQS, DESIGN, ADR, FEATURE) with FDD ID convention for cross-document traceability. Documents placed inside module/subsystem folders. Standards-aligned with IEEE 830, IEEE 1016, IEEE 42010, ISO/IEC 15288/12207, and MADR.

* Good, because provides clear separation of concerns: requirements (PRD), design (DESIGN), rationale (ADR), implementation detail (FEATURE)
* Good, because FDD IDs create machine-verifiable traceability links across all artifacts
* Good, because module-level specs document only deviations from global standards — avoids duplication
* Good, because UPSTREAM_REQS enables API-first and consumer-driven design between modules
* Good, because Markdown is universally supported, diffable, and requires no special tooling
* Neutral, because FDD tooling integration is optional — templates work standalone but full validation requires `fdd validate`
* Bad, because initial template adoption requires learning the structure and conventions
* Bad, because maintaining FDD IDs adds a small but ongoing discipline cost

### Lightweight wiki-style documentation

Flat Markdown files or wiki pages without enforced templates. Each team/module documents freely.

* Good, because minimal process overhead — just write what you need
* Good, because low barrier to entry for contributors
* Bad, because no enforced consistency — documentation quality varies across modules
* Bad, because no built-in traceability mechanism between requirements, design, and implementation
* Bad, because scales poorly — becomes disorganized as the number of modules grows
* Bad, because difficult for AI agents to parse reliably due to unpredictable structure

### Formal requirements management tooling

Dedicated tools such as IBM DOORS, Jama Connect, or Sphinx-based documentation systems with requirements plugins.

* Good, because purpose-built for requirements management with native traceability
* Good, because provides rich querying, reporting, and impact analysis capabilities
* Bad, because introduces vendor lock-in and additional tooling costs
* Bad, because documentation lives outside the code repository — breaks single-source-of-truth principle
* Bad, because higher friction for developers who must switch between IDE and external tools
* Bad, because not diffable or reviewable in standard Git PR workflows

### Code-first documentation only

Rely primarily on Rustdoc / doc-comments for API documentation, with minimal high-level specification documents.

* Good, because documentation stays closest to the code and is updated alongside it
* Good, because zero overhead for maintaining separate spec files
* Bad, because loses the "why" — code comments rarely capture decision rationale or rejected alternatives
* Bad, because no place for high-level requirements, product vision, or cross-cutting architectural decisions
* Bad, because insufficient for complex domains where behavior must be specified before implementation
* Bad, because no traceability from business requirements to implementation

## More Information

* Template definitions: [docs/spec-templates/](../../spec-templates/)
* FDD framework: [Flow-Driven Development](https://github.com/cyberfabric/FDD)
* Document placement convention: specs live inside `docs/arch/common/`, `docs/arch/{subsystem}/`, or `{module}/` directories
* Naming convention: ADR and Feature files use `NNNN-{fdd-id}.md` prefix format
