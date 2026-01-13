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
// ✅ Good - load from environment
let api_key = std::env::var("STRIPE_API_KEY")
    .expect("STRIPE_API_KEY must be set");
let client = StripeClient::new(&api_key);
```

```rust
// ✅ Good - load from secure config
let config = Config::from_env()?;
let connection = connect_db(&config.db_url, &config.db_password);
```

```rust
// ✅ Good - use secret management service
let secrets = SecretManager::new().await?;
let jwt_secret = secrets.get("jwt_secret").await?;
let token = encode(&claims, jwt_secret.as_bytes())?;
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
