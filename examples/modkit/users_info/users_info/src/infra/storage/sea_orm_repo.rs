//! SeaORM-backed repository implementation for the domain port.
//!
//! Uses `SecureConn` to automatically enforce security scoping on all database operations.
//! All queries are filtered by the security context provided at the request level.
//!
//! # Type-Safe OData Implementation
//!
//! This module demonstrates the complete type-safe OData approach:
//! - Uses generated `UserDtoFilterField` enum for all field references
//! - No string-based field names anywhere
//! - No exposure of SeaORM Column types to API/domain layers
//! - All filtering, ordering, and cursor extraction is type-safe

use anyhow::Context;
use sea_orm::{PaginatorTrait, Set};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::api::rest::dto::UserDtoFilterField;
use crate::domain::repo::UsersRepository;
use crate::infra::storage::entity::{ActiveModel as UserAM, Column, Entity as UserEntity};
use crate::infra::storage::odata_mapper::UserODataMapper;
use modkit_db::odata::{paginate_odata, LimitCfg};
use modkit_db::secure::{SecureConn, SecurityCtx};
use modkit_odata::{ODataQuery, Page, SortDir};
use user_info_sdk::User;

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

#[async_trait::async_trait]
impl UsersRepository for SeaOrmUsersRepository {
    #[instrument(
        skip(self, ctx),
        fields(
            db.system = %self.sec.db_engine(),
            db.operation = "SELECT",
            user.id = %id
        )
    )]
    async fn find_by_id(&self, ctx: &SecurityCtx, id: Uuid) -> anyhow::Result<Option<User>> {
        debug!("Finding user by id with security context");

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
        skip(self, ctx),
        fields(
            db.system = %self.sec.db_engine(),
            db.operation = "SELECT COUNT",
            user.email = %email
        )
    )]
    async fn email_exists(&self, ctx: &SecurityCtx, email: &str) -> anyhow::Result<bool> {
        debug!("Checking if email exists within security scope");

        use sea_orm::sea_query::Expr;
        let secure_query = self
            .sec
            .find::<UserEntity>(ctx)
            .filter(sea_orm::Condition::all().add(Expr::col(Column::Email).eq(email)));

        let count = secure_query
            .into_inner()
            .count(self.sec.conn())
            .await
            .context("email_exists query failed")?;

        Ok(count > 0)
    }

    #[instrument(
        skip(self, ctx, u),
        fields(
            db.system = %self.sec.db_engine(),
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

        let _ = self
            .sec
            .insert::<UserEntity>(ctx, m)
            .await
            .context("Secure insert failed")?;

        Ok(())
    }

    #[instrument(
        skip(self, ctx, u),
        fields(
            db.system = %self.sec.db_engine(),
            db.operation = "UPDATE",
            user.id = %u.id,
            user.email = %u.email
        )
    )]
    async fn update(&self, ctx: &SecurityCtx, u: User) -> anyhow::Result<()> {
        debug!("Updating user with security validation");

        let m = UserAM {
            id: Set(u.id),
            tenant_id: Set(u.tenant_id),
            email: Set(u.email),
            display_name: Set(u.display_name),
            created_at: Set(u.created_at),
            updated_at: Set(u.updated_at),
        };

        let _ = self
            .sec
            .update_with_ctx::<UserEntity>(ctx, u.id, m)
            .await
            .context("Secure update failed")?;

        Ok(())
    }

    #[instrument(
        skip(self, ctx),
        fields(
            db.system = %self.sec.db_engine(),
            db.operation = "DELETE",
            user.id = %id
        )
    )]
    async fn delete(&self, ctx: &SecurityCtx, id: Uuid) -> anyhow::Result<bool> {
        debug!("Deleting user with security validation");

        let deleted = self
            .sec
            .delete_by_id::<UserEntity>(ctx, id)
            .await
            .context("Secure delete failed")?;

        Ok(deleted)
    }

    #[instrument(
        skip(self, ctx, query),
        fields(
            db.system = %self.sec.db_engine(),
            db.operation = "SELECT"
        )
    )]
    async fn list_users_page(
        &self,
        ctx: &SecurityCtx,
        query: &ODataQuery,
    ) -> Result<Page<User>, modkit_odata::Error> {
        debug!("Listing users with fully type-safe OData");

        // Apply security scope first
        let secure_query = self.sec.find::<UserEntity>(ctx);

        let base_query = secure_query.into_inner();

        // Use the new type-safe pagination - it handles filters, ordering, and cursors
        paginate_odata::<UserDtoFilterField, UserODataMapper, _, _, _, _>(
            base_query,
            self.sec.conn(),
            query,
            ("id", SortDir::Desc), // Default tiebreaker
            LimitCfg {
                default: 25,
                max: 1000,
            },
            |model| model.into(),
        )
        .await
    }
}
