# PRD Rules (Hyperspot)


## Required References (ALWAYS)

ALWAYS open and follow `../../rules.md`

ALWAYS open and follow `docs/spec-templates/PRD.md` as a template

ALWAYS open and follow `docs/checklists/PRD.md` as a quality checklist

ALWAYS open and follow `{spaider_adapter_path}/specs/project-structure.md` WHEN choosing locations for new artifacts

ALWAYS open and follow `{spaider_adapter_path}/specs/tech-stack.md` WHEN stating platform constraints or runtime assumptions

ALWAYS open and follow `{spaider_adapter_path}/specs/security.md` WHEN writing authn/authz, tenancy, PII, or audit requirements

ALWAYS open and follow `{spaider_adapter_path}/specs/reliability.md` and `{spaider_adapter_path}/specs/observability.md` WHEN writing operational requirements

## Hyperspot Deltas vs Original Spaider SDLC

- Downstream chain is PRD → DESIGN → DECOMPOSITION → FEATURE (no `SPEC` in this weaver).
- Marker-kind names avoid hyphens (e.g. `prdcontext`).

## Generation Checklist

- [ ] Populate all required sections; remove placeholders (no TODO/TBD/FIXME).
- [ ] Define concrete actors (human + system) and reuse actor IDs consistently.
- [ ] Write measurable success criteria (baseline + target + timeframe where possible).
- [ ] Define FRs/NFRs as WHAT, not HOW; include priorities and actor references.
- [ ] Ensure every listed capability is backed by at least one FR and at least one use case.
- [ ] Keep implementation details (routes/DB schemas) out of PRD; defer to DESIGN/FEATURE.

## Validation Checklist

- [ ] `python3 {spaider_path}/skills/spaider/scripts/spaider.py validate --artifact <path>`
- [ ] `python3 {spaider_path}/skills/spaider/scripts/spaider.py list-ids --artifact <path>` and confirm no duplicates.
- [ ] Review against `docs/checklists/PRD.md` and `{spaider_path}/weavers/sdlc/artifacts/PRD/checklist.md`.
- [ ] Confirm `covered_by` warnings are expected until downstream artifacts exist (DESIGN/DECOMPOSITION/FEATURE).
