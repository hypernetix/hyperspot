---
status: accepted
date: 2026-01-21
decision-makers: Cyber Fabric Architects Committee
---

# Use MADR for Architecture Decision Records

## Context and Problem Statement

HyperSpot is a platform that is supposed to be used by multiple organizations. As the platform evolves, we make architectural decisions that affect all stakeholders. How should we document these decisions to ensure:

- Transparency across all stakeholder organizations
- Clear historical record of why decisions were made
- Structured review and approval process
- Easy discoverability for future maintainers

## Decision Drivers

- Multiple stakeholder organizations need visibility into architectural decisions
- Decisions should be version-controlled alongside code
- Review process must support asynchronous, distributed teams
- Format should be easy to write and maintain
- Need categorization for large number of potential decisions

## Considered Options

- [MADR](https://adr.github.io/madr/) 4.0.0 - Markdown Architectural Decision Records
- [Michael Nygard's template](http://thinkrelevance.com/blog/2011/11/15/documenting-architecture-decisions)
- [Y-Statements](https://www.infoq.com/articles/sustainable-architectural-design-decisions)
- Confluence/Wiki pages
- No formal process

## Decision Outcome

Chosen option: "MADR 4.0.0", because:

- **Version controlled**: ADRs live in the repository alongside code, enabling PRs for review
- **Lightweight**: Markdown format is easy to write and doesn't require special tools
- **Structured**: Provides consistent format while allowing flexibility
- **Active community**: MADR is well-maintained and widely adopted
- **GitHub integration**: PR-based approval workflow fits our multi-stakeholder review needs

### No "proposed" Status

We do not use `proposed` status or commit draft ADRs to the repository. All ADRs are committed with `accepted` status from the start, with discussion happening in the Pull Request.

**Rationale:** AI coding agents index repository contents for context. Committing proposed or draft ADRs would cause agents to treat unfinalized decisions as established architecture, leading to confusion and incorrect implementations.

### Consequences

- Good, because all stakeholders can review decisions via familiar GitHub PR workflow
- Good, because decisions are discoverable in the repository
- Good, because history is preserved in git
- Good, because categories help organize decisions by domain
- Good, because AI agents only see finalized decisions
- Neutral, because requires discipline to maintain and update ADRs
- Bad, because adds overhead to decision-making process (justified by multi-stakeholder nature)

### Confirmation

Compliance can be confirmed by:

- Checking that new architectural decisions have corresponding ADRs
- Verifying that ADRs follow the MADR template format
- Ensuring PRs with ADRs have approval from key contributors before merge

## More Information

- MADR Documentation: https://adr.github.io/madr/
- ADR General Info: https://adr.github.io/
- Local templates: [./templates/](./templates/)
- Process documentation: [./README.md](./README.md)
