use chrono::{DateTime, Utc};
use modkit_db_macros::Scopable;
use sea_orm::entity::prelude::*;
use uuid::Uuid;

/// User entity with multi-tenant isolation.
///
/// This entity demonstrates the use of the `#[derive(Scopable)]` macro
/// for automatic implementation of secure ORM scoping.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "users")]
#[secure(
    tenant_col = "tenant_id",  // Multi-tenant entity - scope by tenant_id
    resource_col = "id",        // Primary resource identifier
    no_owner,                   // No owner-based filtering
    no_type                     // No type-based filtering
)]
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
