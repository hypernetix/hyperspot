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
```toml
[dependencies]
clippy_utils = { git = "...", rev = "..." }
dylint_linting = "5.0.0"
lint_utils = { path = "../lint_utils" }

[dev-dependencies]
dylint_testing = "5.0.0"
# Add trait/macro crates needed for tests

[[example]]
name = "test_case_name"
path = "ui/test_case_name.rs"
```

### 3. Module Detection Pattern
```rust
impl EarlyLintPass for LintName {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        // Detect inline modules: mod contract { ... }
        if let ItemKind::Mod(_, ident, mod_kind) = &item.kind {
            if ident.name.as_str() == "contract" {
                if let rustc_ast::ModKind::Loaded(items, ..) = mod_kind {
                    for inner_item in items {
                        check_item_in_contract(cx, inner_item);
                    }
                }
                return;
            }
        }
        
        // Check structs/enums
        if !matches!(item.kind, ItemKind::Struct(..) | ItemKind::Enum(..)) {
            return;
        }

        // File-based module detection
        if !is_in_contract_module_ast(cx, item) {
            return;
        }
        
        check_derives(cx, item);
    }
}
```

### 4. Derive Attribute Checking
```rust
fn check_derives(cx: &EarlyContext<'_>, item: &Item) {
    for attr in &item.attrs {
        if !attr.has_name(rustc_span::symbol::sym::derive) {
            continue;
        }

        if let Some(meta_items) = attr.meta_item_list() {
            for meta_item in meta_items {
                if let Some(ident) = meta_item.ident() {
                    let derive_name = ident.name.as_str();
                    
                    if derive_name == "TargetTrait" {
                        cx.span_lint(LINT_NAME, attr.span, |diag| {
                            diag.primary_message("error message");
                            diag.help("helpful suggestion");
                        });
                    }
                }
            }
        }
    }
}
```

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

    #[derive(Debug, Clone, TargetTrait)]  // Should trigger lint
    pub struct Example {
        pub field: String,
    }
}

fn main() {}
```

### Generating .stderr Files
1. Run tests: `cargo test --lib ui_examples`
2. Copy normalized stderr from test output
3. Create `.stderr` file with `$DIR/` placeholder for paths
4. Line numbers must match exactly

### Example .stderr
```
error: contract type should not derive `TargetTrait` (DEXXX)
  --> $DIR/test_case.rs:4:5
   |
LL |     #[derive(Debug, Clone, TargetTrait)]
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
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
- [ ] Create UI test files in `ui/`
- [ ] Generate `.stderr` golden files
- [ ] Verify all tests pass: `cargo test --lib ui_examples`
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
