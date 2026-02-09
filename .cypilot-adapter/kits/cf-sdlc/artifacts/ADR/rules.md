# ADR Rules (CyberFabric)

## Required References (ALWAYS)

ALWAYS open and follow `../../rules.md`

ALWAYS open and use `docs/spec-templates/ADR.md` as a template

ALWAYS open and use `docs/spec-templates/examples/todo-app/ADR/*.md` as an examples

ALWAYS open and follow `docs/checklists/ADR.md` as a quality checklist

## Constraints Alignment (REQUIRED)

ADR MUST define an `adr` ID (plain **ID**: `cpt-{system}-adr-{slug}` form; task/priority prohibited).

PRD MUST NOT reference ADR IDs as backtick IDs.

ADR IDs MUST be referenced from DESIGN (ensure the DESIGN includes backtick references to relevant ADR IDs).

## Generation Checklist

- [ ] Capture the problem statement, drivers, options considered, and the decision with consequences.
- [ ] Keep ADRs decisions immutable once ACCEPTED, allow only structural or syntax or grammar changes
- [ ] Link related PRD or DESIGN elements by ID (actors/requirements/principles/constraints/components).

## Validation Checklist

- [ ] `python3 {cypilot_path}/skills/cypilot/scripts/cypilot.py validate --artifact <path>`
- [ ] `make validate-artifacts`
- [ ] Review against `docs/checklists/ADR.md`
