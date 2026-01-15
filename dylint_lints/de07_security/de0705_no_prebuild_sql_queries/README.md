# DE0705: No Prebuilt SQL Queries

### What it does

Detects prebuilt SQL queries where strings are concatenated or formatted instead of using parameterized queries.

### Why is this bad?

Prebuilt SQL queries create security vulnerabilities:
- **SQL injection attacks**: Attackers can inject malicious SQL through user input
- **Data breaches**: Can lead to unauthorized data access or deletion
- **Authentication bypass**: Attackers can manipulate authentication queries
- **Industry standard**: Parameterized queries are the recommended practice

### Example

```rust
// ❌ Bad - prebuilt query with string formatting
let query = format!("SELECT * FROM users WHERE id = {}", user_input);
conn.execute(&query).await?;
```

```rust
// ❌ Bad - string concatenation with SQL
let query = "SELECT * FROM users WHERE name = '".to_string() + name + "'";
db.query(&query).await?;
```

```rust
// ❌ Bad - format! macro with SQL
let id = req.param("id");
let sql = format!("DELETE FROM posts WHERE id = {}", id);
conn.execute(&sql).await?;
```

Use instead:

```rust
// ✅ Good - sea-orm query builder with type-safe filters
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
use crate::infra::storage::entity::user::{Entity as UserEntity, Column};

let user = UserEntity::find()
    .filter(Column::Id.eq(user_id))
    .one(&conn)
    .await?;
```

```rust
// ✅ Good - SecureConn with access scope enforcement
use modkit_db::secure::SecureEntityExt;
use modkit_security::AccessScope;

let user = UserEntity::find()
    .filter(Column::Email.eq(email))
    .secure()                    // Enable security layer
    .scope_with(&scope)          // Apply access control
    .one(conn)
    .await
    .map_err(db_err)?;
```

```rust
// ✅ Good - parameterized insert with ActiveModel
use sea_orm::{ActiveModelTrait, Set};

let model = user::ActiveModel {
    id: Set(user.id),
    tenant_id: Set(user.tenant_id),
    email: Set(user.email.clone()),
    display_name: Set(user.display_name.clone()),
    created_at: Set(user.created_at),
    updated_at: Set(user.updated_at),
};

model.insert(conn).await.map_err(db_err)?;
```

### Configuration

This lint is configured to **warn** by default.

It detects:
- `format!()` macros with SQL keywords
- String concatenation (`+`) with SQL strings
- `.to_string()` on SQL queries (often followed by concatenation)

### See Also

- [DE0407](../../de04_infrastructure/de0407_no_hardcoded_connection_strings) - No Hardcoded Connection Strings
- [DE0703](../de0703_no_hardcoded_secrets) - No Hardcoded Secrets
- [DE0706](../de0706_no_direct_sqlx) - No Direct sqlx Usage
