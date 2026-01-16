# Linting Specification

**Source**: Makefile, scripts/ci.py, .clippy.toml, rustfmt.toml, dylint.toml, dylint_lints/

## Overview

Linting is **mandatory** for all code submissions. Three types of lints enforce code quality and architectural compliance:

1. **rustfmt** - Code formatting
2. **clippy** - Rust best practices and idioms
3. **dylint** - Custom architectural rules

**All lints MUST pass** before code can be committed or merged.

---

## 1. rustfmt - Code Formatting

**Purpose**: Enforce consistent code style across the codebase.

**Configuration**: `rustfmt.toml` in project root

**Run Command**:
```bash
cargo fmt --all
```

**Check Command** (CI):
```bash
cargo fmt --all --check
```

**Rules**:
- Standard Rust formatting conventions
- Line length limits
- Import organization
- Indentation (4 spaces)
- Trailing commas

**Integration**:
- Pre-commit: Run `cargo fmt --all`
- CI: Fails if formatting violations detected
- IDE: Enable format-on-save

---

## 2. clippy - Rust Linter

**Purpose**: Catch common mistakes and enforce Rust best practices.

**Configuration**: `.clippy.toml`, `libs/.clippy.toml`

**Run Command**:
```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

**Deny Warnings**: All clippy warnings treated as errors in CI (`-D warnings`)

**Categories Checked**:
- Correctness (bugs)
- Suspicious patterns
- Performance issues
- Style violations
- Complexity
- Pedantic checks

**Common Violations**:
- Unused variables/imports
- Unnecessary allocations
- Non-idiomatic code
- Type complexity
- Missing error handling

**Allowed Lints** (rare exceptions):
```rust
#[allow(clippy::too_many_arguments)] // Only when unavoidable
```

**Integration**:
- Pre-commit: Run clippy
- CI: Fails on any warning
- IDE: Enable clippy integration

---

## 3. dylint - Custom Architectural Lints

**Purpose**: Enforce project-specific architectural and design rules.

**Location**: `dylint_lints/` directory

**Configuration**: `dylint.toml` in project root

**Run Command**:
```bash
make dylint
```

**Or**:
```bash
python scripts/ci.py dylint
```

**Lint Categories**:

### DE01xx: Contract Layer
- **DE0101**: No serde in contracts (transport-agnostic)
- **DE0102**: No ToSchema in contracts
- **DE0103**: No HTTP types in contracts

### DE02xx: API Layer
- **DE0201**: DTOs only in api/rest/
- **DE0202**: DTOs not referenced outside API layer
- **DE0203**: DTOs must have Serialize/Deserialize
- **DE0204**: DTOs must have ToSchema

### DE05xx: Client Layer
- **DE0503**: Plugin clients must have "_client" suffix

### DE08xx: REST Conventions
- **DE0801**: Endpoints must be versioned (/v1, /v2)
- **DE0802**: OData extension methods required

### DE09xx: Documentation
- **DE0901**: Public items must have doc comments

**Violation Example**:
```rust
// ❌ BAD: serde in contract (DE0101)
#[derive(Serialize)]
pub struct UserInfo { ... }

// ✅ GOOD: transport-agnostic contract
pub struct UserInfo { ... }
```

**Integration**:
- Pre-commit: Run `make dylint`
- CI: Fails on any violation
- Development: Run before push

---

## Linting Workflow

### Before Committing

**Step 1**: Format code
```bash
cargo fmt --all
```

**Step 2**: Run clippy
```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

**Step 3**: Run custom lints
```bash
make dylint
```

**Or run all at once**:
```bash
python scripts/ci.py all
```

### CI Pipeline

CI **automatically fails** if any lint check fails:

1. **Format check**: `cargo fmt --all --check`
2. **Clippy check**: `cargo clippy ... -- -D warnings`
3. **Custom lints**: `python scripts/ci.py dylint`

**No exceptions** - all checks must pass.

---

## IDE Integration

### VS Code

**Extensions**:
- rust-analyzer (auto-formatting, clippy integration)
- Even Better TOML

**Settings** (`.vscode/settings.json`):
```json
{
  "editor.formatOnSave": true,
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": ["--all-targets", "--all-features"]
}
```

### Other IDEs

Configure to:
- Run rustfmt on save
- Show clippy warnings inline
- Run lints before commit

---

## Troubleshooting

### rustfmt Issues

**Problem**: Formatting conflicts
**Solution**: Use `cargo fmt --all` (not individual files)

**Problem**: Custom formatting ignored
**Solution**: Check `rustfmt.toml` configuration

