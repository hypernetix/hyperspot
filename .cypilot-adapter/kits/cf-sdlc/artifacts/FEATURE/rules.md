# FEATURE Rules (CyberFabric)

## Required References (ALWAYS)

ALWAYS open and follow `../../rules.md`

ALWAYS open and use `docs/spec-templates/FEATURE.md` as a template
ALWAYS open and use `docs/spec-templates/examples/todo-app/features/*.md` as examples
ALWAYS open and follow `docs/checklists/FEATURE.md` as a quality checklist

## CyberFabric Deltas vs Original Cypilot SDLC

- CyberFabric uses `FEATURE` artifacts as the implementable unit.

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
- [ ] `make cypilot-validate`
- [ ] Review against `docs/checklists/FEATURE.md`.

## Review Checklist

- [ ] Flows cover happy path, errors, edge cases; algorithms testable
- [ ] No type redefinitions, new API endpoints, code snippets, or decision debates
- [ ] DoD criteria testable and map to `make test`/integration/E2E
- [ ] ModKit patterns, secure ORM, security checks in flows
- [ ] Run `docs/checklists/FEATURE.md` for full validation
