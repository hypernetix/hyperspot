# Secure ORM and Database Access

This document describes how to use the secure ORM layer (`SecureConn` + `AccessScope`) and what is forbidden (raw DB handles, plain SQL in module code).

## Core invariants

- **Rule**: Use `SecureConn` for all DB access in handlers/services.
- **Rule**: Use `AccessScope` for tenant/resource scoping. Build it from request context (e.g., `SecurityContext` + resolved accessible tenants).
- **Rule**: Derive `Scopable` on SeaORM entities with tenant/resource columns.
- **Rule**: Modules cannot access raw database connections/pools.
- **Rule**: No plain SQL in handlers/services/repos. Raw SQL is allowed only in migration infrastructure.

## Implicit security policy (how `AccessScope` is applied)

| Scope | Entity has column? | Result |
|------|---------------------|--------|
| Empty (`tenant_ids` and `resource_ids` empty) | N/A | deny all (`WHERE false`) |
| Tenants only | has `tenant_col` | `tenant_col IN (tenant_ids)` |
| Tenants only | no `tenant_col` | deny all |
| Resources only | has `resource_col` | `resource_col IN (resource_ids)` |
| Resources only | no `resource_col` | deny all |
| Tenants + resources | has both | AND them |
| Tenants + resources | missing either column | deny all |

This is enforced inside `modkit-db` when you call `.scope_with(&scope)` / `SecureConn::find*` / `SecureConn::update_many` / `SecureConn::delete_many`.

## SecureConn usage

### Preferred: SecureConn for scoped access

```rust
use modkit_db::secure::AccessScope;

pub async fn list_users(
    Authz(ctx): Authz,
    Extension(db): Extension<Arc<DbHandle>>,
) -> ApiResult<JsonPage<UserDto>> {
    let secure_conn = db.sea_secure();
    let scope = AccessScope::tenant(ctx.tenant_id());
    let users = secure_conn
        .find::<user::Entity>(&scope)
        .all(&secure_conn)
        .await?;
    Ok(Json(users.into_iter().map(UserDto::from).collect()))
}
```

## Executors: `DBRunner` and `SecureTx`

- Repository methods should accept **`runner: &impl DBRunner`**, not `&SecureConn`.
- Inside a transaction callback, you get **`&SecureTx`**. It also implements `DBRunner`, so the same repository methods work both inside and outside a transaction.

Example signature:

```rust
use modkit_db::secure::{AccessScope, DBRunner};

pub async fn create_user(
    runner: &impl DBRunner,
    scope: &AccessScope,
    user: user::ActiveModel,
) -> Result<user::Model, ScopeError> {
    // ...
}
```

## Database Migrations

Modules provide migration definitions that the runtime executes with a privileged connection:

```rust
impl DatabaseCapability for MyModule {
    fn migrations(&self) -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        use sea_orm_migration::MigratorTrait;
        crate::infra::storage::migrations::Migrator::migrations()
    }
}
```

Each module gets its own migration history table (`modkit_migrations__<prefix>__<hash8>`), ensuring isolation between modules.

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

- `tenant_col = "..."` / `no_tenant`
- `resource_col = "..."` / `no_resource`
- `owner_col = "..."` / `no_owner`
- `type_col = "..."` / `no_type`
- `unrestricted` (special case; cannot be combined with other attributes)

Rule: all four dimensions must be declared (either `*_col` or `no_*`), unless `unrestricted` is used.

### Unrestricted entities (`#[secure(unrestricted)]`)

Use `#[secure(unrestricted)]` only for truly global tables where the entity has **no scoping columns**. Notes:

- `secure_insert` does not require `tenant_id` for such entities.
- Queries with a scope that contains tenant IDs will be denied (by policy: tenants requested but entity has no `tenant_col`).
- If you need to read/write a global table within a tenant-scoped request, do not use `unrestricted`. Model it with explicit columns and use an appropriate scope shape (often `resources_only`).

## AccessScope in queries

### Auto-scoped queries

```rust
let secure_conn = db.sea_secure();
let scope = AccessScope::tenant(ctx.tenant_id());

// Automatically adds tenant_id = ? filter
let users = secure_conn
    .find::<user::Entity>(&scope)
    .all(&secure_conn)
    .await?;

// Automatically adds tenant_id = ? AND id = ? filters
let user = secure_conn
    .find_by_id::<user::Entity>(&scope, user_id)?
    .one(&secure_conn)
    .await?;
```

### Manual scoping

```rust
use modkit_db::secure::SecureEntityExt;

// For complex queries, build your filters first, then apply scope and execute via SecureConn.
let user = user::Entity::find()
    .filter(user::Column::Email.eq(email))
    .secure()
    .scope_with(&scope)
    .one(&secure_conn)
    .await?;
```

