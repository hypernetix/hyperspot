# ADR Rules (Hyperspot)

## Required References (ALWAYS)

ALWAYS open and follow `../../rules.md`

ALWAYS open and use `docs/spec-templates/ADR.md` as a template

ALWAYS open and follow `docs/checklists/ADR.md` as a quality checklist

## Constraints Alignment (REQUIRED)

ADR MUST define an `adr` ID (plain `**ID**: `...`` form; task/priority prohibited).

PRD MUST NOT reference ADR IDs as backtick IDs.

ADR IDs MUST be referenced from DESIGN (ensure the DESIGN includes backtick references to relevant ADR IDs).

## Generation Checklist

- [ ] Capture the problem statement, drivers, options considered, and the decision with consequences.
- [ ] Keep ADRs decisions immutable once ACCEPTED, allow only structural or syntax or grammar changes
- [ ] Link related design elements by ID (actors/requirements/principles/constraints/components).

## Validation Checklist

- [ ] `python3 {cypilot_path}/skills/cypilot/scripts/cypilot.py validate --artifact <path>`
- [ ] Review against `docs/checklists/ADR.md`
