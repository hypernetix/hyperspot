# OData Macro Consolidation Migration Guide

## Overview

All OData-related procedural macros have been consolidated into a single crate: **`modkit-odata-macros`**.

## What Changed

### Before
- `ODataSchema` was in `modkit-sdk-macros`
- `ODataFilterable` was in `modkit-db-macros`
- Confusion about which crate owns OData contract generation
- Database crate (`modkit-db-macros`) contained protocol-level macros

### After
- **All** OData macros are now in `modkit-odata-macros`
  - `ODataSchema` - Client-side query builder schema generation
  - `ODataFilterable` - Server-side filter field enum generation
- Clear separation: Protocol macros in `modkit-odata-macros`, DB macros in `modkit-db-macros`
- `modkit-sdk-macros` **removed** (no longer needed)
- `modkit-db-macros` now contains only `Scopable` (SeaORM security)

## Architecture

```
modkit-odata-macros/
├── ODataSchema       → Generates Schema trait for client query building
└── ODataFilterable   → Generates FilterField enum for server filtering

modkit-sdk-macros/    [REMOVED]
modkit-db-macros/
└── Scopable          → SeaORM security scoping (non-OData)
```

## Migration Steps

### For Server-Side Code (API/Service/Infra)

**Old:**
```toml
# Cargo.toml
[dependencies]
modkit-db-macros = { path = "../modkit-db-macros" }
```

```rust
// src/query/mod.rs
use modkit_db_macros::ODataFilterable;

#[derive(ODataFilterable)]
pub struct UserQuery {
    #[odata(filter(kind = "Uuid"))]
    pub id: Uuid,
}
```

**New:**
```toml
# Cargo.toml
[dependencies]
modkit-db-macros = { path = "../modkit-db-macros" }        # Still needed for Scopable
modkit-odata-macros = { path = "../modkit-odata-macros" }  # NEW: For ODataFilterable
```

```rust
// src/query/mod.rs
use modkit_odata_macros::ODataFilterable;  // Changed import

#[derive(ODataFilterable)]
pub struct UserQuery {
    #[odata(filter(kind = "Uuid"))]
    pub id: Uuid,
}
```

### For Client-Side Code (SDK)

**Old:**
```toml
# Cargo.toml
[dependencies]
modkit-sdk = { path = "../modkit-sdk", features = ["derive"] }
```

```rust
use modkit_sdk::ODataSchema;  // Re-exported from modkit-sdk

#[derive(ODataSchema)]
struct User {
    id: uuid::Uuid,
    email: String,
}
```

**New:**
```toml
# Cargo.toml
[dependencies]
modkit-sdk = { path = "../modkit-sdk", features = ["derive"] }
# modkit-sdk now re-exports from modkit-odata-macros internally
```

```rust
use modkit_sdk::ODataSchema;  // Still works! Now re-exports from modkit-odata-macros

#[derive(ODataSchema)]
struct User {
    id: uuid::Uuid,
    email: String,
}
```

**OR** use directly:
```rust
use modkit_odata_macros::ODataSchema;  // Direct import also works

#[derive(ODataSchema)]
struct User {
    id: uuid::Uuid,
    email: String,
}
```

## Breaking Changes

### 1. `modkit-sdk-macros` is now empty

**Impact:** If you imported directly from `modkit-sdk-macros`, you **must** update.

```rust
// ❌ OLD - Will not compile
use modkit_sdk_macros::ODataSchema;

// ✅ NEW - Use modkit-odata-macros
use modkit_odata_macros::ODataSchema;

// ✅ OR - Use modkit-sdk re-export
use modkit_sdk::ODataSchema;
```

### 2. `ODataFilterable` moved out of `modkit-db-macros`

**Impact:** Must add `modkit-odata-macros` dependency and update imports.

```rust
// ❌ OLD - Will not compile
use modkit_db_macros::ODataFilterable;

// ✅ NEW
use modkit_odata_macros::ODataFilterable;
```

## Rationale

### Problem
1. **Layering violation**: `modkit-db-macros` contained protocol-level macros, not DB-specific
2. **Confusion**: OData macros scattered across `modkit-db-macros` and `modkit-sdk-macros`
3. **Wrong dependencies**: After moving filter DSL to `modkit-odata`, macros in `modkit-db-macros` referenced `modkit-odata` types, creating circular conceptual dependencies

### Solution
1. **Single source of truth**: All OData protocol macros in `modkit-odata-macros`
2. **Clear naming**: "odata-macros" explicitly indicates OData protocol
3. **Correct layering**: DB macros (`Scopable`) stay in `modkit-db-macros`, OData macros move to `modkit-odata-macros`
4. **No DB pollution**: OData macros don't pull in SeaORM or database dependencies

## Verification

After migration, verify with:
```bash
cargo check -p modkit-odata-macros
cargo check -p modkit-sdk
cargo check -p modkit-db-macros
cargo check -p your-service
```

## FAQ

**Q: Do I need to change my derive attributes?**  
A: No, `#[derive(ODataFilterable)]` and `#[derive(ODataSchema)]` work exactly the same way.

**Q: Can I still use `modkit-sdk::ODataSchema`?**  
A: Yes! `modkit-sdk` now re-exports from `modkit-odata-macros`, so existing code continues to work.

**Q: What happened to `modkit-sdk-macros`?**  
A: It has been completely removed. All OData macros are now in `modkit-odata-macros`.

**Q: Does this affect runtime behavior?**  
A: No, this is purely a compile-time/packaging change. Generated code is identical.

**Q: Do tests need to change?**  
A: Only test imports. Test assertions and behavior remain unchanged.

## Checklist

- [ ] Update `Cargo.toml` to add `modkit-odata-macros` dependency
- [ ] Change `use modkit_db_macros::ODataFilterable` to `use modkit_odata_macros::ODataFilterable`
- [ ] Change `use modkit_sdk_macros::*` to `use modkit_odata_macros::*` (if importing directly)
- [ ] Run `cargo check` to verify compilation
- [ ] Run tests to verify functionality
- [ ] Update any documentation references

## References

- Filter DSL migration: See earlier migration that moved filter types from `modkit-db` to `modkit-odata`
- Macro crate source: `libs/modkit-odata-macros/`
- Generated code inspection: `cargo expand --package your-crate`
