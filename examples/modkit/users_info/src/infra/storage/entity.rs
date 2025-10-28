use chrono::{DateTime, Utc};
use modkit_db::secure::ScopableEntity;
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
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

// Implement ScopableEntity for secure ORM support
// This is a multi-tenant entity with tenant isolation
impl ScopableEntity for Entity {
    fn tenant_col() -> Option<Self::Column> {
        // Multi-tenant entity - scope by tenant_id
        Some(Column::TenantId)
    }

    fn id_col() -> Self::Column {
        Column::Id
    }
}
