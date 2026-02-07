# FEATURE Rules (Hyperspot)

## Required References (ALWAYS)

ALWAYS open and follow `../../rules.md`

ALWAYS open and follow `docs/spec-templates/FEATURE.md` as a template
ALWAYS open and follow `docs/checklists/FEATURE.md` as a quality checklist

ALWAYS open and follow `{spaider_adapter_path}/specs/api-contracts.md` WHEN specifying endpoints, OpenAPI, OData, or error formats

ALWAYS open and follow `{spaider_adapter_path}/specs/security.md` and `{spaider_adapter_path}/specs/data-governance.md` WHEN specifying persistence and access control

ALWAYS open and follow `{spaider_adapter_path}/specs/testing.md` WHEN writing acceptance criteria and test strategy

ALWAYS open and follow `{spaider_adapter_path}/specs/performance.md` and `{spaider_adapter_path}/specs/reliability.md` WHEN specifying latency/retry/timeout behavior

## Hyperspot Deltas vs Original Spaider SDLC

- Original SDLC uses `SPEC` artifacts; Hyperspot uses `FEATURE` artifacts as the implementable unit.
- The feature ID is defined in DECOMPOSITION; FEATURE references it via `id-ref:feature` (do not redefine).

## Generation Checklist

- [ ] Reference the DECOMPOSITION feature ID via `id-ref:feature` (do not redefine the same feature ID).
- [ ] Define flows/algorithms/states/requirements at implementable detail level (inputs/outputs, errors, edge cases).
- [ ] Keep details consistent with ModKit patterns, secure ORM, and OData/OpenAPI conventions.
- [ ] Write acceptance criteria that are testable and map to `make test`/integration/E2E expectations.

## Validation Checklist

- [ ] `python3 {spaider_path}/skills/spaider/scripts/spaider.py validate --artifact <path>`
- [ ] `python3 {spaider_path}/skills/spaider/scripts/spaider.py list-ids --artifact <path>` and confirm no duplicates.
- [ ] Review against `docs/checklists/FEATURE.md` and `{spaider_path}/weavers/sdlc/artifacts/SPEC/checklist.md`.
