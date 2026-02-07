# FEATURE Rules (Hyperspot)

## Required References (ALWAYS)

ALWAYS open and follow `../../rules.md`

ALWAYS open and follow `docs/spec-templates/FEATURE.md` as a template
ALWAYS open and follow `docs/checklists/FEATURE.md` as a quality checklist

## Hyperspot Deltas vs Original Cypilot SDLC

- Hyperspot uses `FEATURE` artifacts as the implementable unit.
- The feature ID is defined in DECOMPOSITION; FEATURE references it via `cpt-{system}-feature-{slug}` (do not redefine).

## Generation Checklist

- [ ] Reference the DECOMPOSITION feature ID via `cpt-{system}-feature-{slug}` (do not redefine the same feature ID).
- [ ] Define flows/algorithms/states/requirements at implementable detail level (inputs/outputs, errors, edge cases).
- [ ] Keep details consistent with ModKit patterns, secure ORM, and OData/OpenAPI conventions.
- [ ] Write acceptance criteria that are testable and map to `make test`/integration/E2E expectations.

## Validation Checklist

- [ ] `python3 {cypilot_path}/skills/cypilot/scripts/cypilot.py validate --artifact <path>`
- [ ] Review against `docs/checklists/FEATURE.md`.
