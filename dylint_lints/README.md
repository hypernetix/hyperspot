# Hyperspot Dylint Linters

Custom [dylint](https://github.com/trailofbits/dylint) linters enforcing Hyperspot's architectural patterns, layer separation, and REST API conventions.

## Quick Start

```bash
# From workspace root
make dylint              # Run lints on workspace (auto-rebuilds if changed)
make dylint-list         # Show all available lints
make dylint-test         # Test UI cases (compile & verify violations)
```

## What This Checks

**Contract Layer (DE01xx)**
- ✅ DE0101: No Serde in Contract
- ✅ DE0102: No ToSchema in Contract
- ✅ DE0103: No HTTP Types in Contract

**API Layer (DE02xx)**
- ✅ DE0201: DTOs Only in API Rest Folder
- ✅ DE0202: DTOs Not Referenced Outside API
- ✅ DE0203: DTOs Must Have Serde Derives
- ✅ DE0204: DTOs Must Have ToSchema Derive

**REST Conventions (DE08xx)**
- ✅ DE0801: API Endpoint Must Have Version

## Examples

Each lint includes bad/good examples in source comments. View them:

```bash
# Show lint implementation with examples
cat contract_lints/src/de01_contract_layer/de0101_no_serde_in_contract.rs
```

Example output:

```rust
//! ## Example: Bad
//!
//! // src/contract/user.rs - WRONG
//! #[derive(Serialize, Deserialize)]  // ❌ Serde in contract
//! pub struct User { ... }
//!
//! ## Example: Good
//!
//! // src/contract/user.rs - CORRECT
//! #[derive(Debug, Clone)]  // ✅ No serde
//! pub struct User { ... }
//!
//! // src/api/rest/dto.rs - CORRECT
//! #[derive(Serialize, Deserialize)]  // ✅ Serde in DTO
//! pub struct UserDto { ... }
```

## Development

### Project Structure

```
dylint_lints/
├── contract_lints/           # Main lint crate
│   ├── src/
│   │   ├── de01_contract_layer/
│   │   ├── de02_api_layer/
│   │   ├── de08_rest_api_conventions/
│   │   ├── lib.rs            # Lint registration
│   │   └── utils.rs          # Helper functions
│   └── ui/                   # Test cases
│       ├── de0101_contract_serde.rs
│       ├── de0203_dto_serde_derives.rs
│       ├── de0801_api_versioning.rs
│       ├── good_contract.rs  # Correct patterns
│       └── ... (see ui/README.md)
├── Cargo.toml
├── rust-toolchain.toml       # Nightly required
└── README.md
```

### Adding a New Lint

1. Create file in appropriate category (e.g., `src/de02_api_layer/de0205_my_lint.rs`)

2. Implement the lint:

```rust
//! DE0205: My Lint Description
//!
//! ## Example: Bad
//! // ... bad code example
//!
//! ## Example: Good
//! // ... good code example

use rustc_hir::{Item, ItemKind};
use rustc_lint::{LateContext, LintContext};

rustc_session::declare_lint! {
    pub MY_LINT,
    Deny,
    "description of what this checks"
}

pub fn check<'tcx>(cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
    // Implementation
}
```

3. Register in `lib.rs`:

```rust
mod de02_api_layer {
    pub mod de0205_my_lint;
}

impl<'tcx> LateLintPass<'tcx> for ContractLints {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        de02_api_layer::de0205_my_lint::check(cx, item);
    }
}
```

4. Add test case in `ui/` directory (optional but recommended):

```rust
// ui/de0205_my_lint.rs
mod api {
    // Should trigger - violation example
    pub struct BadPattern { }

    // Should NOT trigger - correct pattern
    pub struct GoodPattern { }
}
fn main() {}
```

5. Test:

```bash
make dylint       # Run on workspace code
make dylint-test  # List test cases - compare with your violations
```

### Useful Patterns

**Check if in specific module:**

```rust
use crate::utils::is_in_api_rest_folder;

if !is_in_api_rest_folder(cx, item.owner_id.def_id) {
    return;
}
```

**Check derives:**

```rust
let attrs = cx.tcx.hir_attrs(item.hir_id());
for attr in attrs {
    if attr.has_name(Symbol::intern("derive")) {
        // Check derive attributes
    }
}
```

**Lint with help:**

```rust
cx.span_lint(MY_LINT, item.span, |diag| {
    diag.primary_message("Error message");
    diag.help("Suggestion on how to fix");
});
```

## Troubleshooting

**"dylint library not found"**
```bash
cd dylint_lints && cargo build --release
```

**"feature may not be used on stable"**
Dylint requires nightly. The `rust-toolchain.toml` in `dylint_lints/` sets this automatically.

**Lint not triggering**
- Check file path matches pattern (e.g., `*/api/rest/*`)
- Verify lint is registered in `lib.rs`
- Rebuild: `cd dylint_lints && cargo build --release`

**Changes not reflected**
Use `make dylint` - it auto-rebuilds if sources changed.

## Resources

- [LINTS.md](./LINTS.md) - Complete catalog with examples
- [Makefile](../Makefile) - Tool comparison table (line 60)
- [Dylint Docs](https://github.com/trailofbits/dylint)
- [Clippy Lint Development](https://doc.rust-lang.org/nightly/clippy/development/index.html)

## License

Apache-2.0
