# DE0703: No Hardcoded Secrets

### What it does

Detects hardcoded secrets such as passwords, API keys, tokens, and private keys in source code.

### Why is this bad?

Hardcoded secrets create serious security vulnerabilities:

- **Version control exposure**: Secrets are permanently in git history
- **Public repository risk**: Anyone can access credentials if repository is public
- **Difficult rotation**: Changing secrets requires code changes and redeployment
- **Compliance violations**: Fails security audits and compliance requirements
- **Credential theft**: Attackers can easily extract secrets from binaries

### Example

```rust
// ❌ Bad - hardcoded API key
const API_KEY: &str = "sk_live_abc123def456";
let client = StripeClient::new(API_KEY);
```

```rust
// ❌ Bad - hardcoded password
let password = "MySecretPassword123!";
let connection = connect_db("postgres://localhost", password);
```

```rust
// ❌ Bad - hardcoded JWT secret
const JWT_SECRET: &str = "my-super-secret-jwt-key-2024";
let token = encode(&claims, JWT_SECRET.as_bytes())?;
```

Use instead:

```rust
// ✅ Good - configuration struct with serde
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

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            audit_base_url: default_audit_url(),
            notifications_base_url: default_notifications_url(),
        }
    }
}

fn default_audit_url() -> String {
    "http://audit.local".to_owned()
}

fn default_notifications_url() -> String {
    "http://notifications.local".to_owned()
}
```

```rust
// ✅ Good - load configuration from ModuleCtx
// File: src/module.rs
use modkit::{Module, ModuleCtx, TracedClient};
use url::Url;

impl Module for MyModule {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        // Load typed configuration from YAML/environment
        let cfg: ModuleConfig = ctx.config()?;
        
        // Parse service URLs from config
        let audit_url = Url::parse(&cfg.audit_base_url)
            .map_err(|e| anyhow::anyhow!("invalid audit_base_url: {e}"))?;
        let notify_url = Url::parse(&cfg.notifications_base_url)
            .map_err(|e| anyhow::anyhow!("invalid notifications_base_url: {e}"))?;
        
        // Create traced HTTP client
        let traced_client = TracedClient::default();
        
        // Create adapter with injected dependencies
        let audit_adapter = HttpAuditClient::new(traced_client, audit_url, notify_url);
        
        Ok(())
    }
}
```

### Configuration

This lint is configured to **warn** by default.

It detects patterns indicating secrets:
- API keys: `sk_`, `pk_`, `api_key`, `apikey`
- Passwords: `password`, `passwd`, `pwd`
- Tokens: `token`, `auth`, `secret`, `key`
- Private keys: `private_key`, `-----BEGIN`

### See Also

- [DE0407](../../de04_infrastructure/de0407_no_hardcoded_connection_strings) - No Hardcoded Connection Strings
- [DE0409](../../de04_infrastructure/de0409_no_secrets_in_migrations) - No Secrets in Migrations
