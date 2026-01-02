# New Module Scaffold Templater Design

Goal: Implement scripts/new-module-scaffold/main.py that generates a minimal Hyperspot module (SDK + module crates) from a snake_case name, updates server wiring, and workspace members.

Inputs:
- module_name (snake_case), e.g., users_info.

Derived names:
- snake = module_name (e.g., users_info)
- kebab = users-info (underscores -> dashes)
- PascalCase = UsersInfo (split by '_' and capitalize each segment)
- sdk_crate = "<snake>-sdk"
- sdk_path = modules/<snake>-sdk
- module_path = modules/<snake>

Architecture:
- Single Python entrypoint (main.py) using standard library only.
- Template rendering via str.format() with a minimal placeholder map (no external deps).
- File system ops: create directories recursively; write files atomically; refuse to overwrite existing files unless --force.
- Do not edit any files outside new module crates; print manual wiring instructions instead.

Templates:
- Cargo.toml (SDK and module) with placeholders: {snake}, {pascal}, relative local deps.
- Rust sources: lib.rs, module.rs, config.rs, local_client.rs, domain/*.rs, api/rest/*.rs, tests/smoke.rs.
- All Rust files compile without warnings under workspace lints.

Server wiring (instructions only):
- Print the dependency line to add under "# user modules": {snake} = { path = "../../modules/{snake}" }.
- Print the exact "use {snake} as _;" line to add in registered_modules.rs.

Workspace membership (instructions only):
- Print members entries to add in root Cargo.toml: "modules/{snake}-sdk" and "modules/{snake}" under [workspace].members.

Validation strategy:
- After generation, optionally run (if --validate):
  - cargo check --workspace
  - cargo fmt --all
- Default: dry filesystem operations only (no external commands); user runs make/check.

Error handling:
- Validate input naming (snake_case: [a-z0-9_]+), refuse invalid names.
- If target paths exist and not empty, abort unless --force.
- On edit failures (anchors not found), print actionable message and exit non-zero.

Extensibility:
- Future flags: --with-db, --with-sse, --gateway, --plugin; compose additional templates.
- Abstract template registry to a dict; easy to plug additional files.

Pseudocode:
1. parse_args() -> module_name, flags
2. derive_names()
3. assert_valid(module_name)
4. generate_sdk_crate(sdk_path, snake, pascal)
5. generate_module_crate(module_path, snake, pascal, kebab)
6. emit_server_wiring_instructions(snake)
7. emit_registered_modules_instructions(snake)
8. emit_workspace_members_instructions(snake)
9. print success summary

Security:
- No network calls; no external template fetching.
- Do not modify files outside new module crates; only write under modules/<snake>-sdk and modules/<snake>.
