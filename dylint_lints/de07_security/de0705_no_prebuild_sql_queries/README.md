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
// ✅ Good - parameterized query with sqlx
sqlx::query("SELECT * FROM users WHERE id = $1")
    .bind(user_input)
    .fetch_one(&pool)
    .await?;
```

```rust
// ✅ Good - sea-orm with bind parameters
User::find()
    .filter(user::Column::Name.eq(name))
    .one(&db)
    .await?;
```

```rust
// ✅ Good - multiple parameters
sqlx::query("INSERT INTO posts (title, author_id) VALUES ($1, $2)")
    .bind(&title)
    .bind(author_id)
    .execute(&pool)
    .await?;
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
