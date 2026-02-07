# ADR Rules (Hyperspot)

## Required References (ALWAYS)

ALWAYS open and follow `../../rules.md`

ALWAYS open and use `docs/spec-templates/ADR.md` as a template

ALWAYS open and follow `docs/checklists/ADR.md` as a quality checklist

ALWAYS open and follow `{spaider_adapter_path}/specs/tech-stack.md` WHEN making stack decisions (Rust, Axum, Utoipa, SeaORM, Tonic)

ALWAYS open and follow `{spaider_adapter_path}/specs/security.md` WHEN decisions impact authn/authz, tenancy, secrets, or PII

ALWAYS open and follow `{spaider_adapter_path}/specs/build-deploy.md` and `{spaider_adapter_path}/specs/compliance.md` WHEN decisions impact CI, supply-chain, or policy tooling

## Hyperspot Deltas vs Original Spaider SDLC

- Prefer decisions that align with repository constraints (ModKit patterns, secure ORM, OData `$select`, CI safety tooling).

## Generation Checklist

- [ ] Capture the problem statement, drivers, options considered, and the decision with consequences.
- [ ] Keep ADRs immutable once ACCEPTED; supersede via a new ADR.
- [ ] Link related design elements by ID (actors/requirements/principles/constraints/components).

## Validation Checklist

- [ ] `python3 {spaider_path}/skills/spaider/scripts/spaider.py validate --artifact <path>`
- [ ] `python3 {spaider_path}/skills/spaider/scripts/spaider.py list-ids --artifact <path>` and confirm no duplicates.
- [ ] Review against `docs/checklists/ADR.md`
