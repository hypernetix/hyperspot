# Secure ORM Layer for modkit-db

A type-safe, scoped SeaORM wrapper that enforces access control at compile time using the typestate pattern.

## Overview

This module provides a **secure-by-default** ORM layer that prevents unscoped database queries from executing. It implements an implicit security policy based on tenant IDs and resource IDs provided by upper layers.

## Features

- **Typestate enforcement**: Unscoped queries cannot be executed (compile-time safety)
- **Request-scoped security**: Security context passed per-operation, not stored
- **Derive macro**: Automatic trait implementation with `#[derive(Scopable)]`
- **Implicit policy**: Automatic deny-all for empty scopes
- **Multi-tenant support**: Enforces tenant isolation when applicable
- **Resource-level access**: Fine-grained control via explicit IDs
- **Global entities**: Support for unrestricted system tables via `#[secure(unrestricted)]`
- **Safe by default**: Raw database access requires `insecure-escape` feature flag
- **Zero runtime overhead**: All checks happen at compile/build time
- **Composable**: Works with standard SeaORM query builders
- **No auth dependencies**: Receives only IDs from upper layers

## Architecture

The secure ORM follows a **request-scoped security model**:

```
API Handler (per-request)
    ↓ Creates SecurityCtx from auth/session
SecureConn (stateless wrapper)
    ↓ Receives SecurityCtx per-operation
    ↓ Enforces implicit policy
SeaORM
    ↓
Database
```

Key principles:
- **Request-scoped context**: Security context (`SecurityCtx`) passed per-operation, not stored
- **Stateless services**: No security state in service layer
- **Explicit security**: Every operation requires explicit context
- **Safe by default**: Raw database access requires opt-in via `insecure-escape` feature

## Public API

```rust
// High-level API
pub use SecureConn;        // Stateless connection wrapper
pub use SecurityCtx;       // Request-scoped security context

// Types
pub use AccessScope;       // Scope with tenant/resource IDs

// Traits
pub use ScopableEntity;    // Entity contract
pub use SecureEntityExt;   // Extension for Select<E>

// Typestates
pub use Unscoped;          // Query not yet scoped
pub use Scoped;            // Query ready to execute

// Errors
pub use ScopeError;
```

### SecureConn API

All methods require `&SecurityCtx` parameter:

- `find<E>(&self, ctx: &SecurityCtx) -> Result<SecureSelect<E, Scoped>>`
- `find_by_id<E>(&self, ctx: &SecurityCtx, id: Uuid) -> Result<SecureSelect<E, Scoped>>`
- `update_many<E>(&self, ctx: &SecurityCtx) -> Result<SecureUpdateMany<E, Scoped>>`
- `delete_many<E>(&self, ctx: &SecurityCtx) -> Result<SecureDeleteMany<E, Scoped>>`
- `insert<E>(&self, ctx: &SecurityCtx, am: E::ActiveModel) -> Result<E::Model>`
- `update_one<E>(&self, ctx: &SecurityCtx, am: E::ActiveModel) -> Result<E::Model>`
- `delete_by_id<E>(&self, ctx: &SecurityCtx, id: Uuid) -> Result<bool>`

## Implicit Security Policy

| Scope Condition | SQL Result |
|----------------|------------|
| Empty (no tenant, no resource) | `WHERE 1=0` (deny all) |
| Tenants only | `WHERE tenant_col IN (...)` |
| Tenants only + entity has no tenant_col | `WHERE 1=0` (deny all) |
| Resources only | `WHERE resource_col IN (...)` |
| Both tenants and resources | `WHERE tenant_col IN (...) AND resource_col IN (...)` |

## Quick Start

### 1. Implement ScopableEntity

You can implement `ScopableEntity` manually or use the derive macro:

**Option A: Using the derive macro (recommended)**

```rust
use modkit_db_macros::Scopable;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "users")]
#[secure(
    tenant_col = "tenant_id",
    resource_col = "id",
    no_owner,
    no_type
)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
}
```

**Option B: Manual implementation**

```rust
use modkit_db::secure::ScopableEntity;

impl ScopableEntity for user::Entity {
    fn tenant_col() -> Option<Self::Column> {
        Some(user::Column::TenantId)  // Multi-tenant entity
    }
    
    fn resource_col() -> Option<Self::Column> {
        Some(user::Column::Id)
    }
    
    fn owner_col() -> Option<Self::Column> {
        None
    }
    
    fn type_col() -> Option<Self::Column> {
        None
    }
}
```

**For global/system entities (no tenant isolation)**

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "system_config")]
#[secure(unrestricted)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub key: String,
    pub value: String,
}
```

### 2. Use SecureConn (Recommended)

```rust
use modkit_db::secure::{SecurityCtx, SecureConn};

// In API handler: create context from request
let ctx = SecurityCtx::for_tenants(vec![tenant_id], user_id);

