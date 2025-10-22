# modkit-auth

JWT-based authentication and authorization library for HyperSpot modules.

## Features

- **JWT Token Validation** - JWKS-based validation with OIDC support
- **Role-Based Authorization** - Resource:action pattern matching
- **Scope Building** - Convert JWT claims to SecurityCtx
- **Axum Integration** - Drop-in extractors for handlers
- **Disable Mode** - Full bypass for development environments

## Architecture

```
┌─────────────┐
│ JWT Token   │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│ TokenValidator  │  ─── Validates JWT via JWKS
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Claims          │  ─── Parsed JWT claims (sub, tenants, roles)
└────────┬────────┘
         │
         ├──────────────────┐
         │                  │
         ▼                  ▼
┌──────────────────┐  ┌──────────────────┐
│ ScopeBuilder     │  │ PrimaryAuthorizer│
│                  │  │                  │
│ tenants → scope  │  │ roles check      │
└────────┬─────────┘  └────────┬─────────┘
         │                     │
         ▼                     ▼
┌──────────────────────────────────────┐
│ SecurityCtx (scope + subject)         │
└──────────────────────────────────────┘
```

## Usage

### 1. Configuration

```yaml
api_ingress:
  auth_disabled: false              # Enable auth
  require_auth_by_default: true     # Require token on all routes
  jwks_uri: "https://auth.example.com/.well-known/jwks.json"
  issuer: "https://auth.example.com"
  audience: "my-api"
```

### 2. Handler with Auth

```rust
use modkit_auth::axum_ext::{Authz, Claims};
use axum::{Json, Extension};

async fn list_users(
    Authz(ctx): Authz,               // Validated SecurityCtx
    Extension(claims): Extension<Claims>,  // Optional: JWT claims
) -> Json<Vec<User>> {
    // ctx.scope() → tenant-scoped access
    // ctx.subject_id() → user ID
    // claims.roles → user roles
    
    let users = repository.find()
        .secure()
        .scope_with(ctx.scope())
        .all(&db)
        .await?;
    
    Json(users)
}
```

### 3. Define Route Requirements

Use explicit `.require_auth()` method:

```rust
OperationBuilder::get("/users")
    .require_auth("users", "read")  // ← Explicit auth requirement
    .operation_id("list_users")     // Optional: for OpenAPI only
    .handler(list_users)
    .register(router, &openapi);
```

For public routes:

```rust
OperationBuilder::get("/health")
    .public()  // ← No auth required
    .handler(health_check)
    .register(router, &openapi);
```

### 4. JWT Token Format

```json
{
  "sub": "550e8400-e29b-41d4-a716-446655440000",
  "iss": "https://auth.example.com",
  "aud": ["my-api"],
  "exp": 1735689600,
  "tenants": [
    "00000000-0000-0000-0000-000000000001",
    "00000000-0000-0000-0000-000000000002"
  ],
  "roles": [
    "users:read",
    "users:write",
    "posts:*"
  ]
}
```

## Role Patterns

- **Exact match**: `users:read` → requires `users:read`
- **Resource wildcard**: `users:*` → grants all actions on users
- **Action wildcard**: `*:read` → grants read on all resources
- **Full wildcard**: `*:*` → grants all permissions

## Development Mode

Disable auth entirely for local development:

```yaml
api_ingress:
  auth_disabled: true
```

This injects `SecurityCtx::root_ctx()` on all requests (system-level access).

## Components

### TokenValidator

Validates JWT tokens and extracts claims.

**Implementations:**
- `JwksValidator` - Production JWKS/OIDC validator
- `NoopValidator` - Used internally when auth is disabled

### ScopeBuilder

Converts JWT claims to `AccessScope`.

**Implementations:**
- `SimpleScopeBuilder` - Converts `tenants` claim to scope

### PrimaryAuthorizer

Checks if claims satisfy security requirements.

**Implementations:**
- `RoleAuthorizer` - Role-based pattern matching

## Security Guarantees

✅ **No bypass** - Only `api_ingress` can disable auth via config
✅ **Centralized** - All routes go through the same middleware
✅ **Type-safe** - Handlers extract `SecurityCtx`, not raw tokens
✅ **Auditable** - All auth decisions logged

## Testing

```rust
#[cfg(test)]
mod tests {
    use modkit_auth::*;

    #[tokio::test]
    async fn test_authorization() {
        let auth = RoleAuthorizer::default();
        let claims = Claims {
            sub: Uuid::new_v4(),
            roles: vec!["users:read".to_string()],
            // ...
        };
        
        let req = SecRequirement::new("users", "read");
        assert!(auth.check(&claims, &req).await.is_ok());
    }
}
```

## License

Apache-2.0

