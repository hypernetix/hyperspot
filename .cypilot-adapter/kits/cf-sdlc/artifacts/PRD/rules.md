# PRD Rules (Hyperspot)


## Required References (ALWAYS)

ALWAYS open and follow `../../rules.md`

ALWAYS open and follow `docs/spec-templates/PRD.md` as a template

ALWAYS open and follow `docs/checklists/PRD.md` as a quality checklist


## Hyperspot Deltas vs Original Cypilot SDLC

- This kit enforces constraints.json (allowed/required ID kinds and cross-artifact reference coverage).


## Constraints Alignment (REQUIRED)

PRD MUST define at least one ID of each required kind:

- `fr` (Functional Requirement) — task+priority REQUIRED → use checkbox ID definition form.
- `nfr` (Non-functional Requirement) — task+priority REQUIRED → use checkbox ID definition form.
- `usecase` (Use Case) — task+priority allowed.

PRD MUST NOT reference IDs from prohibited artifact kinds using backticks:

- Do NOT include backtick references to DESIGN IDs (principle/constraint/component/seq).
- Do NOT include backtick references to ADR IDs.


## Generation Checklist

- [ ] Populate all required sections; remove placeholders (no TODO/TBD/FIXME).
- [ ] Define concrete actors (human + system) and reuse actor IDs consistently.
- [ ] Write measurable success criteria (baseline + target + timeframe where possible).
- [ ] Define FRs/NFRs as WHAT, not HOW; include priorities and actor references.
- [ ] Ensure every listed capability is backed by at least one FR and at least one use case.
- [ ] Keep implementation details (routes/DB schemas) out of PRD; defer to DESIGN/FEATURE.

## Validation Checklist

- [ ] `python3 {cypilot_path}/skills/cypilot/scripts/cypilot.py validate --artifact <path>`
- [ ] Review against `docs/checklists/PRD.md`.
