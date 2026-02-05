# Build & Deploy (Hyperspot)

**Version**: 1.0  
**Purpose**: Document the canonical build/test/check entrypoints and release automation used by this repo.  
**Scope**: Local developer workflows and GitHub Actions CI.  

## Canonical developer commands

- `make fmt` — formatting check
- `make clippy` — clippy on workspace (deny warnings)
- `make test` — workspace tests
- `make check` — full quality gate (fmt, clippy, security, dylint, gts-docs, tests)
- `make ci` — alias for `make check`
- `make build` — release build

## Release automation

- Release PR automation is configured via `release-plz.toml` and workflow `release-plz.yml`.

## CI

- Primary CI workflow is `.github/workflows/ci.yml`:
  - `cargo fmt --check`
  - `cargo clippy` (deny warnings)
  - `cargo test` (workspace)
  - UI tests for macro crates
  - security checks (`cargo-deny`)
  - coverage job (`cargo-llvm-cov`)

## Validation Criteria

- [ ] New checks belong in `make check` if they are required gates.
- [ ] CI changes keep cross-platform compatibility (Linux/macOS/Windows).
- [ ] Release changes follow release-plz conventions (single changelog).

## Examples

✅ Valid:
- Add a new `make` target and call it from `check` when it’s a required gate.

❌ Invalid:
- Add a CI-only script path that doesn’t exist locally or isn’t tracked.

---

**Source**: `Makefile`, `.github/workflows/ci.yml`, `release-plz.toml`, `.github/workflows/release-plz.yml`.  
**Last Updated**: 2026-02-05

