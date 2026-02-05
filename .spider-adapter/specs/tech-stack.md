# Tech Stack (Hyperspot)

**Version**: 1.0  
**Purpose**: Standardize technology choices and how we use them (Rust, ModKit, REST/OpenAPI, DB, gRPC, observability).  
**Scope**: This repository (`hyperspot`) and all workspace crates.  

## Languages & Runtime

- Primary language: Rust (workspace).
- Async runtime: `tokio` (full).

## Web / REST / OpenAPI

- HTTP framework: `axum` (used primarily by the gateway module).
- OpenAPI generation: `utoipa`.
- API error format: RFC-9457 `Problem` via ModKit error utilities.
- OData query options: via `modkit-odata` + `$select` support (field projection).

## Database & Storage

- ORM: `sea-orm` + `sea-orm-migration`.
- Secure ORM wrapper: `modkit-db` secure layer (request-scoped `SecurityCtx`, typestate gating).

## gRPC

- gRPC stack: `tonic` (+ `prost`).

## Observability

- Logging/tracing: `tracing`, `tracing-subscriber`.
- OpenTelemetry: `opentelemetry`, `tracing-opentelemetry`, `opentelemetry-otlp`.

## Tooling (repo-level)

- Formatting: `cargo fmt` (rustfmt).
- Linting: `cargo clippy` (workspace `-D warnings` in CI).
- Custom architectural lints: Dylint (`dylint_lints/`) for layer rules and REST conventions.
- Supply-chain: `cargo-deny` (RustSec advisories, license policy).
- Coverage: `cargo-llvm-cov` via `scripts/coverage.py`.

## Validation Criteria

- [ ] New features use existing stack components unless there is a documented reason to add/replace.
- [ ] REST endpoints use ModKit patterns and OpenAPI annotations where applicable.
- [ ] DB access uses the secure ORM APIs (no unscoped execution paths).
- [ ] Cross-module APIs prefer SDK traits + ClientHub patterns (ModKit).

## Examples

✅ Valid:
- Add a REST route via the gateway module using ModKit REST builder + `utoipa`.
- Use `modkit_db::secure::SecureConn` + `SecurityCtx` for DB access.

❌ Invalid:
- Introduce a second async runtime or a parallel web framework without a clear need.
- Execute SeaORM queries directly when the secure wrapper is required.

---

**Source**: `Cargo.toml` (`[workspace.dependencies]`), `Makefile`, `docs/MODKIT_UNIFIED_SYSTEM.md`, `docs/SECURE-ORM.md`, `docs/ODATA_SELECT.md`.  
**Last Updated**: 2026-02-05

