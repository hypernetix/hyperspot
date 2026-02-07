# DECOMPOSITION Rules (Hyperspot)

## Required References (ALWAYS)

ALWAYS open and follow `../../rules.md`

ALWAYS open and follow `docs/spec-templates/DECOMPOSITION.md` as a template
ALWAYS open and follow `docs/checklists/DECOMPOSITION.md` as a quality checklist

ALWAYS open and follow `{spaider_adapter_path}/specs/patterns.md` WHEN mapping design elements to features and boundaries

ALWAYS open and follow `{spaider_adapter_path}/specs/project-structure.md` WHEN choosing locations for FEATURE artifacts

## CyberFabric SDLC Chain (this weaver)

- PRD → DESIGN → DECOMPOSITION → FEATURE
- NOTE: No `SPEC` kind in this weaver.

## Hyperspot Deltas vs Original Spaider SDLC

- Original SDLC decomposes DESIGN into `SPEC` entries; Hyperspot decomposes DESIGN into `FEATURE` entries.
- Each entry MUST link to a corresponding FEATURE artifact; FEATURE references the feature ID via `id-ref:feature` (do not redefine).

## Generation Checklist

- [ ] Decompose DESIGN components/sequences/data into features with high cohesion and clear boundaries.
- [ ] Ensure 100% coverage: every relevant DESIGN element appears in at least one feature entry.
- [ ] Avoid overlap: design elements should not be duplicated across features without an explicit reason.
- [ ] Assign priorities (`p1`-`p9`) and keep dependencies explicit and acyclic.
- [ ] Ensure each DECOMPOSITION entry links to a corresponding FEATURE artifact path and that FEATURE is registered (if applicable).

## Validation Checklist

- [ ] `python3 {spaider_path}/skills/spaider/scripts/spaider.py validate --artifact <path>`
- [ ] `python3 {spaider_path}/skills/spaider/scripts/spaider.py list-ids --artifact <path>` and confirm no duplicates.
- [ ] Review against `docs/checklists/DECOMPOSITION.md`.
