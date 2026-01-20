use async_trait::async_trait;

use crate::domain::error::DomainError;
use crate::domain::repos::CitiesRepository;
use crate::infra::storage::db::db_err;
use crate::infra::storage::entity::city::{
    ActiveModel as CityAM, Column as CityColumn, Entity as CityEntity,
};
use crate::infra::storage::odata_mapper::CityODataMapper;
use modkit_db::DbConnTrait;
use modkit_db::odata::{LimitCfg, paginate_odata};
use modkit_db::secure::{SecureDeleteExt, SecureEntityExt};
use modkit_odata::{ODataQuery, Page, SortDir};
use modkit_security::AccessScope;
use sea_orm::sea_query::Expr;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryFilter, Set};
use user_info_sdk::City;
use user_info_sdk::odata::CityFilterField;
use uuid::Uuid;

/// ORM-based implementation of the `CitiesRepository` trait.
#[derive(Clone)]
pub struct OrmCitiesRepository {
    limit_cfg: LimitCfg,
}

impl OrmCitiesRepository {
    #[must_use]
    pub fn new(limit_cfg: LimitCfg) -> Self {
        Self { limit_cfg }
    }
}

#[async_trait]
impl CitiesRepository for OrmCitiesRepository {
    async fn get<C: DbConnTrait + Send + Sync>(
        &self,
        conn: &C,
        scope: &AccessScope,
        id: Uuid,
    ) -> Result<Option<City>, DomainError> {
        let found = CityEntity::find()
            .filter(sea_orm::Condition::all().add(Expr::col(CityColumn::Id).eq(id)))
            .secure()
            .scope_with(scope)
            .one(conn)
            .await
            .map_err(db_err)?;
        Ok(found.map(Into::into))
    }

    async fn list_page<C: DbConnTrait + Send + Sync>(
        &self,
        conn: &C,
        scope: &AccessScope,
        query: &ODataQuery,
    ) -> Result<Page<City>, DomainError> {
        let base_query = CityEntity::find().secure().scope_with(scope).into_inner();

        let page = paginate_odata::<CityFilterField, CityODataMapper, _, _, _, _>(
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

    async fn create<C: DbConnTrait + Send + Sync>(
        &self,
        conn: &C,
        scope: &AccessScope,
        city: City,
    ) -> Result<City, DomainError> {
        if !scope.has_tenants() {
            return Err(DomainError::validation(
                "scope",
                "Security scope must contain tenant for insert operation",
            ));
        }

        let m = CityAM {
            id: Set(city.id),
            tenant_id: Set(city.tenant_id),
            name: Set(city.name.clone()),
            country: Set(city.country.clone()),
            created_at: Set(city.created_at),
            updated_at: Set(city.updated_at),
        };

        let _ = m.insert(conn).await.map_err(db_err)?;
        Ok(city)
    }

    async fn update<C: DbConnTrait + Send + Sync>(
        &self,
        conn: &C,
        scope: &AccessScope,
        city: City,
    ) -> Result<City, DomainError> {
        let exists = CityEntity::find()
            .filter(sea_orm::Condition::all().add(Expr::col(CityColumn::Id).eq(city.id)))
            .secure()
            .scope_with(scope)
            .one(conn)
            .await
            .map_err(db_err)?
            .is_some();

        if !exists {
            return Err(DomainError::not_found("City", city.id));
        }

        let m = CityAM {
            id: Set(city.id),
            tenant_id: Set(city.tenant_id),
            name: Set(city.name.clone()),
            country: Set(city.country.clone()),
            created_at: Set(city.created_at),
            updated_at: Set(city.updated_at),
        };

        let _ = m.update(conn).await.map_err(db_err)?;
        Ok(city)
    }

    async fn delete<C: DbConnTrait + Send + Sync>(
        &self,
        conn: &C,
        scope: &AccessScope,
        id: Uuid,
    ) -> Result<bool, DomainError> {
        let result = CityEntity::delete_many()
            .filter(sea_orm::Condition::all().add(Expr::col(CityColumn::Id).eq(id)))
            .secure()
            .scope_with(scope)
            .exec(conn)
            .await
            .map_err(db_err)?;

        Ok(result.rows_affected > 0)
    }
}