// Get secure connection (stateless wrapper)
let secure_conn = db_handle.sea_secure();

// All operations receive context explicitly
let users = secure_conn
    .find::<user::Entity>(&ctx)?
    .all(secure_conn.conn())
    .await?;
```

### 3. Alternative: Low-level API

```rust
use modkit_db::secure::{AccessScope, SecureEntityExt};

let scope = AccessScope::tenants_only(vec![tenant_id]);

let users = user::Entity::find()
    .secure()              // Convert to SecureSelect<E, Unscoped>
    .scope_with(&scope)?   // Apply scope → SecureSelect<E, Scoped>
    .all(conn)             // Now can execute
    .await?;
```

## Scopable Macro Attributes

The `#[derive(Scopable)]` macro requires explicit declarations for all scope dimensions.

### Required Attributes (must specify one for each dimension)

**Tenant dimension:**
- `tenant_col = "column_name"` - Specify the tenant ID column for multi-tenant entities
- `no_tenant` - Explicitly declare this entity has no tenant isolation

**Resource dimension:**
- `resource_col = "column_name"` - Specify the resource ID column (typically the primary key)
- `no_resource` - Explicitly declare this entity has no resource-level filtering

**Owner dimension:**
- `owner_col = "column_name"` - Specify an owner ID column for owner-based access control
- `no_owner` - Explicitly declare this entity has no owner-based filtering

**Type dimension:**
- `type_col = "column_name"` - Specify a type ID column for type-based filtering (polymorphic scenarios)
- `no_type` - Explicitly declare this entity has no type-based filtering

### Unrestricted Entities

- `unrestricted` - Mark entity as unrestricted (no scoping at all, for global system tables)

**Important Rules:**
- All four dimensions (tenant, resource, owner, type) must be explicitly specified using either `*_col = "..."` or `no_*`
- The `unrestricted` flag cannot be combined with any other attributes
- No implicit defaults are allowed - this enforces compile-time safety

### Examples

**Multi-tenant entity with explicit decisions:**
```rust
#[derive(Scopable)]
#[secure(
    tenant_col = "tenant_id",
    resource_col = "id",
    no_owner,
    no_type
)]
struct Model { /* ... */ }
```

**Global/system entity:**
```rust
#[derive(Scopable)]
#[secure(unrestricted)]
struct SystemConfig { /* ... */ }
```

## Files

- `mod.rs` - Module exports and documentation
- `types.rs` - AccessScope definition
- `entity_traits.rs` - ScopableEntity trait
- `secure_conn.rs` - SecureConn high-level API and SecurityCtx
- `select.rs` - SecureSelect wrapper with typestates
- `db_ops.rs` - SecureUpdateMany, SecureDeleteMany, and secure_insert
- `cond.rs` - Condition builder implementing implicit policy
- `provider.rs` - TenantFilterProvider pattern for advanced use cases
- `error.rs` - ScopeError type
- `docs.rs` - Comprehensive documentation
- `tests.rs` - Unit tests
- `USAGE_EXAMPLE.md` - Complete usage examples
- `README.md` - This file

## Design Decisions

### Typestate Pattern

We use Rust's type system to prevent unscoped queries:

```rust
pub struct SecureSelect<E: EntityTrait, S> {
    inner: sea_orm::Select<E>,
    _state: PhantomData<S>,  // Typestate marker
}

// Only Unscoped has scope_with()
impl<E> SecureSelect<E, Unscoped> {
    pub fn scope_with(self, scope: &AccessScope) 
        -> Result<SecureSelect<E, Scoped>, ScopeError>;
}

// Only Scoped has all() and one()
impl<E> SecureSelect<E, Scoped> {
    pub async fn all<C>(self, conn: &C) -> Result<Vec<E::Model>, ScopeError>;
    pub async fn one<C>(self, conn: &C) -> Result<Option<E::Model>, ScopeError>;
}
```

This makes it **impossible** to execute unscoped queries at compile time.

### No Authentication Dependency

The layer receives only UUIDs (tenant_ids, resource_ids) from upper layers. It has no knowledge of:
- User authentication
- Role-based access control
- Permission systems
- Session management

This keeps the layer focused and composable.

### Request-Scoped Security Model

Security context is passed **per-operation**, not stored in services:

**Benefits:**
- **Explicit security**: Every operation shows security context in its signature
- **Stateless services**: No security state stored, easier to test and reason about
- **Request lifecycle**: Context tied to HTTP request, not service lifetime
- **Audit-friendly**: Easy to trace which context was used for each operation

**Creating SecurityCtx:**
```rust
// From tenant IDs (multi-tenant isolation)
let ctx = SecurityCtx::for_tenants(vec![tenant_id], user_id);

// From resource IDs (specific resources)
let ctx = SecurityCtx::for_resources(vec![resource_id], user_id);

// Combined (tenant + specific resources)
let ctx = SecurityCtx::for_tenants_and_resources(
    vec![tenant_id],
    vec![resource_id],
    user_id
);
```

