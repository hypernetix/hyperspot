# Code Conventions Specification

**Source**: CONTRIBUTING.md, guidelines/DNA/, dylint_lints/

## File Naming

**Rust Files**: `snake_case.rs`  
**Modules**: `snake_case/`  
**Tests**: `{feature}_tests.rs` or `tests/{feature}.rs`

## Code Style

**Formatter**: `rustfmt` (Rust standard)  
**Command**: `cargo fmt --all`  
**CI Check**: `cargo fmt --all -- --check`

**Linter**: `clippy`  
**Command**: `cargo clippy --workspace --all-targets --all-features -- -D warnings`  
**CI Check**: Same command (warnings denied)

**Custom Lints**: `dylint`  
**Command**: `python scripts/ci.py dylint` or `make dylint`  
**Location**: `dylint_lints/` (project-specific architectural rules)

**📋 Detailed Linting Requirements**: See `specs/linting.md` for comprehensive linting specification (MANDATORY for all code validation)

## Naming Conventions

**Types**: `PascalCase` (struct, enum, trait)  
**Functions/Variables**: `snake_case`  
**Constants**: `SCREAMING_SNAKE_CASE`  
**Modules**: `snake_case`

**Examples**:
```rust
pub struct ModuleInfo { ... }           // Type
pub fn get_module_info() -> ... { }     // Function
const MAX_RETRIES: u32 = 3;             // Constant
mod api_gateway { ... }                 // Module
```

## Module Organization

```
modules/{module}/
├── src/
│   ├── lib.rs                  # Public exports
│   ├── module.rs               # Module implementation
│   ├── contract/mod.rs         # Public contracts
│   ├── api/rest/routes.rs      # REST endpoints
│   ├── domain/                 # Business logic
│   └── infra/                  # Infrastructure
```

## Documentation

**Doc Comments**: Use `///` for public items  
**Module Docs**: Use `//!` at top of files  
**Examples**: Include examples in doc comments when helpful

```rust
/// Gets information about the parser.
///
/// # Example
/// ```
/// let info = get_parser_info().await?;
/// ```
pub async fn get_parser_info() -> Result<Info> { ... }
```

## Error Handling

**Use Result<T, E>** throughout  
**Never panic** in library code  
**Domain Errors**: Custom error types per module  
**API Errors**: Convert to RFC 7807 Problem Details

## Testing Conventions

**Unit Tests**: `#[cfg(test)] mod tests { ... }` in same file  
**Integration Tests**: `tests/` directory  
**Test Function Names**: `test_{what}_when_{condition}_then_{expected}`

**Example**:
```rust
#[test]
fn test_parse_when_valid_pdf_then_success() {
    // Test implementation
}
```

## Import Organization

**Group imports**:
1. Standard library (`std::...`)
2. External crates
3. Internal crates (`modkit::...`)
4. Local modules (`crate::...`, `super::...`)

```rust
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use modkit::Context;

use crate::contract::FileInfo;
```

## Guidelines

**Rust Guidelines**: `guidelines/DNA/languages/RUST.md`  
**API Guidelines**: `guidelines/DNA/REST/API.md`  
**Security**: `guidelines/SECURITY.md`  
**New Modules**: `guidelines/NEW_MODULE.md`

## CI Requirements

All code must pass:
- ✅ `cargo fmt --check` (formatting)
- ✅ `cargo clippy` (linting)
- ✅ `python scripts/ci.py dylint` (custom lints)
- ✅ `cargo test` (tests)
- ✅ `cargo deny check` (license/security)

---

## Validation Checklist

### MUST Requirements (Mandatory)

- [ ] **File naming follows snake_case** for Rust files
- [ ] **Type names use PascalCase** (structs, enums, traits)
- [ ] **Function/variable names use snake_case**
- [ ] **Constants use SCREAMING_SNAKE_CASE**
- [ ] **Code formatted with rustfmt** (no manual styling)
- [ ] **Clippy warnings resolved** (deny warnings in CI)
- [ ] **Custom dylint lints pass** (architectural compliance)
- [ ] **Result<T, E> used** (no panics in library code)
- [ ] **Doc comments for public items** (`///`)
- [ ] **Imports organized** (std → external → internal → local)

### SHOULD Requirements (Strongly Recommended)

- [ ] Module structure follows canonical layout
- [ ] Test names follow `test_{what}_when_{condition}_then_{expected}`
- [ ] Examples included in doc comments
- [ ] Domain errors custom per module
- [ ] Integration tests in `tests/` directory

### MAY Requirements (Optional)

- [ ] Additional inline documentation
- [ ] Performance notes in comments
- [ ] TODO comments for future improvements

## Compliance Criteria

**Pass**: All MUST requirements met (10/10)  
**Fail**: Any MUST requirement missing or CI check failing

### Agent Instructions

When writing code:
1. ✅ **ALWAYS run rustfmt** before committing
2. ✅ **ALWAYS resolve clippy warnings** (zero warnings policy)
3. ✅ **ALWAYS pass dylint** (architectural enforcement)
4. ✅ **ALWAYS use snake_case** for files, functions, variables
5. ✅ **ALWAYS use PascalCase** for types
6. ✅ **ALWAYS document public APIs** with `///`
7. ✅ **ALWAYS use Result<T, E>** (no unwrap/expect in lib code)
8. ✅ **ALWAYS organize imports** (std → external → internal → local)
9. ❌ **NEVER commit unformatted code**
10. ❌ **NEVER ignore clippy warnings**
11. ❌ **NEVER panic in library code**
12. ❌ **NEVER mix naming conventions**

### Code Review Checklist

Before submitting code:
- [ ] Ran `cargo fmt --all`
- [ ] Ran `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] Ran `python scripts/ci.py dylint` or `make dylint`
- [ ] All tests pass (`cargo test`)
- [ ] Public APIs documented
- [ ] No unwrap/expect in production code
- [ ] Imports organized correctly
