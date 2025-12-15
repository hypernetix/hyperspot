# UI Test Crate

A test crate that simulates a typical module layout with intentional lint violations to verify dylint lints work correctly.

## Structure

The `src/` directory mimics a real project's module structure:

```
src/
├── lib.rs              # Crate root
├── contract/           # Contract layer (DE01xx lints)
│   ├── de0101_test.rs  # Tests: No Serde in Contract
│   ├── de0102_test.rs  # Tests: No ToSchema in Contract
│   └── de0103_test.rs  # Tests: No HTTP Types in Contract
├── domain/             # Domain layer (DE02xx lints)
│   ├── de0201_test.rs  # Tests: DTOs Only in api/rest/
│   └── de0202_test.rs  # Tests: DTOs Not Referenced Outside API
└── api/rest/           # API layer (DE02xx, DE08xx lints)
    ├── dto.rs          # Shared DTOs for import tests
    ├── de0203_test.rs  # Tests: DTOs Must Have Serde
    ├── de0204_test.rs  # Tests: DTOs Must Have ToSchema
    └── de0801_test.rs  # Tests: API Endpoints Must Have Version
```

## Why This Structure?

The lints check for violations based on **file paths**:
- `contract/` - Contract layer rules apply
- `domain/` - Domain layer rules apply
- `api/rest/` - API layer rules apply

Standalone test files don't trigger path-based lints. This crate structure ensures lints see the correct module paths.

## Running Tests

```bash
# Run all UI tests
make dylint-test

# View test output for a specific lint
cat dylint_lints/contract_lints/ui/src/contract/de0101_test.rs

# Run lints on your actual project code
make dylint
```

## Test File Format

Each test file contains:
- `// Should trigger DExxxx` comments marking code that should violate the lint (must include the lint code)
- `// Should NOT trigger DExxxx` comments marking correct code (must include the lint code)
- Both violation patterns and correct patterns for comparison

**Important:** The `DExxxx` code is required in both `Should trigger` and `Should NOT trigger` comments for the test framework to properly match expected violations.

Example:

```rust
// Test DE0101: No Serde in Contract
use serde::{Deserialize, Serialize};

// Should trigger DE0101 - Serialize in contract layer
#[derive(Debug, Clone, Serialize)]
pub struct BadModel { }

// Should NOT trigger DE0101 - no serde (correct)
#[derive(Debug, Clone)]
pub struct GoodModel { }
```

## Adding New Tests

1. Add test file to appropriate directory (`contract/`, `domain/`, or `api/rest/`)
2. Add module declaration to `mod.rs`
3. Include `// Should trigger` comments for expected violations
4. Run `make dylint-test` to verify
