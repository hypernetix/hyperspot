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
// ❌ Bad - hardcoded in code even without credentials
const DATABASE_URL: &str = "postgresql://localhost:5432/production";
let pool = PgPool::connect(DATABASE_URL).await?;
```

Use instead:

```rust
// ✅ Good - configuration struct with defaults
// File: src/config.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleConfig {
    #[serde(default = "default_audit_url")]
    pub audit_base_url: String,
    #[serde(default = "default_notifications_url")]
    pub notifications_base_url: String,
}

fn default_audit_url() -> String {
    "http://audit.local".to_owned()
}

fn default_notifications_url() -> String {
    "http://notifications.local".to_owned()
}
```

```rust
// ✅ Good - load config and database from ModuleCtx
// File: src/module.rs
use modkit::{Module, ModuleCtx, DatabaseCapability};
use url::Url;

impl Module for MyModule {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        // Load typed configuration
        let cfg: ModuleConfig = ctx.config()?;
        
        // Get database connection with security enforcement
        let db = ctx.db_required()?;
        let conn = db.sea_secure();  // SecureConn for all queries
        
        // Parse URLs from config
        let audit_url = Url::parse(&cfg.audit_base_url)?;
        let notify_url = Url::parse(&cfg.notifications_base_url)?;
        
        Ok(())
    }
}
```

### Configuration

This lint is configured to **warn** by default.

It detects connection string URL schemes:
- `postgres://`, `postgresql://`, `mysql://`, `mariadb://`
- `mongodb://`, `redis://`, `sqlite://`

### See Also

- [DE0409](../de0409_no_secrets_in_migrations) - No Secrets in Migrations
- [DE0703](../../de07_security/de0703_no_hardcoded_secrets) - No Hardcoded Secrets
