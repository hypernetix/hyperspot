use modkit_db_macros::Scopable;
use sea_orm::entity::prelude::*;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "addresses")]
#[secure(tenant_col = "tenant_id", resource_col = "id", no_owner, no_type)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub city_id: Uuid,
    pub street: String,
    pub postal_code: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::city::Entity",
        from = "Column::CityId",
        to = "super::city::Column::Id"
    )]
    City,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::city::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::City.def()
    }
}
