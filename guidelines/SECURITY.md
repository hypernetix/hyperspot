# Security Guideline

## Input Validation

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

- **Never commit secrets** to version control
- **Use environment variables** for configuration
- **Rotate secrets regularly**
- **Use secure random generation** for tokens

```rust
// Bad: hardcoded secret
const API_KEY: &str = "sk-1234567890abcdef";

// Good: environment variable
let api_key = std::env::var("API_KEY")
    .context("API_KEY environment variable not set")?;
```

## Secure ORM

HyperSpot provides a secure-by-default ORM layer that enforces access control at compile time using the typestate pattern. This prevents unscoped database queries from executing and ensures tenant isolation.

For complete documentation on the Secure ORM layer, see [SECURE-ORM.md](../docs/SECURE-ORM.md).

### Key Features

1. **Compile-time enforcement**: Cannot bypass scoping via type system
2. **Deny-by-default**: Empty scopes explicitly denied with `WHERE 1=0`
3. **Tenant isolation**: Automatic filtering when tenant_ids provided
4. **Request-scoped**: Security context tied to request lifecycle
5. **No SQL injection**: Uses SeaORM's parameterized queries
6. **Audit trail**: Raw access logs security warnings

### Quick Example

```rust
use modkit_db::secure::{SecurityCtx, SecureConn};
use modkit_db_macros::Scopable;

// Define a scopable entity
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "users")]
#[secure(tenant_col = "tenant_id")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
}

// Use in API handler
pub async fn list_users_handler(
    Extension(auth): Extension<AuthContext>,
    Extension(db): Extension<DbHandle>,
) -> Result<Json<Vec<User>>, ApiError> {
    // Create security context from request
    let ctx = SecurityCtx::for_tenants(vec![auth.tenant_id], auth.user_id);

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
