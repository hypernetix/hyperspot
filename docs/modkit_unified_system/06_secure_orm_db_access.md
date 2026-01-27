# Secure ORM and Database Access

ModKit provides a secure-by-default ORM layer that enforces request-scoped security with compile-time guarantees. See `docs/SECURE-ORM.md` for the complete guide.

## Core invariants

- **Rule**: Use `SecureConn` for all DB access in handlers/services.
- **Rule**: Use `SecurityContext` for tenant/resource scoping.
- **Rule**: Derive `Scopable` on SeaORM entities with tenant/resource columns.
- **Rule**: Raw access only for migrations/admin tools and requires `insecure-escape` feature.
- **Rule**: Do not bypass SecureConn without explicit justification.

## SecureConn usage

### Preferred: SecureConn for scoped access

```rust
use modkit_db::SecureConn;
use modkit_security::SecurityContext;

pub async fn list_users(
    Authz(ctx): Authz,
    Extension(db): Extension<Arc<DbHandle>>,
) -> ApiResult<JsonPage<UserDto>> {
    let secure_conn = db.sea_secure();
    let users = secure_conn
        .find::<user::Entity>(&ctx)?
        .all(secure_conn.conn())
        .await?;
    Ok(Json(users.into_iter().map(UserDto::from).collect()))
}
```

### Exceptional: Raw access (migrations, admin tools)

```rust
// Requires insecure-escape feature
let sea = db.sea();      // Direct SeaORM connection
let pool = db.sqlx_pool();  // Direct SQLx pool
```

## Scopable entities

### Entity definition

```rust
use modkit_db_macros::Scopable;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "users")]
#[secure(
    tenant_col = "tenant_id",
    resource_col = "id",
    no_owner,
    no_type
)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

### Scopable attributes

- `tenant_col`: Column name for tenant scoping (required for multi-tenant)
- `resource_col`: Column name for resource-level scoping (optional)
- `owner_col`: Column name for owner-based scoping (optional)
- `type_col`: Column name for type-based scoping (optional)
- `no_owner`: Skip owner-based scoping
- `no_type`: Skip type-based scoping

## SecurityContext in queries

### Auto-scoped queries

```rust
let secure_conn = db.sea_secure();

// Automatically adds tenant_id = ? filter
let users = secure_conn
    .find::<user::Entity>(&ctx)?
    .all(secure_conn.conn())
    .await?;

// Automatically adds tenant_id = ? AND id = ? filters
let user = secure_conn
    .find_by_id::<user::Entity>(&ctx, user_id)?
    .one(secure_conn.conn())
    .await?;
```

### Manual scoping

```rust
// For complex queries, use with_security_ctx
let query = user::Entity::find()
    .filter(user::Column::Email.eq(email))
    .with_security_ctx(&ctx);

let user = query.one(secure_conn.conn()).await?;
```

## Repository pattern

### Repository with SecureConn

```rust
pub struct UserRepository {
    db: Arc<DbHandle>,
}

impl UserRepository {
    pub fn new(db: Arc<DbHandle>) -> Self {
        Self { db }
    }

    pub async fn find_by_id(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
    ) -> Result<Option<user::Model>, DomainError> {
        let secure_conn = self.db.sea_secure();
        let user = secure_conn
            .find_by_id::<user::Entity>(ctx, id)?
            .one(secure_conn.conn())
            .await?;
        Ok(user)
    }