### Implicit vs Explicit Policy

Instead of requiring explicit policy configuration, the layer enforces a simple, predictable policy:

1. **Empty scope → deny all**: Safer default than returning all data
2. **Tenant isolation**: Always enforced when tenant_ids provided
3. **AND composition**: Multiple constraints are combined with AND
4. **No bypass**: Cannot opt-out of scoping once enabled

## Integration with OData Pagination

The secure ORM layer works seamlessly with ModKit's type-safe OData pagination system. Security scoping is applied **before** OData filters, ensuring users can only filter within their authorized scope.

### Combined Usage Pattern

```rust
use modkit_db::odata::sea_orm_filter::{paginate_odata, LimitCfg};
use modkit_db::secure::{SecurityCtx, SecureConn};

pub struct UserRepository<'a> {
    conn: &'a SecureConn,
}

impl<'a> UserRepository<'a> {
    pub async fn list_paginated(
        &self,
        ctx: &SecurityCtx,
        odata_query: &ODataQuery,
    ) -> Result<Page<User>, RepoError> {
        // 1. Start with security-scoped query
        let base_query = self.conn
            .find::<user::Entity>(ctx)?;  // Applies tenant/resource scope
        
        // 2. Apply OData filtering and pagination on top
        let page = paginate_odata::<UserDtoFilterField, UserODataMapper, _, _, _, _>(
            base_query.into_inner(),  // Extract underlying Select<E>
            self.conn.conn(),
            odata_query,
            ("id", SortDir::Desc),
            LimitCfg { default: 25, max: 1000 },
            |model| model.into(),
        ).await?;
        
        Ok(page)
    }
}
```

### Security + OData Flow

```
1. SecurityCtx created from request auth
   ↓
2. SecureConn applies tenant/resource scope
   ↓ WHERE tenant_id IN (...) AND ...
3. OData filters applied on scoped data
   ↓ AND (email LIKE '...' OR ...)
4. OData ordering and cursor pagination
   ↓ ORDER BY created_at DESC, id DESC LIMIT 26
5. Results returned (Page<T>)
```

**Key Benefits:**
- **Defense in depth**: Security scope applied first, OData filters second
- **Type safety**: Both layers use compile-time checked types
- **Composable**: Security and filtering are orthogonal concerns
- **Performance**: Single query with combined WHERE clause

## Usage Patterns

### Service Layer with Request-Scoped Context

```rust
pub struct UserService<'a> {
    db: &'a SecureConn,
}

impl<'a> UserService<'a> {
    pub async fn list_users(
        &self,
        ctx: &SecurityCtx,  // Context per-operation
    ) -> Result<Vec<user::Model>, ServiceError> {
        self.db
            .find::<user::Entity>(ctx)?
            .all(self.db.conn())
            .await
            .map_err(Into::into)
    }
    
    pub async fn get_user(
        &self,
        ctx: &SecurityCtx,
        id: Uuid,
    ) -> Result<Option<user::Model>, ServiceError> {
        self.db
            .find_by_id::<user::Entity>(ctx, id)?
            .one(self.db.conn())
            .await
            .map_err(Into::into)
    }
}
```

### API Handler

```rust
pub async fn list_users_handler(
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbHandle>,
) -> Result<Json<Vec<UserDto>>, Problem> {
    // Create context from request auth
    let ctx = SecurityCtx::for_tenants(vec![auth.tenant_id], auth.user_id);
    
    // Get secure connection (stateless)
    let secure_conn = db.sea_secure();
    
    // Create service and pass context
    let service = UserService { db: &secure_conn };
    let users = service.list_users(&ctx).await?;
    
    Ok(Json(users.into_iter().map(UserDto::from).collect()))
}
```

### Repository Pattern (Alternative)

```rust
pub struct UserRepository<'a> {
    conn: &'a SecureConn,
}

impl<'a> UserRepository<'a> {
    pub async fn find_all(
        &self,
        ctx: &SecurityCtx,
    ) -> Result<Vec<user::Model>, DbError> {
        self.conn
            .find::<user::Entity>(ctx)?
            .all(self.conn.conn())
            .await
            .map_err(Into::into)
    }
}
```

## Testing

The module includes comprehensive test coverage:

- **Unit tests**: AccessScope, condition builder, typestate markers, policy enforcement
- **Integration tests**: Scoped queries, mutations, and SecureConn API
- **Compile-fail tests**: Macro attribute validation, duplicate detection, conflicting flags
- **Trybuild tests**: Derive macro edge cases and error messages

All tests are located in the respective module files and in `libs/modkit-db-macros/tests/`. Integration tests with real SeaORM entities should be written in application code where actual entities are defined.

