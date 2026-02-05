# Dependencies Policy (Hyperspot)

**Version**: 1.0  
**Purpose**: Keep dependencies consistent, secure, and aligned with project guidelines.  
**Scope**: All Rust crates in this workspace.  

## Workspace dependencies

Prefer adding deps to root `Cargo.toml` under `[workspace.dependencies]` and using `workspace = true` from member crates where appropriate.

## Preferred / disallowed choices

- YAML: use `serde-saphyr` (not `serde_yaml`).
- Avoid unnecessary new crates; check if a ModKit/lib crate already provides the functionality.

## Security & licensing

- `cargo-deny` runs in CI; licenses are allowlisted and RustSec advisories are checked.
- If a dependency triggers an advisory, prefer upgrading/replacing; only add ignore entries with a clear justification.

## Validation Criteria

- [ ] New deps are added at workspace level when they’re shared.
- [ ] `serde-saphyr` is used for YAML (no `serde_yaml`).
- [ ] License and advisory checks remain green (or justified exceptions are documented).
- [ ] Dependency additions include a brief rationale (PR description / ADR if significant).

## Examples

✅ Valid:
- Add `serde-saphyr` at workspace and use it in crates that parse YAML.
- Add a targeted dependency only in one crate when it’s truly local.

❌ Invalid:
- Add duplicate versions of the same crate across multiple member crates without reason.
- Introduce `serde_yaml` contrary to project guidelines.

---

**Source**: `guidelines/DEPENDENCIES.md`, `Cargo.toml` (`[workspace.dependencies]`), `deny.toml`.  
**Last Updated**: 2026-02-05

