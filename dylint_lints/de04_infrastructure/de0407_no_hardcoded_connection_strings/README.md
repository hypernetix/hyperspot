# DE0407: No Hardcoded Connection Strings

### What it does

Detects hardcoded database and service connection strings in source code.

### Why is this bad?

Hardcoded connection strings are a security and operational risk:
- **Exposes credentials**: Connection strings often contain username/password
- **Version control exposure**: Secrets are committed to git history
- **Environment inflexibility**: Different environments need different connections
- **Requires redeployment**: Changes need code modifications and rebuilds

### Example

```rust
// ❌ Bad - hardcoded connection string
let db_url = "postgres://user:password@localhost:5432/mydb";
let cache = "redis://localhost:6379";
let mongo = "mongodb://admin:secret@db.example.com/myapp";
```

```rust
// ❌ Bad - hardcoded with credentials
const DATABASE_URL: &str = "postgresql://app:secretpass@prod.db.internal:5432/production";
```

Use instead:

```rust
// ✅ Good - load from environment
let db_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");
let cache_url = std::env::var("REDIS_URL")?;
```

```rust
// ✅ Good - load from configuration
#[derive(Deserialize)]
struct Config {
    database_url: String,
    redis_url: String,
}

let config = Config::from_env()?;
let db = Database::connect(&config.database_url).await?;
```

### Configuration

This lint is configured to **warn** by default.

It detects connection string URL schemes:
- `postgres://`, `postgresql://`, `mysql://`, `mariadb://`
- `mongodb://`, `redis://`, `sqlite://`

### See Also

- [DE0409](../de0409_no_secrets_in_migrations) - No Secrets in Migrations
- [DE0703](../../de07_security/de0703_no_hardcoded_secrets) - No Hardcoded Secrets
