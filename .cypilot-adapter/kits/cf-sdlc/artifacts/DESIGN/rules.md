# DESIGN Rules (Hyperspot)

## Required References (ALWAYS)

ALWAYS open and follow `../../rules.md`

ALWAYS open and follow `docs/spec-templates/DESIGN.md` as a template

ALWAYS open and follow `docs/checklists/DESIGN.md` as a quality checklist

## Hyperspot Deltas vs Original Cypilot SDLC

- Downstream chain uses DECOMPOSITION → FEATURE.
- Prefer decisions aligned with ModKit patterns and the secure ORM (`SecurityCtx` request-scoped).

## Constraints Alignment (REQUIRED)

DESIGN MUST define at least one ID of each required kind:

- `principle` (Design Principle)
- `constraint` (Design Constraint)
- `component` (Component)
- `seq` (Sequence)

DESIGN SHOULD reference PRD FR/NFR IDs (so PRD IDs satisfy their required coverage to DESIGN).

PRD MUST NOT reference DESIGN IDs (`principle`/`constraint`/`component`/`seq`) as backtick IDs.

ADR IDs MUST be referenced from DESIGN (i.e., DESIGN should include backtick references to ADR IDs where decisions are material).

## Generation Checklist

- [ ] Reference PRD FR/NFR IDs and map them to system-level design (WHAT → HOW at architecture level).
- [ ] Define components/sequences/data that will be decomposed later (keep feature-level detail out).
- [ ] Capture principles/constraints explicitly and link ADRs where decisions are material.
- [ ] Keep alignment with ModKit module structure and secure ORM constraints.

## Validation Checklist

- [ ] `python3 {cypilot_path}/skills/cypilot/scripts/cypilot.py validate --artifact <path>`
- [ ] Review against `docs/checklists/DESIGN.md`.
