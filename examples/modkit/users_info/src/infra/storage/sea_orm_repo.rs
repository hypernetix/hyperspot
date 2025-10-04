//! SeaORM-backed repository implementation for the domain port.
//!
//! This struct is generic over `C: ConnectionTrait`, so you can construct it
//! with a `DatabaseConnection` **or** a transactional connection.
//! For transactional flows create a new instance with a transaction connection
//! and pass it down to a short-lived service (or expose lower-level APIs).

use anyhow::Context;
use once_cell::sync::Lazy;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::contract::User;
use crate::domain::repo::UsersRepository;
use crate::infra::storage::entity::{ActiveModel as UserAM, Column, Entity as UserEntity};
use modkit_db::odata;
use odata_core::ODataQuery;
use odata_core::Page;

use odata_core::SortDir;

/// SeaORM repository impl.
/// Holds a connection object; its lifetime/ownership is up to the caller.
pub struct SeaOrmUsersRepository<C>
where
    C: ConnectionTrait + Send + Sync,
{
    conn: C,
}

impl<C> SeaOrmUsersRepository<C>
where
    C: ConnectionTrait + Send + Sync,
{
    pub fn new(conn: C) -> Self {
        Self { conn }
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
impl<C> UsersRepository for SeaOrmUsersRepository<C>
where
    C: ConnectionTrait + Send + Sync + 'static,
{
    #[instrument(
        name = "users_info.repo.find_by_id",
        skip(self),
        fields(
            db.system = "sqlite",
            db.operation = "SELECT",
            user.id = %id
        )
    )]
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<User>> {
        debug!("Finding user by id");
        let found = UserEntity::find_by_id(id)
            .one(&self.conn)
            .await
            .context("find_by_id failed")?;
        Ok(found.map(Into::into))
    }

    #[instrument(
        name = "users_info.repo.email_exists",
        skip(self),
        fields(
            db.system = "sqlite",
            db.operation = "SELECT COUNT",
            user.email = %email
        )
    )]
    async fn email_exists(&self, email: &str) -> anyhow::Result<bool> {
        debug!("Checking if email exists");
        let count = UserEntity::find()
            .filter(Column::Email.eq(email))
            .count(&self.conn)
            .await
            .context("email_exists failed")?;
        Ok(count > 0)
    }

    #[instrument(
        name = "users_info.repo.insert",
        skip(self, u),
        fields(
            db.system = "sqlite",
            db.operation = "INSERT",
            user.id = %u.id,
            user.email = %u.email
        )
    )]
    async fn insert(&self, u: User) -> anyhow::Result<()> {
        debug!("Inserting new user");
        let m = UserAM {
            id: Set(u.id),
            email: Set(u.email),
            display_name: Set(u.display_name),
            created_at: Set(u.created_at),
            updated_at: Set(u.updated_at),
        };
        let _ = m.insert(&self.conn).await.context("insert failed")?;
        Ok(())
    }

    #[instrument(
        name = "users_info.repo.update",
        skip(self, u),
        fields(
            db.system = "sqlite",
            db.operation = "UPDATE",
            user.id = %u.id,
            user.email = %u.email
        )
    )]
    async fn update(&self, u: User) -> anyhow::Result<()> {
        debug!("Updating user");
        // Minimal upsert-by-PK via ActiveModel::update
        let m = UserAM {
            id: Set(u.id),
            email: Set(u.email),
            display_name: Set(u.display_name),
            created_at: Set(u.created_at),
            updated_at: Set(u.updated_at),
        };
        let _ = m.update(&self.conn).await.context("update failed")?;
        Ok(())
    }

    #[instrument(
        name = "users_info.repo.delete",
        skip(self),
        fields(
            db.system = "sqlite",
            db.operation = "DELETE",
            user.id = %id
        )
    )]
    async fn delete(&self, id: Uuid) -> anyhow::Result<bool> {
        debug!("Deleting user");
        let res = UserEntity::delete_by_id(id)
            .exec(&self.conn)
            .await
            .context("delete failed")?;
        Ok(res.rows_affected > 0)
    }

    #[instrument(
        name = "users_info.repo.list_users_page",
        skip(self, query),
        fields(
            db.system = "sqlite",
            db.operation = "SELECT"
        )
    )]
    async fn list_users_page(&self, query: &ODataQuery) -> Result<Page<User>, odata_core::Error> {
        modkit_db::odata::paginate_with_odata::<UserEntity, User, _, _>(
            UserEntity::find(),
            &self.conn,
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
