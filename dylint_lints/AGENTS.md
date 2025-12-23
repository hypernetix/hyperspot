# Agent Guide: Adding Dylint Lints

## Quick Start

1. **Initialize**: `cargo dylint new <lint_name>` in `dylint_lints/`
2. **Configure**: Update `Cargo.toml` with dependencies and example targets
3. **Implement**: Write lint logic in `src/lib.rs`
4. **Test**: Create UI test files in `ui/` with corresponding `.stderr` files. If the `main.rs` and `main.stderr` are empty, remove them.
5. **Register**: Add to workspace in `dylint_lints/Cargo.toml`

## Lint Pass Selection

### Pre-Expansion Lint (`declare_pre_expansion_lint!`)
**Use when**: Checking derive attributes before macro expansion

**Characteristics**:
- Runs before proc macros expand
- Uses `EarlyLintPass` with AST (`rustc_ast`)
- Can see `#[derive(...)]` attributes directly
- Required for detecting serde/utoipa derives

**Example**: `de0101_no_serde_in_contract`, `de0102_no_toschema_in_contract`

```rust
dylint_linting::declare_pre_expansion_lint! {
    pub LINT_NAME,
    Deny,
    "description"
}

impl EarlyLintPass for LintName {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        // Check derive attributes before macro expansion
    }
}
```

### Early Lint Pass (`declare_early_lint!`)
**Use when**: Checking syntax/structure before type checking

**Characteristics**:
- Runs after macro expansion but before type resolution
- Uses `EarlyLintPass` with AST (`rustc_ast`)
- No type information available
- Fast, syntax-level checks

**Example**: Naming conventions, syntax patterns

### Late Lint Pass (`declare_late_lint!`)
**Use when**: Need type information or semantic analysis

**Characteristics**:
- Runs after type checking
- Uses `LateLintPass` with HIR (`rustc_hir`)
- Full type information available
- Can check trait implementations, method calls, etc.

**Example**: Type-based checks, semantic validation

## Implementation Pattern (Pre-Expansion)

### 1. Crate Structure
```
de0xxx_lint_name/
├── Cargo.toml          # Dependencies + example targets
├── src/lib.rs          # Lint implementation
└── ui/                 # UI tests
    ├── test1.rs
    ├── test1.stderr
    ├── test2.rs
    └── test2.stderr
```

### 2. Cargo.toml Configuration
Common dependencies (`clippy_utils`, `dylint_linting`, `dylint_testing`, `lint_utils`) are defined in the workspace `Cargo.toml`. Reference them using `.workspace = true`:

```toml
[dependencies]
clippy_utils.workspace = true
dylint_linting.workspace = true
lint_utils.workspace = true

[dev-dependencies]
dylint_testing.workspace = true
# Add trait/macro crates needed for tests

[[example]]
name = "test_case_name"
path = "ui/test_case_name.rs"
```

**Note**: Only add lint-specific dependencies to individual `Cargo.toml` files. Keep common dependencies in the workspace to avoid duplication.

## Testing Options

### ui_examples vs ui_test vs ui_test_example

**Use `ui_test_examples`** (Recommended):
- Tests all example targets defined in `Cargo.toml`
- Each example is a separate test case
- Examples live in `ui/` directory
- Best for multiple independent test scenarios
- Used by: `de0101`, `de0102`

```rust
#[test]
fn ui_examples() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
```

**Use `ui_test`**:
- Tests all `.rs` files in a directory
- No need for `[[example]]` targets in `Cargo.toml`
- Files share dependencies from `[dev-dependencies]`
- Good for many small test cases

```rust
#[test]
fn ui() {
    dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui");
}
```

**Use `ui_test_example`**:
- Tests a single specific example target
- Useful for focused testing during development
- Can be combined with `ui_test_examples`

```rust
#[test]
fn specific_case() {
    dylint_testing::ui_test_example(env!("CARGO_PKG_NAME"), "example_name");
}
```

**Choose `ui_test_examples` when**:
- Your tests need external dependencies (e.g., serde, utoipa)
- You want explicit test case organization
- Test cases are logically distinct scenarios

