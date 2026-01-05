---
name: hyperspot-module-scaffold
description: Generate and validate new Hyperspot modules (SDK + implementation) with REST capability using scripts/new-module-scaffold/main.py. Use when creating a new module scaffold and wiring it into the workspace
---

# Hyperspot Module Scaffold

This skill guides Agent to create a minimal Hyperspot module via the repository's scaffold tool.

## Quick Start

- Run:
  - `python scripts/new-module-scaffold/main.py <module_name> --with-db --validate`
- Apply the printed wiring instructions to:
  - Root `Cargo.toml` workspace members
  - `apps/hyperspot-server/Cargo.toml` dependency
  - `apps/hyperspot-server/src/registered_modules.rs` import
- Validate:
  - `cargo check --workspace && cargo fmt --all && cargo clippy --workspace`
  - `cargo test -p <module_name> --test smoke`

## Inputs

- `module_name` (snake_case): e.g., `users_info`, `types_registry`
- Flags:
  - `--force`: overwrite existing files
  - `--validate`: run cargo check/fmt after generation
  - `--with-db`: generate example scaffold for db capability

## Outputs

Creates two crates in a parent directory:

```
modules/<module_name>/
├── <module_name>-sdk/          # Public API crate
└── <module_name>/              # Implementation crate
```

Includes REST health endpoint at `GET /<kebab>/v1/health`, ClientHub local client registration, OpenAPI schema, and RFC-9457 Problem mapping. All templates use canonical Rust module glue (mod.rs) and satisfy workspace clippy lints.

## Workflow (imperative)

1. Validate input name: must match `^[a-z0-9_]+$`.
2. Execute scaffold:
   - `python scripts/new-module-scaffold/main.py <module_name> [--force] [--validate]`
3. Apply wiring instructions exactly as printed by the tool.
4. Run validations:
   - `cargo check --workspace`
   - `cargo fmt --all`
   - `cargo clippy --workspace`
   - `cargo test -p <module_name> --test smoke`
5. Start server and confirm:
   - Health endpoint: `GET /<kebab>/v1/health` returns 200
   - OpenAPI docs available
6. Follow instruction in `@guidelines/NEW_MODULE.md` to extend module according to the requirements and user prompt

## References (load on demand)

- `guidelines/NEW_MODULE.md` — canonical module guidance

Only load references if implementation details are needed; keep SKILL.md context lean.

## Degrees of Freedom

- Default: medium freedom (deterministic script + minimal configuration)
- Optional flags control overwrite/validation; extend only if necessary (e.g., DB/SSE in future variants).

## Failure Modes & Fixes

- Invalid module name: ensure snake_case; rerun.
- `modules/` not found: run from workspace root; confirm repository layout.
- Workspace errors: add generated paths to `[workspace].members` exactly as printed.
- Import errors (http::StatusCode): use `modkit::api::prelude::StatusCode`.
- Test failure (500): ensure route is `.public()` and handler uses `SecurityCtx::root_ctx()` (public health).
- Clippy lints: ensure docs use backticks; add `#[must_use]`; prefer `to_owned()`; avoid unnecessary clones; return `Router` directly when infallible.

## Notes

- The generator only writes under `modules/<module_name>/`; it prints wiring lines without editing existing files.
- All templates use canonical Rust module layout with `mod.rs` to align with workspace lints and compilation.
