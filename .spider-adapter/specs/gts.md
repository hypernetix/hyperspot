# GTS (Global Type System) Conventions

**Version**: 1.0  
**Purpose**: Capture how GTS identifiers and related docs are validated in this repository.  
**Scope**: Docs and code referencing GTS identifiers (modules/libs/examples).  

## Identifier policy

- GTS identifiers used in docs/code should follow the validated patterns enforced by the docs validator and lints.

## Validation tooling

- `make gts-docs` runs the `gts-docs-validator` over `.md`/`.json` (and YAML) in `docs`, `modules`, `libs`, `examples`.
- Dylint includes GTS-related checks (`DE09xx`).

## Validation Criteria

- [ ] Docs changes keep `make gts-docs` passing.
- [ ] Code changes keep GTS-related lints passing.
- [ ] New docs follow the established ID conventions (vendor, format).

## Examples

✅ Valid:
- Add a new identifier and verify it via `make gts-docs`.

❌ Invalid:
- Introduce ad-hoc identifier formats that break validation.

---

**Source**: `Makefile` (`gts-docs*` targets), `dylint_lints/README.md` (`DE09xx`), `.github/workflows/ci.yml`.  
**Last Updated**: 2026-02-05

