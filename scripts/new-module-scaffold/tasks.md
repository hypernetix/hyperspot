# Tasks: New Module Scaffold

# Overview:
- Goal:  Provide a minimal, production-ready scaffold-tool to generate code for a new Hyperspot module following guidelines/NEW_MODULE.md.
- Input: module name in snake_case (e.g., users_info, types_registry). The tool derives PascalCase for types and kebab-case for REST paths when needed.
- Output: Two crates under modules/: <name>-sdk and <name>, with DDD-light layout, REST-ready, inventory-registered, and ClientHub-local client wiring.


## 1. Define naming and inputs
- Description: Parse snake_case module name and derive kebab-case and PascalCase.
- Acceptance criteria:
  - [ ] Reject names not matching ^[a-z0-9_]+$.
  - [ ] Derive kebab = underscores->dashes; PascalCase = capitalize segments.
  - [ ] Print a summary of derived names.

## 2. Generate SDK crate
- Description: Create modules/<snake>-sdk with Cargo.toml and src/{lib.rs,api.rs,models.rs,errors.rs}.
- Acceptance criteria:
  - [ ] SDK Cargo.toml exists with workspace lints and minimal deps.
  - [ ] lib.rs re-exports api/errors/models; forbids unsafe.
  - [ ] api.rs defines trait <PascalCase>Api with health(ctx) -> Result<Health, <PascalCase>Error>.
  - [ ] models.rs defines Health (no serde derives).
  - [ ] errors.rs defines <PascalCase>Error enum.

## 3. Generate Module crate
- Description: Create modules/<snake> with Cargo.toml and src tree: lib.rs, module.rs, config.rs, local_client.rs, domain/{error.rs,service.rs,ports.rs,repo.rs}, api/rest/{dto.rs,handlers.rs,routes.rs,error.rs}, tests/smoke.rs.
- Acceptance criteria:
  - [ ] Module Cargo.toml depends on <snake>-sdk and local libs (modkit, modkit-auth).
  - [ ] module.rs has #[modkit::module(name = "<snake>", capabilities = [rest])] and stores Service in ArcSwapOption.
  - [ ] init() registers local client into ClientHub for <PascalCase>Api.
  - [ ] RestfulModule::register_rest attaches service and registers routes.
  - [ ] handlers.rs compiles and returns ApiResult<JsonBody<HealthDto>> for GET /health.
  - [ ] error.rs maps DomainError -> Problem with standard statuses.

## 4. Emit wiring instructions (no file changes)
- Description: Print exact lines to add in apps/hyperspot-server/Cargo.toml and registered_modules.rs; do not modify files.
- Acceptance criteria:
  - [ ] Outputs the dependency line for server Cargo.toml and the "use {snake} as _;" line for registered_modules.rs.
  - [ ] Does not modify any files outside modules/{snake}-sdk and modules/{snake}.
  - [ ] Idempotent output: repeated runs print the same instructions without side effects.

## 5. Emit workspace members instructions (no file changes)
- Description: Print SDK and module paths to add to root Cargo.toml [workspace].members; do not modify files.
- Acceptance criteria:
  - [ ] Outputs "modules/{snake}-sdk" and "modules/{snake}" entries.
  - [ ] Explains where to insert (near existing modules) while preserving formatting.
  - [ ] Does not modify any files outside modules/{snake}-sdk and modules/{snake}.

## 6. Post-gen validation (optional)
- Description: If --validate, run cargo check and fmt to verify scaffold compiles.
- Acceptance criteria:
  - [ ] cargo check --workspace succeeds.
  - [ ] Generated crates compile with no warnings under workspace lints.
  - [ ] GET /<kebab>/v1/health returns 200 in a handler smoke test.

## 7. Documentation and help
- Description: Provide --help text describing behavior, templates, and next steps.
- Acceptance criteria:
  - [ ] --help prints usage, examples, and flags.
  - [ ] Mentions how to enable DB/SSE/plugin scaffolds in future.

## 8. Generate acceptance criteria doc
- Description: Create scripts/new-module-scaffold/new_module_acceptance.md with general acceptance criteria for the new module.
- Acceptance criteria:
  - [ ] File exists at scripts/new-module-scaffold/new_module_acceptance.md.
  - [ ] File should be copied to modules/<snake>/new_module_acceptance.md.
  - [ ] Lists criteria: compile, lint/fmt pass, health endpoint + OpenAPI, ClientHub registration, Problem mapping, SecurityCtx usage, no changes outside new module crates.
