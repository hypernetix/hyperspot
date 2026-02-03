---
status: "{proposed | accepted | deprecated | superseded by ADR-NNNN}"
date: {YYYY-MM-DD}
---
<!--
=============================================================================
ARCHITECTURE DECISION RECORD (ADR) — based on MADR format
=============================================================================
PURPOSE: Capture WHY a significant technical decision was made — context,
options considered, trade-offs, and consequences.

SCOPE:
  ✓ Context and problem statement
  ✓ Decision drivers (constraints, quality attributes)
  ✓ Options considered with pros/cons
  ✓ Chosen option with justification
  ✓ Consequences (good and bad)

NOT IN THIS DOCUMENT (see other templates):
  ✗ Requirements → PRD.md
  ✗ Full architecture description → DESIGN.md
  ✗ Implementation details → features/

RULES:
  - ADRs represent actual decision dilemma and decision state
  - DESIGN is the primary artifact ("what"); ADRs annotate DESIGN with rationale ("why")
  - Avoid "everything is a decision"; write ADRs only when the rationale needs to be explained and recorded for traceability
  - Decision history is in git, not in documents
  - Use single ADR per decision

STANDARDS ALIGNMENT:
  - MADR (Markdown Any Decision Records)
  - IEEE 42010 (architecture decisions as first-class elements)
  - ISO/IEC 15288 / 12207 (decision analysis process)
=============================================================================
-->

# {Short title describing problem and chosen solution}

- [ ] `p2` - **ID**: `fdd-{module}-adr-{slug}`

## Context and Problem Statement

{Describe the context and problem statement in 2-3 sentences. You may articulate the problem as a question.}

## Decision Drivers and Traceability

* {Decision driver 1, e.g., a force, facing concern, …}
* {Decision driver 2, e.g., a force, facing concern, …}

- **PRD**: [PRD.md](../PRD.md)
- **DESIGN**: [DESIGN.md](../DESIGN.md)

This decision directly addresses the following requirements from PRD/DESIGN:

* `fdd-{module}-req-{slug}` — {Brief description of how this decision satisfies/constrains this requirement}
* `fdd-{module}-design-{slug}` — {Brief description of design element affected}

## Considered Options

* {Title of option 1}
* {Title of option 2}
* {Title of option 3}

## Decision Outcome

Chosen option: "{title of option 1}", because {justification, e.g., only option which meets k.o. criterion decision driver | resolves force | comes out best}.

### Consequences

* Good, because {positive consequence, e.g., improvement of one or more desired qualities}
* Bad, because {negative consequence, e.g., compromising one or more desired qualities}

### Confirmation

{Describe how the implementation/compliance of the ADR can be confirmed. E.g., design/code review, ArchUnit test, etc.}

## Pros and Cons of the Options

### {Title of option 1}

{Description or pointer to more information}

* Good, because {argument a}
* Good, because {argument b}
* Neutral, because {argument c}
* Bad, because {argument d}

### {Title of option 2}

{Description or pointer to more information}

* Good, because {argument a}
* Bad, because {argument b}

## More Information

{Additional evidence, team agreement, links to related decisions and resources.}
