# DESIGN Rules (Hyperspot)

## Required References (ALWAYS)

ALWAYS open and follow `../../rules.md`

ALWAYS open and follow `docs/spec-templates/DESIGN.md` as a template

ALWAYS open and follow `docs/checklists/DESIGN.md` as a quality checklist

ALWAYS open and follow `{spaider_adapter_path}/specs/patterns.md` WHEN choosing module boundaries, layering, ClientHub, or gateway patterns

ALWAYS open and follow `{spaider_adapter_path}/specs/api-contracts.md` WHEN defining REST/OpenAPI or OData behavior

ALWAYS open and follow `{spaider_adapter_path}/specs/data-governance.md` and `{spaider_adapter_path}/specs/security.md` WHEN defining persistence, tenancy, and access control

ALWAYS open and follow `{spaider_adapter_path}/specs/observability.md`, `{spaider_adapter_path}/specs/performance.md`, `{spaider_adapter_path}/specs/reliability.md` WHEN defining operational constraints

## Hyperspot Deltas vs Original Spaider SDLC

- Downstream chain uses DECOMPOSITION → FEATURE (no `SPEC` in this weaver).
- Prefer decisions aligned with ModKit patterns and the secure ORM (`SecurityCtx` request-scoped).

## Generation Checklist

- [ ] Reference PRD FR/NFR IDs and map them to system-level design (WHAT → HOW at architecture level).
- [ ] Define components/sequences/data that will be decomposed later (keep feature-level detail out).
- [ ] Capture principles/constraints explicitly and link ADRs where decisions are material.
- [ ] Keep alignment with ModKit module structure and secure ORM constraints.

## Validation Checklist

- [ ] `python3 {spaider_path}/skills/spaider/scripts/spaider.py validate --artifact <path>`
- [ ] `python3 {spaider_path}/skills/spaider/scripts/spaider.py list-ids --artifact <path>` and confirm no duplicates.
- [ ] Review against `docs/checklists/DESIGN.md` and `{spaider_path}/weavers/sdlc/artifacts/DESIGN/checklist.md`.
- [ ] Ensure referenced PRD IDs exist (use `where-defined` if needed).
