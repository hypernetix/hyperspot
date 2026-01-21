# Security Guidelines

**Source**: guidelines/SECURITY.md, docs/SECURE-ORM.md

## Input Validation

**Always validate user input**:
```rust
use validator::{Validate, ValidationError};

#[derive(Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(custom = "validate_password")]
    pub password: String,
}

fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8 {
        return Err(ValidationError::new("Password too short"));
    }
    Ok(())
}
```

## Secrets Management

**Critical Rules**:
- ❌ **Never commit secrets** to version control
- ✅ **Use environment variables** for configuration
- ✅ **Rotate secrets regularly**
- ✅ **Use secure random generation** for tokens

```rust
// ❌ BAD: hardcoded secret
const API_KEY: &str = "sk-1234567890abcdef";

// ✅ GOOD: environment variable
let api_key = std::env::var("API_KEY")
    .context("API_KEY environment variable not set")?;
```

## Secure ORM (Mandatory for Multi-Tenant Systems)

HyperSpot provides a **secure-by-default ORM layer** that enforces access control at **compile time** using the typestate pattern.

### Key Features

1. **Compile-time enforcement**: Cannot bypass scoping via type system
2. **Deny-by-default**: Empty scopes explicitly denied with `WHERE 1=0`
3. **Tenant isolation**: Automatic filtering when tenant_ids provided
4. **Request-scoped**: Security context tied to request lifecycle
5. **No SQL injection**: Uses SeaORM's parameterized queries
6. **Audit trail**: Raw access logs security warnings

### Architecture

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

**Key Principles**:
- **Request-scoped context**: `SecurityCtx` passed per-operation, not stored
- **Stateless services**: No security state in service layer
- **Explicit security**: Every operation requires explicit context
- **Safe by default**: Raw database access requires opt-in via `insecure-escape` feature

### Define Scopable Entity

```rust
use modkit_db_macros::Scopable;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "users")]
#[secure(tenant_col = "tenant_id")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
}
```

**Attributes**:
- `#[secure(tenant_col = "tenant_id")]` - Column for tenant isolation
- `#[secure(unrestricted)]` - For global system tables (no scoping)

### Usage in API Handler

```rust
pub async fn list_users_handler(
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbHandle>,
) -> Result<Json<Vec<User>>, Problem> {
    // Create security context from request
    let ctx = SecurityCtx::for_tenants(
        vec![auth.tenant_id], 
        auth.user_id
    );
    
    // Get secure connection
    let secure_conn = db.sea_secure();
    
    // Query with automatic tenant isolation
    let users = secure_conn
        .find::<user::Entity>(&ctx)?
        .all(secure_conn.conn())
        .await?;
    
    Ok(Json(users))
}
```

### Implicit Security Policy

| Scope Condition | SQL Result |
|----------------|------------|
| Empty (no tenant, no resource) | `WHERE 1=0` (deny all) |
| Tenants only | `WHERE tenant_col IN (...)` |
| Tenants only + entity has no tenant_col | `WHERE 1=0` (deny all) |
| Resources only | `WHERE resource_col IN (...)` |
| Both tenants and resources | `WHERE tenant_col IN (...) AND resource_col IN (...)` |

### SecurityCtx Creation

**For multi-tenant operations**:
```rust
let ctx = SecurityCtx::for_tenants(
    vec![tenant_id1, tenant_id2],
    user_id
);
```

**For resource-specific operations**:
```rust
let ctx = SecurityCtx::for_resources(
    vec![resource_id1, resource_id2],
    user_id
);
```

**For both**:
```rust
let ctx = SecurityCtx::new(
    vec![tenant_id],
    vec![resource_id],
    user_id
);
```

### SecureConn API

All methods require `&SecurityCtx` parameter:

```rust
// Find all (with scope)
let users = secure_conn
    .find::<user::Entity>(&ctx)?
    .all(secure_conn.conn())
    .await?;

// Find by ID (with scope)
let user = secure_conn
    .find_by_id::<user::Entity>(&ctx, user_id)?
    .one(secure_conn.conn())
    .await?;

// Update (with scope)
let result = secure_conn
    .update_many::<user::Entity>(&ctx)?
    .set(user::Column::Status.eq("active"))
    .exec(secure_conn.conn())
    .await?;

// Delete (with scope)
let result = secure_conn
    .delete_many::<user::Entity>(&ctx)?
    .exec(secure_conn.conn())
    .await?;

// Insert (with tenant validation)
let user = secure_conn
    .insert::<user::Entity>(&ctx, active_model)
    .await?;
```