**Choose `ui_test` when**:
- All tests have no external dependencies
- You have many small, similar test cases
- You want simpler `Cargo.toml` configuration

## UI Testing

### Test File Structure
```rust
mod contract {
    use target_crate::TargetTrait;

    #[derive(Debug, Clone, TargetTrait)]
    // Should trigger DEXXX - description of what triggers
    pub struct Example {
        pub field: String,
    }
}

fn main() {}
```

### Comment Annotations for Test Validation

**Purpose**: Validate that test comments match actual lint behavior in `.stderr` files.

**Comment Format**:
- `// Should trigger DEXXX - description` - Marks code that MUST trigger the lint
- `// Should not trigger DEXXX - description` - Marks code that MUST NOT trigger the lint

**Placement Rules**:
- Place comment on the line **immediately before** where the error is reported
- For multiline spans (structs, enums, functions), the error is reported on the **first line** of the item
- NOT on the attribute line (e.g., `#[derive(...)]`), but on the item declaration line

**Example - Correct**:
```rust
#[derive(Debug, Clone)]
// Should trigger DE0203 - DTOs must have serde derives
pub struct UserDto {  // Error reported HERE
    pub id: String,
}
```

**Example - Incorrect**:
```rust
// Should trigger DE0203 - DTOs must have serde derives
#[derive(Debug, Clone)]  // Comment expects error on next line (derive)
pub struct UserDto {     // But error is actually reported HERE
    pub id: String,
}
```

**Required Unit Test**:
Every lint MUST include this test to enforce comment/stderr alignment:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn ui_examples() {
        dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
    }

    #[test]
    fn test_comment_annotations_match_stderr() {
        let ui_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui");
        lint_utils::test_comment_annotations_match_stderr(
            &ui_dir,
            "DEXXX",  // Lint code
            "description of what triggers"  // Must match comment text
        );
    }
}
```

This test validates:
1. Every "Should trigger" comment has a corresponding error in `.stderr`
2. Every "Should not trigger" comment has NO error in `.stderr`
3. Every error in `.stderr` has a corresponding "Should trigger" comment

### Generating .stderr Files
1. Run tests: `cargo test --lib ui_examples`
2. Copy normalized stderr from test output
3. Create `.stderr` file with `$DIR/` placeholder for paths
4. Line numbers must match exactly
5. Add comment annotations as described above

### Example .stderr
```
error: contract type should not derive `TargetTrait` (DEXXX)
  --> $DIR/test_case.rs:5:5
   |
LL | / pub struct Example {
LL | |     pub field: String,
LL | | }
   | |_^
   |
   = help: helpful suggestion here
   = note: `#[deny(lint_name)]` on by default

error: aborting due to 1 previous error

```

## Shared Utilities

### lint_utils Crate
- `is_in_contract_module_ast()`: Check if AST item is in contract/ directory
- Add new helpers as needed for common patterns

## Checklist

- [ ] Run `cargo dylint new <name>` 
- [ ] Update `Cargo.toml` with dependencies
- [ ] Add example targets for each test case
- [ ] Implement lint with appropriate pass type
- [ ] Create UI test files in `ui/` with comment annotations
- [ ] Generate `.stderr` golden files
- [ ] Add `test_comment_annotations_match_stderr` unit test
- [ ] Verify all tests pass: `cargo test --lib`
- [ ] Add to workspace `members` in root `Cargo.toml`
- [ ] Document lint behavior in doc comments

## Common Pitfalls

1. **Wrong lint pass**: Pre-expansion for derives, late for types
2. **Module detection**: Must handle both `mod contract {}` and `contract/` directories
3. **Line numbers**: `.stderr` files must match exact line numbers including `#[allow(dead_code)]`
4. **Empty tests**: Include test case with no violations (empty `.stderr`)
5. **Workspace**: Don't forget to add new crate to workspace members
6. **Test verification**: Always verify correct package tests are running with `-p` flag
7. **simulated_dir**: Only works with EarlyLintPass, not LateLintPass
