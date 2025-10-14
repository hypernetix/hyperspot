//! SeaORM-backed repository implementation for the domain port.
//!
//! Uses `SecureConn` to automatically enforce security scoping on all database operations.
//! All queries are filtered by the security context provided at the request level.

use anyhow::Context;
use once_cell::sync::Lazy;
use sea_orm::{PaginatorTrait, Set};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::contract::User;
use crate::domain::repo::UsersRepository;
use crate::infra::storage::entity::{ActiveModel as UserAM, Column, Entity as UserEntity};
use modkit_db::odata;
use modkit_db::secure::{SecureConn, SecurityCtx};
use modkit_odata::ODataQuery;
use modkit_odata::Page;

use modkit_odata::SortDir;

/// SeaORM repository implementation with automatic security scoping.
///
/// This repository uses `SecureConn` to ensure all database operations
/// respect the security context provided by the caller. Queries are automatically
/// filtered based on tenant/resource access rules.
///
/// # Security Model
///
/// The users table is tenant-scoped via the `tenant_id` column:
/// - **Tenant isolation**: Users are automatically filtered by tenant_id from the security context
/// - **Email uniqueness**: Email addresses must be unique within a tenant (not globally)
/// - **Deny-by-default**: Empty security context denies all access
pub struct SeaOrmUsersRepository {
    sec: SecureConn,
}

impl SeaOrmUsersRepository {
    /// Create a new repository with a secure database connection.
    pub fn new(sec: SecureConn) -> Self {
        Self { sec }
    }
}

// Whitelist of fields available in $filter (API name -> DB column) with extractors
static USER_FMAP: Lazy<odata::FieldMap<UserEntity>> = Lazy::new(|| {
    odata::FieldMap::<UserEntity>::new()
        .insert_with_extractor("id", Column::Id, odata::FieldKind::Uuid, |m| {
            m.id.to_string()
        })
        .insert_with_extractor("email", Column::Email, odata::FieldKind::String, |m| {
            m.email.clone()
        })
        .insert_with_extractor(
            "created_at",
            Column::CreatedAt,
            odata::FieldKind::DateTimeUtc,
            |m| m.created_at.to_rfc3339(),
        )
});

#[async_trait::async_trait]
impl UsersRepository for SeaOrmUsersRepository {
    #[instrument(
        name = "users_info.repo.find_by_id",
        skip(self, ctx),
        fields(
            db.system = "sqlite",
            db.operation = "SELECT",
            user.id = %id
        )
    )]
    async fn find_by_id(&self, ctx: &SecurityCtx, id: Uuid) -> anyhow::Result<Option<User>> {
        debug!("Finding user by id with security context");

        // Use SecureConn to automatically apply security filtering
        let found = self
            .sec
            .find_by_id::<UserEntity>(ctx, id)
            .context("Failed to create secure query")?
            .one(self.sec.conn())
            .await
            .context("find_by_id query failed")?;

        Ok(found.map(Into::into))
    }

    #[instrument(
        name = "users_info.repo.email_exists",
        skip(self, ctx),
        fields(
            db.system = "sqlite",
            db.operation = "SELECT COUNT",
            user.email = %email
        )
    )]
    async fn email_exists(&self, ctx: &SecurityCtx, email: &str) -> anyhow::Result<bool> {
        debug!("Checking if email exists within security scope");

        // Use SecureConn to ensure we only check within accessible scope
        use sea_orm::sea_query::Expr;
        let secure_query = self
            .sec
            .find::<UserEntity>(ctx)
            .context("Failed to create secure query")?
            .filter(sea_orm::Condition::all().add(Expr::col(Column::Email).eq(email)));

        // Get the underlying Select and execute count
        let count = secure_query
            .into_inner()
            .count(self.sec.conn())
            .await
            .context("email_exists query failed")?;

        Ok(count > 0)
    }

    #[instrument(
        name = "users_info.repo.insert",
        skip(self, ctx, u),
        fields(
            db.system = "sqlite",
            db.operation = "INSERT",
            user.id = %u.id,
            user.email = %u.email
        )
    )]
    async fn insert(&self, ctx: &SecurityCtx, u: User) -> anyhow::Result<()> {
        debug!("Inserting new user with security validation");

        let m = UserAM {
            id: Set(u.id),
            tenant_id: Set(u.tenant_id),
            email: Set(u.email),
            display_name: Set(u.display_name),
            created_at: Set(u.created_at),
            updated_at: Set(u.updated_at),
        };

        // Secure insert validates that tenant_id matches the security context
        let _ = self
            .sec
            .insert::<UserEntity>(ctx, m)
            .await
            .context("Secure insert failed")?;

        Ok(())
    }

    #[instrument(
        name = "users_info.repo.update",
        skip(self, ctx, u),
        fields(
            db.system = "sqlite",
            db.operation = "UPDATE",
            user.id = %u.id,
            user.email = %u.email
        )
    )]
    async fn update(&self, ctx: &SecurityCtx, u: User) -> anyhow::Result<()> {
        debug!("Updating user with security validation");

        // Build ActiveModel for update
        let m = UserAM {
            id: Set(u.id),
            tenant_id: Set(u.tenant_id),
            email: Set(u.email),
            display_name: Set(u.display_name),
            created_at: Set(u.created_at),
            updated_at: Set(u.updated_at),
        };

        // update_with_ctx validates the entity is in scope before updating
        let _ = self
            .sec
            .update_with_ctx::<UserEntity>(ctx, u.id, m)
            .await
            .context("Secure update failed")?;

        Ok(())
    }

    #[instrument(
        name = "users_info.repo.delete",
        skip(self, ctx),
        fields(
            db.system = "sqlite",
            db.operation = "DELETE",
            user.id = %id
        )
    )]
    async fn delete(&self, ctx: &SecurityCtx, id: Uuid) -> anyhow::Result<bool> {
        debug!("Deleting user with security validation");

        // Use SecureConn's delete_by_id which validates the entity is in scope
        let deleted = self
            .sec
            .delete_by_id::<UserEntity>(ctx, id)
            .await
            .context("Secure delete failed")?;

        Ok(deleted)
    }

    #[instrument(
        name = "users_info.repo.list_users_page",
        skip(self, ctx, query),
        fields(
            db.system = "sqlite",
            db.operation = "SELECT"
        )
    )]
    async fn list_users_page(
        &self,
        ctx: &SecurityCtx,
        query: &ODataQuery,
    ) -> Result<Page<User>, modkit_odata::Error> {
        debug!("Listing users with security filtering");

        // Create a secure base query that automatically applies scoping
        let secure_query = self.sec.find::<UserEntity>(ctx).map_err(|e| {
            // Convert ScopeError to ODataError
            tracing::error!(error = %e, "Failed to create secure query");
            modkit_odata::Error::Db(format!("Failed to create secure query: {}", e))
        })?;

        // Extract the underlying Select query (which has security filters already applied)
        let base_query = secure_query.into_inner();

        // Use the OData pagination helper with the secure base query
        modkit_db::odata::paginate_with_odata::<UserEntity, User, _, _>(
            base_query,
            self.sec.conn(),
            query,
            &USER_FMAP,
            ("id", SortDir::Desc),
            modkit_db::odata::LimitCfg {
                default: 25,
                max: 1000,
            },
            |model| model.into(),
        )
        .await
    }
}
