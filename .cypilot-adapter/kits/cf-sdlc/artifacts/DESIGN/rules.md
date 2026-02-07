# DESIGN Rules (Hyperspot)

## Required References (ALWAYS)

ALWAYS open and follow `../../rules.md`

ALWAYS open and follow `docs/spec-templates/DESIGN.md` as a template

ALWAYS open and follow `docs/checklists/DESIGN.md` as a quality checklist

## Hyperspot Deltas vs Original Cypilot SDLC

- Downstream chain uses DECOMPOSITION → FEATURE.
- Prefer decisions aligned with ModKit patterns and the secure ORM (`SecurityCtx` request-scoped).

## Generation Checklist

- [ ] Reference PRD FR/NFR IDs and map them to system-level design (WHAT → HOW at architecture level).
- [ ] Define components/sequences/data that will be decomposed later (keep feature-level detail out).
- [ ] Capture principles/constraints explicitly and link ADRs where decisions are material.
- [ ] Keep alignment with ModKit module structure and secure ORM constraints.

## Validation Checklist

- [ ] `python3 {cypilot_path}/skills/cypilot/scripts/cypilot.py validate --artifact <path>`
- [ ] Review against `docs/checklists/DESIGN.md`.