### clippy Issues

**Problem**: False positives
**Solution**: Use `#[allow(clippy::...)]` with justification comment

**Problem**: Too many warnings
**Solution**: Fix incrementally, never disable globally

### dylint Issues

**Problem**: Lint not found
**Solution**: 
```bash
cargo clean
make dylint
```

**Problem**: Lint fails to build
**Solution**: Check Rust version matches `rust-toolchain.toml`

**Problem**: Unclear violation
**Solution**: Check lint documentation in `dylint_lints/{category}/{lint}/`

---

## Validation Checklist

### MUST Requirements (Mandatory)

- [ ] **rustfmt configured** (rustfmt.toml present)
- [ ] **clippy configured** (.clippy.toml present)
- [ ] **dylint configured** (dylint.toml present)
- [ ] **Format command documented** (`cargo fmt --all`)
- [ ] **Clippy command documented** (with -D warnings)
- [ ] **Dylint command documented** (`make dylint`)
- [ ] **CI integration configured** (all lints in pipeline)
- [ ] **Zero warnings policy enforced** (clippy, dylint)
- [ ] **Pre-commit workflow documented**
- [ ] **IDE integration instructions provided**

### SHOULD Requirements (Strongly Recommended)

- [ ] Format-on-save enabled in IDE
- [ ] Clippy integration in IDE
- [ ] Pre-commit hooks configured
- [ ] Lint documentation accessible
- [ ] Custom lint examples provided

### MAY Requirements (Optional)

- [ ] Additional clippy lints enabled
- [ ] Custom lint metrics
- [ ] Lint performance monitoring

## Compliance Criteria

**Pass**: All MUST requirements met (10/10) + all lints pass (rustfmt, clippy, dylint)
**Fail**: Any MUST requirement missing OR any lint violation present

### Agent Instructions

When writing code:
1. ✅ **ALWAYS run rustfmt** before committing (`cargo fmt --all`)
2. ✅ **ALWAYS run clippy** and fix all warnings
3. ✅ **ALWAYS run dylint** before push (`make dylint`)
4. ✅ **ALWAYS resolve all lint violations** (never ignore)
5. ✅ **ALWAYS check CI passes** all lint checks
6. ✅ **ALWAYS use -D warnings** for clippy (deny warnings)
7. ✅ **ALWAYS run all lints together** (`python scripts/ci.py all`)
8. ✅ **ALWAYS document allowed lints** (rare exceptions only)
9. ❌ **NEVER commit unformatted code**
10. ❌ **NEVER ignore clippy warnings** without justification
11. ❌ **NEVER bypass dylint checks**
12. ❌ **NEVER disable lints globally** (workspace level)
13. ❌ **NEVER commit with lint violations**
14. ❌ **NEVER use #[allow(...)]** for architectural lints (dylint)

### Pre-Commit Linting Checklist

**Before every commit**:
- [ ] Ran `cargo fmt --all` (code formatted)
- [ ] Ran `cargo clippy --workspace --all-targets --all-features -- -D warnings` (no warnings)
- [ ] Ran `make dylint` or `python scripts/ci.py dylint` (no violations)
- [ ] All tests pass (`cargo test`)
- [ ] No compilation warnings
- [ ] IDE shows no lint errors
- [ ] Ready for CI (will pass all checks)

### CI Validation Checklist

**CI must verify**:
- [ ] Format check passes (`cargo fmt --all --check`)
- [ ] Clippy check passes (zero warnings)
- [ ] Custom lints pass (dylint)
- [ ] No compilation warnings
- [ ] All tests pass

---

## Reference

- **rustfmt config**: `rustfmt.toml`
- **clippy config**: `.clippy.toml`, `libs/.clippy.toml`
- **dylint config**: `dylint.toml`
- **Custom lints**: `dylint_lints/` directory
- **CI script**: `scripts/ci.py`
- **Makefile**: `Makefile` (dylint target)
- **Architectural lints spec**: `specs/architectural-lints.md`
- **Conventions spec**: `specs/conventions.md`

---

## Summary

**Linting is non-negotiable**:
- 🔴 **rustfmt** - Code MUST be formatted
- 🔴 **clippy** - Warnings MUST be fixed
- 🔴 **dylint** - Architecture MUST comply

**Zero tolerance**: No code with lint violations can be committed or merged.

**Commands to remember**:
```bash
# Format
cargo fmt --all

# Lint
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Architecture
make dylint

# All checks
python scripts/ci.py all
```

**Golden rule**: If CI fails on lints, fix the code, don't fix the lints.
