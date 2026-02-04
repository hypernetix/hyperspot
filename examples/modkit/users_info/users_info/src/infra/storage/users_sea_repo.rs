use async_trait::async_trait;

use crate::infra::storage::db::db_err;
use crate::infra::storage::entity::user::{ActiveModel as UserAM, Column, Entity as UserEntity};
use crate::infra::storage::odata_mapper::UserODataMapper;
use crate::{domain::error::DomainError, domain::repos::UsersRepository};
use modkit_db::odata::{LimitCfg, paginate_odata};
use modkit_db::secure::{
    DBRunner, SecureDeleteExt, SecureEntityExt, secure_insert, secure_update_with_scope,
};
use modkit_odata::{ODataQuery, Page, SortDir};
use modkit_security::AccessScope;
use sea_orm::sea_query::Expr;
use sea_orm::{EntityTrait, QueryFilter, Set};
use user_info_sdk::User;
use user_info_sdk::odata::UserFilterField;
use uuid::Uuid;

/// ORM-based implementation of the `UsersRepository` trait.
#[derive(Clone)]
pub struct OrmUsersRepository {
    limit_cfg: LimitCfg,
}

impl OrmUsersRepository {
    #[must_use]
    pub fn new(limit_cfg: LimitCfg) -> Self {
        Self { limit_cfg }
    }
}

#[async_trait]
impl UsersRepository for OrmUsersRepository {
    async fn get<C: DBRunner>(
        &self,
        conn: &C,
        scope: &AccessScope,
        id: Uuid,
    ) -> Result<Option<User>, DomainError> {
        let found = UserEntity::find()
            .filter(sea_orm::Condition::all().add(Expr::col(Column::Id).eq(id)))
            .secure()
            .scope_with(scope)
            .one(conn)
            .await
            .map_err(db_err)?;
        Ok(found.map(Into::into))
    }

    async fn list_page<C: DBRunner>(
        &self,
        conn: &C,
        scope: &AccessScope,
        query: &ODataQuery,
    ) -> Result<Page<User>, DomainError> {
        let base_query = UserEntity::find().secure().scope_with(scope);

        let page = paginate_odata::<UserFilterField, UserODataMapper, _, _, _, _>(
            base_query,
            conn,
            query,
            ("id", SortDir::Desc),
            self.limit_cfg,
            Into::into,
        )
        .await
        .map_err(db_err)?;

        Ok(page)
    }

    async fn create<C: DBRunner>(
        &self,
        conn: &C,
        scope: &AccessScope,
        user: User,
    ) -> Result<User, DomainError> {
        let m = UserAM {
            id: Set(user.id),
            tenant_id: Set(user.tenant_id),
            email: Set(user.email.clone()),
            display_name: Set(user.display_name.clone()),
            created_at: Set(user.created_at),
            updated_at: Set(user.updated_at),
        };

        let _ = secure_insert::<UserEntity>(m, scope, conn)
            .await
            .map_err(db_err)?;
        Ok(user)
    }

    async fn update<C: DBRunner>(
        &self,
        conn: &C,
        scope: &AccessScope,
        user: User,
    ) -> Result<User, DomainError> {
        let exists = UserEntity::find()
            .filter(sea_orm::Condition::all().add(Expr::col(Column::Id).eq(user.id)))
            .secure()
            .scope_with(scope)
            .one(conn)
            .await
            .map_err(db_err)?
            .is_some();

        if !exists {
            return Err(DomainError::not_found("User", user.id));
        }

        let m = UserAM {
            id: Set(user.id),
            tenant_id: Set(user.tenant_id),
            email: Set(user.email.clone()),
            display_name: Set(user.display_name.clone()),
            created_at: Set(user.created_at),
            updated_at: Set(user.updated_at),
        };

        let _ = secure_update_with_scope::<UserEntity>(m, scope, user.id, conn)
            .await
            .map_err(db_err)?;
        Ok(user)
    }

    async fn delete<C: DBRunner>(
        &self,
        conn: &C,
        scope: &AccessScope,
        id: Uuid,
    ) -> Result<bool, DomainError> {
        let result = UserEntity::delete_many()
            .filter(sea_orm::Condition::all().add(Expr::col(Column::Id).eq(id)))
            .secure()
            .scope_with(scope)
            .exec(conn)
            .await
            .map_err(db_err)?;

        Ok(result.rows_affected > 0)
    }

    async fn exists<C: DBRunner>(
        &self,
        conn: &C,
        scope: &AccessScope,
        id: Uuid,
    ) -> Result<bool, DomainError> {
        let found = UserEntity::find()
            .filter(sea_orm::Condition::all().add(Expr::col(Column::Id).eq(id)))
            .secure()
            .scope_with(scope)
            .one(conn)
            .await
            .map_err(db_err)?;
        Ok(found.is_some())
    }

    async fn count_by_email<C: DBRunner>(
        &self,
        conn: &C,
        scope: &AccessScope,
        email: &str,
    ) -> Result<u64, DomainError> {
        let count = UserEntity::find()
            .filter(sea_orm::Condition::all().add(Expr::col(Column::Email).eq(email)))
            .secure()
            .scope_with(scope)
            .count(conn)
            .await
            .map_err(db_err)?;
        Ok(count)
    }
}
