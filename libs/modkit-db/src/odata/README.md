# OData + Secure ORM Integration

This module provides a minimal, ergonomic fluent builder (`OPager`) that combines Secure ORM scoping with OData pagination.

## Overview

- **No new dependencies**: Uses existing `modkit-odata`, `sea_orm`, and secure ORM types
- **No macros**: Simple, explicit builder pattern
- **No facades**: Works directly with existing `SecureConn`, `SecurityCtx`, `ODataQuery`, etc.
- **Type-safe**: Leverages Rust's type system for compile-time correctness
- **Zero overhead**: Thin wrapper over existing pagination logic

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        OPager Builder                       │
│  (Combines security scope + OData pagination)              │
└─────────────────────────────────────────────────────────────┘
                              │
                    ┌─────────┴─────────┐
                    │                   │
           ┌────────▼────────┐  ┌──────▼────────┐
           │  SecureConn     │  │  paginate_    │
           │  (Security      │  │  with_odata   │
           │   scoping)      │  │  (OData       │
           └────────┬────────┘  │   pagination) │
                    │           └───────────────┘
                    │
           ┌────────▼────────┐
           │  SecurityCtx    │
           │  (Tenant/       │
           │   resource      │
           │   boundaries)   │
           └─────────────────┘
```

## Files

- **`pager.rs`**: The `OPager` fluent builder implementation
- **`core.rs`**: Core OData → SeaORM translation (filters, cursors, ordering)
- **`mod.rs`**: Module exports and documentation
- **`tests.rs`**: Unit tests (currently disabled, needs refactoring)

## Usage

### Basic Example

```rust
use modkit_db::odata::{FieldMap, FieldKind, pager::OPager};
use modkit_db::secure::{SecureConn, SecurityCtx};
use modkit_odata::{ODataQuery, SortDir};

// Define field mappings once
static USER_FMAP: Lazy<FieldMap<user::Entity>> = Lazy::new(|| {
    FieldMap::new()
        .insert("id", user::Column::Id, FieldKind::Uuid)
        .insert("name", user::Column::Name, FieldKind::String)
        .insert("email", user::Column::Email, FieldKind::String)
});

// Use in your service
async fn list_users(
    db: &SecureConn,
    ctx: &SecurityCtx,
    query: &ODataQuery,
) -> Result<Page<UserDto>, ODataError> {
    OPager::<user::Entity, _>::new(db, ctx, db.conn(), &USER_FMAP)
        .tiebreaker("id", SortDir::Desc)
        .limits(25, 1000)
        .fetch(query, |m| UserDto::from(m))
        .await
}
```

### Features

1. **Security-first**: Automatically applies tenant/resource scoping before any filters
2. **OData integration**: Supports filters, ordering, cursors, and limits from OData queries
3. **Ergonomic API**: Fluent builder with sensible defaults
4. **Type-safe**: Generic over entity and connection types
5. **Performance**: Cursor-based pagination, limit+1 fetching, database-level filtering

### Defaults

- **Tiebreaker**: `("id", SortDir::Desc)` - Ensures stable, deterministic pagination
- **Limits**: `{ default: 25, max: 1000 }` - Reasonable defaults for most APIs

## Implementation Details

### Security Flow

1. `OPager::new()` receives `SecureConn` and `SecurityCtx`
2. `fetch()` calls `SecureConn::find::<E>(&ctx)` to create a scoped `SecureSelect`
3. `into_inner()` extracts the raw `sea_orm::Select<E>` (already scoped)
4. `paginate_with_odata()` applies OData filters, cursor, and ordering
5. Query executes with both security scope AND OData constraints

### OData Flow

1. Parse filter (done by caller, we receive `ODataQuery`)
2. Apply security scope (from `SecurityCtx`)
3. Apply OData filter (if present)
4. Apply cursor predicate (for pagination)
5. Apply ordering (with tiebreaker)
6. Fetch limit+1 rows
7. Trim and build next/prev cursors
8. Map models to DTOs

## Migration Guide

### Before (Manual Approach)

```rust
async fn list_users(
    db: &SecureConn,
    ctx: &SecurityCtx,
    query: &ODataQuery,
) -> Result<Page<User>, ODataError> {
    // 1. Create scoped select
    let select = db.find::<user::Entity>(ctx)?
        .into_inner();
    
    // 2. Manually call paginate_with_odata
    paginate_with_odata::<user::Entity, User, _, _>(
        select,
        db.conn(),
        query,
        &USER_FMAP,
        ("id", SortDir::Desc),
        LimitCfg { default: 25, max: 1000 },
        |m| m.into(),
    ).await
}
```

### After (With OPager)

```rust
async fn list_users(
    db: &SecureConn,
    ctx: &SecurityCtx,
    query: &ODataQuery,
) -> Result<Page<User>, ODataError> {
    OPager::<user::Entity, _>::new(db, ctx, db.conn(), &USER_FMAP)
        .fetch(query, |m| m.into())
        .await
}
```

## Public API

### Exports

From `modkit_db::odata::pager`:
- `OPager<'a, E, C>` - The fluent builder struct

From `modkit_db::odata` (re-exported from core):
- `FieldMap<E>` - OData field → entity column mappings
- `FieldKind` - Supported field types (String, I64, F64, Bool, Uuid, etc.)
- `LimitCfg` - Pagination limit configuration
- `paginate_with_odata()` - Core pagination function
- All other OData helper functions and types

## Testing

Run tests:
```bash
cargo test -p modkit-db --features sea-orm,sqlite
```

Build library:
```bash
cargo build -p modkit-db --features sea-orm,sqlite
```

## Future Enhancements

- [ ] Fix `tests.rs` mock entities after module refactoring
- [ ] Add integration tests with real database
- [ ] Consider adding `OPager::new_with_defaults()` for common cases
- [ ] Add metrics/tracing support for pagination performance
- [ ] Consider caching field maps at compile time

## Acceptance Criteria

New builder compiles and is fully generic over `E` and `C`  
No new dependencies, no macros, no repo traits  
Does not change any existing behavior of OData or Secure ORM  
Default behavior: tiebreaker ("id", Desc), limits { default: 25, max: 1000 }  
Builder returns `modkit_odata::Page<D>` and `modkit_odata::Error` on failure  
Public API surface is small and discoverable  
Entire workspace builds successfully  

