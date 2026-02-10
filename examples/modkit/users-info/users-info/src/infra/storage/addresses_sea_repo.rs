use async_trait::async_trait;

use crate::domain::error::DomainError;
use crate::domain::repos::AddressesRepository;
use crate::infra::storage::db::db_err;
use crate::infra::storage::entity::address::{
    ActiveModel as AddressAM, Column as AddressColumn, Entity as AddressEntity,
};
use crate::infra::storage::odata_mapper::AddressODataMapper;
use modkit_db::odata::{LimitCfg, paginate_odata};
use modkit_db::secure::{
    DBRunner, SecureDeleteExt, SecureEntityExt, secure_insert, secure_update_with_scope,
};
use modkit_odata::{ODataQuery, Page, SortDir};
use modkit_security::AccessScope;
use sea_orm::sea_query::Expr;
use sea_orm::{EntityTrait, QueryFilter, Set};
use users_info_sdk::Address;
use users_info_sdk::odata::AddressFilterField;
use uuid::Uuid;

/// ORM-based implementation of the `AddressesRepository` trait.
#[derive(Clone)]
pub struct OrmAddressesRepository {
    limit_cfg: LimitCfg,
}

impl OrmAddressesRepository {
    #[must_use]
    pub fn new(limit_cfg: LimitCfg) -> Self {
        Self { limit_cfg }
    }
}

#[async_trait]
impl AddressesRepository for OrmAddressesRepository {
    async fn get<C: DBRunner>(
        &self,
        conn: &C,
        scope: &AccessScope,
        id: Uuid,
    ) -> Result<Option<Address>, DomainError> {
        let found = AddressEntity::find()
            .filter(sea_orm::Condition::all().add(Expr::col(AddressColumn::Id).eq(id)))
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
    ) -> Result<Page<Address>, DomainError> {
        let base_query = AddressEntity::find().secure().scope_with(scope);

        let page = paginate_odata::<AddressFilterField, AddressODataMapper, _, _, _, _>(
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

    async fn get_by_user_id<C: DBRunner>(
        &self,
        conn: &C,
        scope: &AccessScope,
        user_id: Uuid,
    ) -> Result<Option<Address>, DomainError> {
        let found = AddressEntity::find()
            .filter(sea_orm::Condition::all().add(Expr::col(AddressColumn::UserId).eq(user_id)))
            .secure()
            .scope_with(scope)
            .one(conn)
            .await
            .map_err(db_err)?;
        Ok(found.map(Into::into))
    }

    async fn create<C: DBRunner>(
        &self,
        conn: &C,
        scope: &AccessScope,
        address: Address,
    ) -> Result<Address, DomainError> {
        let m = AddressAM {
            id: Set(address.id),
            tenant_id: Set(address.tenant_id),
            user_id: Set(address.user_id),
            city_id: Set(address.city_id),
            street: Set(address.street.clone()),
            postal_code: Set(address.postal_code.clone()),
            created_at: Set(address.created_at),
            updated_at: Set(address.updated_at),
        };

        let _ = secure_insert::<AddressEntity>(m, scope, conn)
            .await
            .map_err(db_err)?;
        Ok(address)
    }

    async fn update<C: DBRunner>(
        &self,
        conn: &C,
        scope: &AccessScope,
        address: Address,
    ) -> Result<Address, DomainError> {
        let exists = AddressEntity::find()
            .filter(sea_orm::Condition::all().add(Expr::col(AddressColumn::Id).eq(address.id)))
            .secure()
            .scope_with(scope)
            .one(conn)
            .await
            .map_err(db_err)?
            .is_some();

        if !exists {
            return Err(DomainError::not_found("Address", address.id));
        }

        let m = AddressAM {
            id: Set(address.id),
            tenant_id: Set(address.tenant_id),
            user_id: Set(address.user_id),
            city_id: Set(address.city_id),
            street: Set(address.street.clone()),
            postal_code: Set(address.postal_code.clone()),
            created_at: Set(address.created_at),
            updated_at: Set(address.updated_at),
        };

        let _ = secure_update_with_scope::<AddressEntity>(m, scope, address.id, conn)
            .await
            .map_err(db_err)?;
        Ok(address)
    }

    async fn delete<C: DBRunner>(
        &self,
        conn: &C,
        scope: &AccessScope,
        id: Uuid,
    ) -> Result<bool, DomainError> {
        let result = AddressEntity::delete_many()
            .filter(sea_orm::Condition::all().add(Expr::col(AddressColumn::Id).eq(id)))
            .secure()
            .scope_with(scope)
            .exec(conn)
            .await
            .map_err(db_err)?;

        Ok(result.rows_affected > 0)
    }

    async fn delete_by_user_id<C: DBRunner>(
        &self,
        conn: &C,
        scope: &AccessScope,
        user_id: Uuid,
    ) -> Result<u64, DomainError> {
        let result = AddressEntity::delete_many()
            .filter(sea_orm::Condition::all().add(Expr::col(AddressColumn::UserId).eq(user_id)))
            .secure()
            .scope_with(scope)
            .exec(conn)
            .await
            .map_err(db_err)?;

        Ok(result.rows_affected)
    }
}