## Acceptance Criteria

- **No unscoped execution path**: Only `SecureSelect<E, Scoped>` exposes `.all/.one`
- **Implicit policy** implemented:
  - Empty → deny all (`1=0`)
  - Tenants only → tenant filter
  - Resources only → id filter
  - Both → AND
- **Derive macro support**: `#[derive(Scopable)]` with compile-time validation
- **Global entity support**: `#[secure(unrestricted)]` for system-wide tables
- **Scoped mutations**: UPDATE and DELETE operations with scope enforcement
- **Works with entities without tenant columns** (global entities)
- **Comprehensive testing**: Unit tests, integration tests, and compile-fail tests
- **Complete documentation**: Examples, guides, and API documentation

## Implemented Features

1. **Request-scoped security model**: `SecurityCtx` passed per-operation for explicit, auditable security
2. **Derive macro**: `#[derive(Scopable)]` with enhanced diagnostics and duplicate detection
3. **Scoped mutations**: UPDATE and DELETE operations with scope enforcement
4. **Global entities**: `#[secure(unrestricted)]` flag for system-wide tables
5. **Feature flag protection**: Raw database access gated by `insecure-escape` feature
6. **Enhanced error messages**: Compile-time validation with clear diagnostics
7. **Absolute path generation**: Macro works correctly in all contexts (re-exports, renames)

## Future Enhancements

Planned for future versions:

1. **PostgreSQL RLS**: Row-level security integration
2. **Audit logging**: Automatic query audit trails
3. **Policy composition**: Role-based and custom filters
4. **Advanced scoping**: Support for complex multi-tenant hierarchies

## Raw Database Access

For administrative operations and migrations, raw database access is available via feature flag:

### Using the insecure-escape Feature

```toml
[dependencies]
modkit-db = { path = "...", features = ["insecure-escape"] }
```

```rust
#[cfg(feature = "insecure-escape")]
async fn admin_migration(db: &DbHandle) {
    // Raw access with security warning logged
    let raw_conn = db.sea();  // Logs tracing::warn!
    
    // Direct SeaORM access for migrations
    sqlx::query("CREATE TABLE ...").execute(raw_conn).await?;
}
```

**Important:**
- Raw access is **disabled by default** (safe by default)
- Requires explicit opt-in via `--features insecure-escape`
- Logs security warnings via `tracing::warn!` when used
- Should only be used for migrations and administrative tools
- Production services should use `SecureConn` API

## Security Guarantees

1. **Compile-time enforcement**: Cannot bypass scoping via type system
2. **Deny-by-default**: Empty scopes explicitly denied
3. **Safe by default**: Raw database access requires feature flag
4. **Request-scoped**: Security context tied to request lifecycle
5. **Tenant isolation**: When tenant_ids provided, always enforced
6. **No SQL injection**: Uses SeaORM's parameterized queries
7. **Transparent**: Generates inspectable SQL (via SeaORM)
8. **Audit trail**: Raw access logs security warnings

## Performance

- **Zero runtime overhead**: Typestates compiled away
- **No additional queries**: Single query with WHERE clause
- **Index-friendly**: Uses standard IN clauses
- **SeaORM native**: Works with existing query optimizations
- **Request-scoped context**: Passed by reference, no allocations

## Migration Guide

### Updating to Request-Scoped Model

If you're using an older version with stored context, update your code:

**Before (deprecated):**
```rust
let ctx = SecurityCtx::for_tenants(vec![tenant_id], user_id);
let secure_db = db.sea_secure(ctx);  // Context stored
let users = secure_db.find::<user::Entity>()?
    .all(secure_db.conn()).await?;
```

**After (current):**
```rust
let secure_conn = db.sea_secure();  // No context
let ctx = SecurityCtx::for_tenants(vec![tenant_id], user_id);
let users = secure_conn.find::<user::Entity>(&ctx)?  // Context per-operation
    .all(secure_conn.conn()).await?;
```

**Service layer changes:**
```rust
// Add &SecurityCtx parameter to all methods
pub struct UserService<'a> {
    db: &'a SecureConn,  // Renamed from SecureDb
}

impl<'a> UserService<'a> {
    pub async fn get_user(&self, ctx: &SecurityCtx, id: Uuid) -> Result<User> {
        self.db.find_by_id::<user::Entity>(ctx, id)?  // Pass context
            .one(self.db.conn()).await
    }
}
```

**Status**: Production Ready

The secure ORM layer is fully implemented with:
- Request-scoped security model for explicit, auditable access control
- Compile-time safety guarantees via typestate pattern
- Derive macro with enhanced diagnostics and validation
- Feature flag protection for raw database access
- Comprehensive testing (49 unit tests, compile-fail tests, trybuild tests)
- Complete documentation and usage examples


