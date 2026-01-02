# New Module Acceptance Criteria

Use this checklist to validate a newly scaffolded module before integration.

General
- [ ] Only files under `modules/<name>-sdk` and `modules/<name>` were created/modified by the tool.
- [ ] No changes were made to root `Cargo.toml`, server Cargo.toml, or `apps/hyperspot-server/src/registered_modules.rs` (manual wiring instructions were printed instead).

Build & Quality
- [ ] `cargo check --workspace` succeeds.
- [ ] `cargo fmt --all --check` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes for the new crates.

API & Behavior
- [ ] SDK defines `<PascalCase>Api`, `models::Health`, and `errors::<PascalCase>Error` (no `serde` derives in SDK types).
- [ ] Module implements a local client that registers into ClientHub: `ctx.client_hub().register::<dyn <PascalCase>Api>(...)`.
- [ ] REST GET `/<kebab>/v1/health` is registered via `OperationBuilder`, appears in OpenAPI, and returns JSON `HealthDto`.
- [ ] RFC-9457 `Problem` mapping exists: `impl From<DomainError> for Problem` with standard statuses.
- [ ] Handlers use `modkit::api::prelude::*` and accept `SecurityCtx` via `Authz` extractor.

Testing
- [ ] Handler smoke test for `/health` returns `200 OK`.
- [ ] Unit tests compile and pass (if present).

Design Conformance
- [ ] SDK is transport-agnostic; DTOs with `serde`/`ToSchema` live in `src/api/rest/dto.rs` (module crate).
- [ ] Error conversion exists: `From<DomainError> for <PascalCase>Error`.
- [ ] No global statics; dependency injection via `Arc` and `ArcSwapOption`.

Wiring (manual)
- [ ] Add `use <snake> as _;` in `apps/hyperspot-server/src/registered_modules.rs`.
- [ ] Add dependency `
    <snake> = { path = "../../modules/<snake>" }
  ` under `# user modules` in `apps/hyperspot-server/Cargo.toml`.
- [ ] Add `modules/<snake>-sdk` and `modules/<snake>` to root `[workspace].members`.