### Advanced scoping for joins / related entities

Use these when the base entity cannot be tenant-filtered directly:

- `SecureSelect::and_scope_for::<J>(&scope)` — apply tenant scoping on a joined entity `J`.
- `SecureSelect::scope_via_exists::<J>(&scope)` — apply tenant scoping via an `EXISTS` subquery on `J`.

## Repository pattern

### Repository with `DBRunner` (works with both `SecureConn` and `SecureTx`)

```rust
use modkit_db::secure::{AccessScope, DBRunner, ScopeError, SecureEntityExt};
use sea_orm::Set;

pub struct UserRepository;

impl UserRepository {
    pub async fn find_by_id(
        &self,
        runner: &impl DBRunner,
        scope: &AccessScope,
        id: Uuid,
    ) -> Result<Option<user::Model>, ScopeError> {
        Ok(user::Entity::find_by_id(id)
            .secure()
            .scope_with(scope)
            .one(runner)
            .await?)
    }

    pub async fn create(
        &self,
        runner: &impl DBRunner,
        scope: &AccessScope,
        new_user: user_info_sdk::NewUser,
    ) -> Result<user::Model, ScopeError> {
        let am = user::ActiveModel {
            id: Set(new_user.id.unwrap_or_else(Uuid::new_v4)),
            tenant_id: Set(new_user.tenant_id),
            email: Set(new_user.email),
            display_name: Set(new_user.display_name),
            ..Default::default()
        };

        modkit_db::secure::secure_insert::<user::Entity>(am, scope, runner).await
    }
}
```

## Mutations (security rules)

### Insert (`secure_insert` / `SecureConn::insert`)

- If the entity has a `tenant_col`, the `ActiveModel` MUST include `tenant_id`.
- The inserted `tenant_id` MUST be inside `scope.tenant_ids()`.
- Violations are errors (`Denied` / `TenantNotInScope` / `Invalid("tenant_id is required")`).

### Update one record (`SecureConn::update_with_ctx`)

- There is no public unscoped update-one API.
- `update_with_ctx(scope, id, am)` first checks the row exists in scope.
- For tenant-scoped entities, `tenant_id` is immutable. Attempts to change it are denied.

### Update many (`SecureConn::update_many`)

- Must be scoped via `scope_with` / `SecureConn::update_many(scope)`.
- Attempts to set the `tenant_id` column are denied at runtime (`Denied("tenant_id is immutable")`).

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
    let scope = AccessScope::tenant(ctx.tenant_id());

    secure_conn
        .in_transaction_mapped(DomainError::database_infra, move |tx| {
            Box::pin(async move {
                // Use `tx` as the connection for repository calls.
                // Example:
                // repo.transfer_user(tx, &scope, from_tenant, to_tenant, user_id).await?;
                Ok(())
            })
        })
        .await
}
```

## Raw SQL (policy)

Raw SQL is **allowed only in migration infrastructure** (migration runner + migration definitions).

- Module code (handlers/services/repos) must use the Secure ORM (`SecureConn` / `SecureTx` + secure wrappers).
- Direct SQL execution from module code is forbidden.

## Migration considerations

### Migrations use raw SQL

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Users::TenantId).uuid().not_null())
                    .col(ColumnDef::new(Users::Email).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    TenantId,
    Email,
}
```

## Testing with SecureConn

### Test setup

```rust
use modkit_db::DbHandle;
use modkit_security::AccessScope;

#[tokio::test]
async fn test_user_repository() {
    let db = setup_test_db().await;
    let scope = AccessScope::tenant(Uuid::new_v4());
    let repo = UserRepository;
    let conn = db.sea_secure();

    // Test operations
    let user = repo.create(&conn, &scope, new_user).await.unwrap();
    let found = repo.find_by_id(&conn, &scope, user.id).await.unwrap();
    assert_eq!(found.id, user.id);
}
```

## Quick checklist

- [ ] Derive `Scopable` on SeaORM entities with `tenant_col` (required).
- [ ] Use `db.sea_secure()` for all DB access in handlers/services.
- [ ] Build `AccessScope` from request context (and resolved accessible tenants if applicable).
- [ ] Use `secure_conn.find::<Entity>(&scope).all(&secure_conn)` for auto-scoped queries.
- [ ] Use `secure_conn.update_with_ctx::<Entity>(&scope, id, am)` for single-record updates.
- [ ] Use raw SQL only in `migrations/*.rs` (enforced later via dylint).
- [ ] Add indexes on security columns (tenant_id, resource_id).
- [ ] In tests, build scopes explicitly (`AccessScope::tenant(...)`, `AccessScope::tenants_only(...)`, `AccessScope::resources_only(...)`).

## Related docs

- OData pagination / filtering: `docs/modkit_unified_system/07_odata_pagination_select_filter.md`