### Global Entities (Unrestricted)

For system tables that don't have tenant isolation:

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "global_config")]
#[secure(unrestricted)]  // ← No scoping
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub key: String,
    pub value: String,
}
```

## Authentication & Authorization

**Authentication**:
- Use OAuth2/OIDC Bearer tokens
- Header: `Authorization: Bearer <token>`
- Never include secrets in URLs

**Authorization**:
- Check permissions per operation
- Use scopes/roles from token
- Return 403 Forbidden for insufficient permissions

## HTTPS & Transport Security

**Requirements**:
- HTTPS only (no HTTP in production)
- HSTS enabled
- TLS 1.2+ minimum
- Certificate validation

## CORS Configuration

```yaml
# config/server.yaml
modules:
  api_gateway:
    cors_enabled: true
    cors_allowed_origins:
      - "https://app.example.com"
      - "https://admin.example.com"
```

**Never use wildcards** (`*`) in production.

## SQL Injection Prevention

**Secure ORM automatically prevents SQL injection**:
- Uses SeaORM's parameterized queries
- Never builds raw SQL strings
- Compile-time query validation

**If raw SQL is absolutely required**:
```rust
// Requires insecure-escape feature flag
db.sqlx_pool().execute(
    sqlx::query("SELECT * FROM users WHERE id = $1")
        .bind(user_id)  // ← Parameterized
).await?;
```

## Rate Limiting

**Implement per endpoint**:
- Standard: 100 requests/hour
- Sensitive ops: Lower limits
- Return 429 with `Retry-After` header

## Audit Logging

**Log security events**:
```rust
tracing::warn!(
    user_id = %user_id,
    tenant_id = %tenant_id,
    action = "delete_user",
    target_id = %target_user_id,
    "User deleted another user"
);
```

**Log**:
- Authentication attempts
- Authorization failures
- Resource access (especially sensitive data)
- Configuration changes
- Security policy violations

## Password Requirements

**Minimum standards**:
- Length: ≥ 8 characters
- Complexity: Mix of upper/lower/numbers/symbols
- No common passwords (use dictionary check)
- Bcrypt/Argon2 for hashing

## Session Management

- Short token TTLs (e.g., 1 hour)
- Refresh tokens for long sessions
- Secure cookie flags: `HttpOnly`, `Secure`, `SameSite`
- Invalidate on logout

## Security Headers

**Required headers**:
```http
Strict-Transport-Security: max-age=31536000; includeSubDomains
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Content-Security-Policy: default-src 'self'
```

## Data Protection

**Sensitive data**:
- Encrypt at rest (database encryption)
- Encrypt in transit (TLS)
- Mask in logs (no passwords, tokens, PII)
- Secure deletion (overwrite, not just DELETE)

## Dependency Security

**Regular checks**:
```bash
cargo audit          # Security vulnerabilities
cargo deny check     # License & security policy
```

**CI integration**: Run on every PR.

## Best Practices

- ✅ **Always use Secure ORM** for multi-tenant data
- ✅ **Validate all inputs** with `validator` crate
- ✅ **Never hardcode secrets** (use env vars)
- ✅ **Use HTTPS only** in production
- ✅ **Implement rate limiting** per endpoint
- ✅ **Log security events** with structured logging
- ✅ **Audit dependencies** regularly
- ✅ **Use strong typing** to prevent security bugs
- ❌ **Never bypass Secure ORM** without explicit review
- ❌ **Never use raw SQL** without parameterization
- ❌ **Never log sensitive data** (passwords, tokens)
- ❌ **Never trust client input** (validate everything)

## Security Checklist

Before deployment:
- [ ] All database queries use Secure ORM or parameterized SQL
- [ ] Input validation on all API endpoints
- [ ] Authentication/authorization on protected endpoints
- [ ] HTTPS enforced, HSTS enabled
- [ ] CORS configured with explicit origins
- [ ] Rate limiting implemented
- [ ] Security headers configured
- [ ] Audit logging for sensitive operations
- [ ] Secrets managed via env vars (not hardcoded)
- [ ] Dependencies audited (`cargo audit`, `cargo deny`)
- [ ] No sensitive data in logs
- [ ] Password requirements enforced
- [ ] Session tokens have appropriate TTL

## Reference

- Complete Secure ORM docs: `docs/SECURE-ORM.md`
- Security guidelines: `guidelines/SECURITY.md`
- OAuth2/OIDC: Use `modkit-auth` library

---

## Validation Checklist

### MUST Requirements (Mandatory)

- [ ] **Input validation implemented** (validator crate)
- [ ] **Secrets in environment variables** (never hardcoded)
- [ ] **Secure ORM used** for multi-tenant data
- [ ] **SecurityCtx passed** to all database operations
- [ ] **Compile-time security enforcement** (typestate pattern)
- [ ] **HTTPS enforced** (no HTTP in production)
- [ ] **OAuth2/OIDC authentication** with Bearer tokens
- [ ] **Authorization checks** per operation
- [ ] **Rate limiting implemented**
- [ ] **Audit logging** for security events
- [ ] **Password requirements** enforced (≥8 chars, complexity)
- [ ] **Session management** secure (HttpOnly, Secure, SameSite)
- [ ] **Security headers** configured (HSTS, CSP, etc.)
- [ ] **Dependency audits** automated (`cargo audit`, `cargo deny`)

### SHOULD Requirements (Strongly Recommended)

- [ ] Secrets rotation process documented
- [ ] Multi-tenant isolation tested
- [ ] CORS allow-list configured
- [ ] SQL injection tests included
- [ ] Security event monitoring
- [ ] Penetration testing conducted

### MAY Requirements (Optional)

- [ ] WAF integration
- [ ] DDoS protection
- [ ] Anomaly detection
- [ ] Security metrics dashboard

## Compliance Criteria

**Pass**: All MUST requirements met (14/14) + security audit clean  
**Fail**: Any MUST requirement missing or security vulnerabilities found

### Agent Instructions

When implementing security:
1. ✅ **ALWAYS validate input** with validator crate
2. ✅ **ALWAYS use environment variables** for secrets
3. ✅ **ALWAYS use Secure ORM** for multi-tenant data
4. ✅ **ALWAYS pass SecurityCtx** to database operations
5. ✅ **ALWAYS enforce compile-time security** (typestate)
6. ✅ **ALWAYS use HTTPS** (no HTTP in production)
7. ✅ **ALWAYS authenticate** with OAuth2/OIDC
8. ✅ **ALWAYS authorize** before operations
9. ✅ **ALWAYS implement rate limiting**
10. ✅ **ALWAYS log security events**
11. ✅ **ALWAYS enforce password requirements**
12. ✅ **ALWAYS use secure session management**
13. ✅ **ALWAYS configure security headers**
14. ✅ **ALWAYS audit dependencies**
15. ❌ **NEVER bypass Secure ORM** (compile-time enforcement)
16. ❌ **NEVER use raw SQL** without parameterization
17. ❌ **NEVER hardcode secrets** (environment variables only)
18. ❌ **NEVER log sensitive data** (passwords, tokens, PII)
19. ❌ **NEVER trust client input** (validate everything)
20. ❌ **NEVER skip authorization checks**

### Security Implementation Checklist

Before deploying:
- [ ] All inputs validated with validator crate
- [ ] No secrets hardcoded (all in env vars)
- [ ] Secure ORM used for all database queries
- [ ] SecurityCtx passed to all SecureConn operations
- [ ] Compile-time security checks pass
- [ ] HTTPS enforced with HSTS
- [ ] OAuth2/OIDC authentication configured
- [ ] Authorization checks on all protected endpoints
- [ ] Rate limiting configured per endpoint
- [ ] Security events logged with trace_id
- [ ] Password requirements validated
- [ ] Sessions use HttpOnly/Secure/SameSite cookies
- [ ] Security headers configured
- [ ] cargo audit clean (no vulnerabilities)
- [ ] cargo deny check passes (licenses + advisories)
- [ ] No sensitive data in logs (tested)
- [ ] Tenant isolation tested
- [ ] SQL injection tests pass
- [ ] CORS configured with allow-list
- [ ] Penetration test completed (if required)