    pub async fn list(
        &self,
        ctx: &SecurityContext,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<user::Model>, DomainError> {
        let secure_conn = self.db.sea_secure();
        let users = secure_conn
            .find::<user::Entity>(ctx)?
            .limit(limit)
            .offset(offset)
            .all(secure_conn.conn())
            .await?;
        Ok(users)
    }

    pub async fn create(
        &self,
        ctx: &SecurityContext,
        new_user: user_info_sdk::NewUser,
    ) -> Result<user::Model, DomainError> {
        let secure_conn = self.db.sea_secure();
        let user = user::ActiveModel {
            id: Set(new_user.id.unwrap_or_else(Uuid::new_v4)),
            tenant_id: Set(ctx.tenant_id()),
            email: Set(new_user.email),
            display_name: Set(new_user.display_name),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        let result = user.insert(secure_conn.conn()).await?;
        Ok(result)
    }
}
```

## Transactions

### Transaction with SecureConn

```rust
pub async fn transfer_user(
    &self,
    ctx: &SecurityContext,
    from_tenant: Uuid,
    to_tenant: Uuid,
    user_id: Uuid,
) -> Result<(), DomainError> {
    let secure_conn = self.db.sea_secure();
    let txn = secure_conn.conn().begin().await?;

    // Use transaction for all operations
    let txn_conn = &txn;
    
    // Update user tenant
    let user = user::ActiveModel {
        id: Set(user_id),
        tenant_id: Set(to_tenant),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };
    
    user.update(txn_conn).await?;
    
    txn.commit().await?;
    Ok(())
}
```

## Raw SQL (exceptional)

### When to use raw SQL

- Complex queries not expressible with SeaORM
- Performance-critical bulk operations
- Migrations and admin tools

### Raw SQL with SecureConn

```rust
use sea_orm::Statement;
use sea_orm::FromQueryResult;

#[derive(FromQueryResult)]
struct UserSummary {
    id: Uuid,
    email: String,
    post_count: i64,
}

pub async fn get_user_summary(
    &self,
    ctx: &SecurityContext,
    user_id: Uuid,
) -> Result<UserSummary, DomainError> {
    let secure_conn = self.db.sea_secure();
    
    let stmt = Statement::from_sql_and_values(
        secure_conn.conn().get_database_backend(),
        r#"
        SELECT 
            u.id,
            u.email,
            COUNT(p.id) as post_count
        FROM users u
        LEFT JOIN posts p ON u.id = p.user_id
        WHERE u.tenant_id = $1 AND u.id = $2
        GROUP BY u.id, u.email
        "#,
        [ctx.tenant_id().into(), user_id.into()],
    );

    let result = UserSummary::find_by_statement(stmt)
        .one(secure_conn.conn())
        .await?
        .ok_or(DomainError::UserNotFound { id: user_id })?;

    Ok(result)
}
```

## Migration considerations

### Migrations use raw access

```rust
// In migration files, use raw access
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Use raw SQL for schema changes
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).uuid().primary_key())
                    .col(ColumnDef::new(Users::TenantId).uuid().not_null())
                    .col(ColumnDef::new(Users::Email).string().not_null())
                    .col(ColumnDef::new(Users::DisplayName).string().not_null())
                    .col(ColumnDef::new(Users::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Users::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        // Add indexes for security columns
        manager
            .create_index(
                Index::create()
                    .name("idx_users_tenant_id")
                    .table(Users::Table)
                    .col(Users::TenantId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
```

## Testing with SecureConn

### Test setup

```rust
use modkit_db::DbHandle;
use modkit_security::SecurityContext;

#[tokio::test]
async fn test_user_repository() {
    let db = setup_test_db().await;
    let ctx = SecurityContext::test_tenant(Uuid::new_v4());
    let repo = UserRepository::new(db);

    // Test operations
    let user = repo.create(&ctx, new_user).await.unwrap();
    let found = repo.find_by_id(&ctx, user.id).await.unwrap();
    assert_eq!(found.id, user.id);
}
```

## Quick checklist

- [ ] Derive `Scopable` on SeaORM entities with `tenant_col` (required).
- [ ] Use `db.sea_secure()` for all DB access in handlers/services.
- [ ] Pass `SecurityContext` to repository methods.
- [ ] Use `secure_conn.find::<Entity>(&ctx)?` for auto-scoped queries.
- [ ] Use raw SQL only for exceptional cases (migrations, admin tools).
- [ ] Add indexes on security columns (tenant_id, resource_id).
- [ ] Test with `SecurityContext::test_tenant()` for unit tests.
