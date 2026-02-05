# Testing (Hyperspot)

**Version**: 1.0  
**Purpose**: Standardize test types, how to run them, and what “good coverage” means in this repo.  
**Scope**: Workspace tests and CI test jobs.  

## Test categories

- Unit tests: `cargo test --workspace`
- Integration tests (DB-backed): module-specific feature-gated tests (SQLite/Postgres/MySQL variants exist).
- UI tests: `trybuild` tests for macro crates run in CI.
- E2E tests: `scripts/ci.py e2e` (local or docker).

## How to run

- Local default: `make test`
- DB integration: `make test-db` (or `test-sqlite` / `test-pg` / `test-mysql`)
- UI macro tests: CI has a dedicated job; locally: run the specific `cargo test -p <crate> --test ui`.

## Coverage

- Coverage tooling uses `cargo-llvm-cov` via `make coverage*`.

## Validation Criteria

- [ ] New behavior has at least unit tests; DB logic has integration tests where applicable.
- [ ] Feature flags for integration tests are documented in the crate README if non-obvious.
- [ ] Tests run via `make test` and remain deterministic.

## Examples

✅ Valid:
- Add an integration test under the crate with an `integration` feature, then wire it into Makefile target if needed.

❌ Invalid:
- Add tests that require local secrets or manual setup without documenting in `scripts/check_local_env.py` or README.

---

**Source**: `Makefile` targets (`test*`, `e2e*`, `coverage*`), `.github/workflows/ci.yml`.  
**Last Updated**: 2026-02-05

