# Project Structure (Hyperspot)

**Version**: 1.0  
**Purpose**: Make navigation and file placement predictable across the Rust workspace (apps/libs/modules/examples).  
**Scope**: This repository (`hyperspot`) and all workspace crates.  

## Overview

Hyperspot is a Rust workspace organized as:
- `apps/` — runnable binaries and tooling apps (e.g., server, validators).
- `libs/` — shared libraries (ModKit + supporting crates).
- `modules/` — composable product/system modules (often paired with `*-sdk` crates).
- `examples/` — example modules and patterns (used as references for new work).
- `config/` — YAML configs used for local runs and CI.
- `docs/` + `guidelines/` — architecture and engineering guidance.
- `dylint_lints/` — custom lints enforcing architecture/layer rules.

## Where to add things

### New runtime/service binary
- Add under `apps/<name>/` and register as a workspace member.

### New shared library
- Add under `libs/<name>/` (prefer `libs/` for reusable, cross-module code).

### New module (product/system)
- Add under `modules/<name>/` following the SDK pattern (`<module>_sdk` or `<module>-sdk` + `<module>` impl crate as per repo conventions).

### New docs
- Architecture-level guidance: `docs/`
- Developer processes and coding standards: `guidelines/`
- Generated or exported API spec: `docs/api/`

## Workspace membership

All crates must be added to the root `Cargo.toml` workspace `members` list.

## Validation Criteria

- [ ] New crates live under `apps/`, `libs/`, `modules/`, or `examples/` (unless there is a clear reason).
- [ ] New crates are added to root `Cargo.toml` workspace `members`.
- [ ] Modules follow the documented ModKit module layout and SDK separation.
- [ ] Changes do not introduce ad-hoc top-level directories without justification.

## Examples

✅ Valid:
- Add a server binary in `apps/hyperspot-server/`.
- Add a reusable crate in `libs/modkit-foo/`.
- Add a module in `modules/my_module/` with an SDK crate.

❌ Invalid:
- Put reusable code directly into `apps/` without factoring into `libs/`.
- Create a new top-level `src/` at repo root for product code.

---

**Source**: `Cargo.toml` (workspace members), `README.md`, `docs/MODKIT_UNIFIED_SYSTEM.md`, `guidelines/NEW_MODULE.md`.  
**Last Updated**: 2026-02-05

