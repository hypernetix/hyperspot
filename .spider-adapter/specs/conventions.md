# Conventions (Hyperspot)

**Version**: 1.0  
**Purpose**: Capture repo-wide style, safety, and architecture guardrails that affect everyday coding.  
**Scope**: All Rust code and repository changes.  

## Formatting

- `rustfmt.toml` sets `max_width = 100` and Unix newlines.
- Enforce formatting in CI (`cargo fmt -- --check`).

## Linting & quality bars

- CI runs `cargo clippy -- -D warnings` and uses strict workspace lints.
- `clippy.toml` disallows unsafe DB query execution methods to enforce secure ORM usage (via `clippy::disallowed_methods` when enabled in crates).
- Dylint lints enforce architectural boundaries (contract/api/rest conventions, GTS rules).

## Architecture conventions (ModKit)

- Follow the ModKit DDD-light layering and module layout.
- Prefer SDK pattern for module public surfaces (consumer depends on SDK; module crate provides implementation and adapters).

## Docs & line endings

- `.editorconfig` enforces LF and final newline; `.bat` uses CRLF.

## Validation Criteria

- [ ] Code is `cargo fmt` clean.
- [ ] Clippy is clean under workspace settings (no warnings).
- [ ] Layering boundaries are maintained (Dylint lints stay passing).
- [ ] Secure ORM rules are followed where applicable.

## Examples

✅ Valid:
- Keep REST DTOs confined to REST layer and out of contract/domain layers.
- Use secure ORM wrappers for SeaORM queries.

❌ Invalid:
- Reference REST DTO types from domain/contract code.
- Add direct SeaORM query execution that bypasses secure wrappers.

---

**Source**: `rustfmt.toml`, `clippy.toml`, `dylint_lints/README.md`, `docs/MODKIT_UNIFIED_SYSTEM.md`, `.editorconfig`.  
**Last Updated**: 2026-02-05

