# DE0706: No Direct sqlx Usage

### What it does

Prohibits direct usage of the `sqlx` crate in the codebase.

### Why is this bad?

Direct sqlx usage bypasses important architectural layers:
- **Skips security enforcement**: SecureConn and AccessScope are not applied
- **Bypasses query building**: Loses type-safe query construction
- **Inconsistent patterns**: Makes codebase harder to maintain
- **No audit logging**: Loses automatic operation tracking
- **No tenant isolation**: Multi-tenant security controls are bypassed

### Example

```rust
// ❌ Bad - direct sqlx usage
use sqlx::PgPool;
use sqlx::query;

let pool = PgPool::connect(&db_url).await?;
let users = sqlx::query("SELECT * FROM users")
    .fetch_all(&pool)
    .await?;
```

```rust
// ❌ Bad - sqlx query macros
use sqlx::query_as;

let user = query_as!(User, "SELECT * FROM users WHERE id = $1", id)
    .fetch_one(&pool)
    .await?;
```

Use instead:

```rust
// ✅ Good - sea-orm with type-safe queries
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
use crate::infra::storage::entity::user::{Entity as UserEntity, Column};

let users = UserEntity::find()
    .all(&conn)
    .await?;

let user = UserEntity::find()
    .filter(Column::Id.eq(id))
    .one(&conn)
    .await?;
```

```rust
// ✅ Good - SecureConn with access scope
use modkit_db::secure::SecureEntityExt;
use modkit_security::AccessScope;

let users = UserEntity::find()
    .secure()                    // Enable security layer
    .scope_with(&scope)          // Apply tenant/access control
    .all(conn)
    .await
    .map_err(db_err)?;
```

### Configuration

This lint is configured to **deny** by default.

It detects:
- `use sqlx::*` imports
- `use sqlx::{...}` nested imports
- `extern crate sqlx` declarations

### See Also

- [DE0705](../de0705_no_prebuild_sql_queries) - No Prebuilt SQL Queries
- [DE0407](../../de04_infrastructure/de0407_no_hardcoded_connection_strings) - No Hardcoded Connection Strings
