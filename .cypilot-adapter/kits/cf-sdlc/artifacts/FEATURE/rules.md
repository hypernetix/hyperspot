# FEATURE Rules (Hyperspot)

## Required References (ALWAYS)

ALWAYS open and follow `../../rules.md`

ALWAYS open and follow `docs/spec-templates/FEATURE.md` as a template
ALWAYS open and follow `docs/checklists/FEATURE.md` as a quality checklist

## Hyperspot Deltas vs Original Cypilot SDLC

- Hyperspot uses `FEATURE` artifacts as the implementable unit.

## Constraints Alignment (REQUIRED)

FEATURE MUST define at least one `dod` ID.

- `dod` requires task+priority → use checkbox ID definition form (`- [ ] `pN` - **ID**: ...`).
- `dod` is `to_code: true` (intended to drive implementation + traceability).

Optional ID kinds allowed in FEATURE:

- `flow`
- `algo`
- `state`
- `featurecontext`

## Generation Checklist

- [ ] Ensure at least one `dod` exists and is written as testable acceptance criteria.
- [ ] Ensure the FEATURE references its corresponding DECOMPOSITION `feature` ID (as a backtick reference) so DECOMPOSITION coverage requirements can pass.
- [ ] Define flows/algorithms/states/requirements at implementable detail level (inputs/outputs, errors, edge cases).
- [ ] Keep details consistent with ModKit patterns, secure ORM, and OData/OpenAPI conventions.
- [ ] Write acceptance criteria that are testable and map to `make test`/integration/E2E expectations.

## Validation Checklist

- [ ] `python3 {cypilot_path}/skills/cypilot/scripts/cypilot.py validate --artifact <path>`
- [ ] Review against `docs/checklists/FEATURE.md`.
