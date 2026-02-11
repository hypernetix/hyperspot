# DECOMPOSITION Rules (CyberFabric)

## Required References (ALWAYS)

ALWAYS open and follow `../../rules.md`

ALWAYS open and follow `docs/spec-templates/DECOMPOSITION.md` as a template
ALWAYS open and follow `docs/spec-templates/examples/todo-app/DECOMPOSITION.md`
ALWAYS open and follow `docs/checklists/DECOMPOSITION.md` as a quality checklist

## CyberFabric SDLC Chain (this kit)

- PRD → ADR → DESIGN → DECOMPOSITION → FEATURE

## CyberFabric Deltas vs Original Cypilot SDLC

- CyberFabric decomposes DESIGN into `FEATURE` entries.
- Each entry MUST link to a corresponding FEATURE artifact and satisfy constraints.json coverage rules.

## Constraints Alignment (REQUIRED)

DECOMPOSITION MUST define at least one `feature` ID (Feature Entry).

Each `feature` ID is expected to be covered by FEATURE artifacts according to constraints.json (coverage: required).

## Generation Checklist

- [ ] Decompose DESIGN components/sequences/data into features with high cohesion and clear boundaries.
- [ ] Ensure 100% coverage: every relevant DESIGN element appears in at least one feature entry.
- [ ] Avoid overlap: design elements should not be duplicated across features without an explicit reason.
- [ ] Assign priorities (`p1`-`p9`) and keep dependencies explicit and acyclic.
- [ ] Ensure each DECOMPOSITION entry links to a corresponding FEATURE artifact path and that FEATURE is registered (if applicable).
- [ ] Make all decomposition entries as features in `features/` as `NNNN-cpt-{system}-feature-{slug}.md` with `## 1. Feature Context` header only which includes reference to feature CPT ID, open `docs/spec-templates/FEATURE.md` for details

## Validation Checklist

- [ ] `python3 {cypilot_path}/skills/cypilot/scripts/cypilot.py validate --artifact <path>`
- [ ] `make cypilot-validate`
- [ ] Review against `docs/checklists/DECOMPOSITION.md`.

## Review Checklist

- [ ] At least one `feature` ID; each with Purpose, Scope, Dependencies, covered IDs
- [ ] 100% DESIGN coverage; no overlap without explicit reason
- [ ] No circular dependencies — valid DAG; consistent granularity
- [ ] No implementation details, requirements defs, or decision debates
- [ ] Run `docs/checklists/DECOMPOSITION.md` for full validation
