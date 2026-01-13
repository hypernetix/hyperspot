# DE0409: No Secrets in Migrations

### What it does

Detects hardcoded secrets (passwords, API keys, tokens) in database migration files.

### Why is this bad?

Migration files are often committed to version control and executed in different environments:
- **Version control exposure**: Credentials are permanently in git history
- **Environment inflexibility**: Cannot use different secrets per environment (dev/staging/prod)
- **Security vulnerability**: Public repositories expose credentials to everyone
- **Audit trail**: Secret rotation requires tracking down all migration files

### Example

```rust
// ❌ Bad - hardcoded password in migration
// File: migrations/20240101_add_admin.rs
fn up(conn: &Connection) {
    conn.execute(
        "INSERT INTO users (name, password) VALUES ('admin', 'secret123')"
    );
}
```

```rust
// ❌ Bad - API key in seed data
// File: migrations/20240115_add_integrations.rs
fn up(conn: &Connection) {
    conn.execute(
        "INSERT INTO api_keys (service, key) VALUES ('stripe', 'sk_live_abc123')"
    );
}
```

Use instead:

```rust
// ✅ Good - use environment variables
// File: migrations/20240101_add_admin.rs
fn up(conn: &Connection) {
    let password = std::env::var("ADMIN_PASSWORD")
        .expect("ADMIN_PASSWORD must be set");
    conn.execute(
        "INSERT INTO users (name, password_hash) VALUES ($1, $2)",
        &["admin", &hash_password(&password)]
    )?;
}
```

```rust
// ✅ Good - use placeholder values
// File: migrations/20240115_add_integrations.rs
fn up(conn: &Connection) {
    // Create schema only, secrets configured via admin panel
    conn.execute(
        "INSERT INTO api_keys (service, key) VALUES ('stripe', 'PLACEHOLDER')"
    );
}
```

### Configuration

This lint is configured to **warn** by default.

It checks files in `migrations/` directories for patterns indicating secrets:
- Password-like strings (`password`, `secret`, `token`)
- API key patterns (`sk_`, `pk_`, authentication tokens)

### See Also

- [DE0407](../de0407_no_hardcoded_connection_strings) - No Hardcoded Connection Strings
- [DE0703](../../de07_security/de0703_no_hardcoded_secrets) - No Hardcoded Secrets
