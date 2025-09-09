use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, QuerySelect, Set, TryIntoModel};
use uuid::Uuid;
use log::info;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub tenant_id: Uuid,
    pub theme: String,
    pub language: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// Data for creating a new user entity
pub struct NewSettingsEntity {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub theme: String,
    pub language: String,
}

/// Data for updating an existing user entity
pub struct UpdateSettingsEntity {
    pub theme: Option<String>,
    pub language: Option<String>,
}

pub struct SettingsRepo;

/// Find a user's settings by user and tenant id
pub async fn find<TX>(tx: &TX, user_id: Uuid, tenant_id: Uuid) -> Result<Option<Model>, DbErr>
where
    TX: ConnectionTrait,
{
    Entity::find_by_id((user_id, tenant_id)).one(tx).await
}

/// Update user's settings
pub async fn update<TX>(
    tx: &TX,
    user_id: Uuid,
    tenant_id: Uuid,
    update_data: UpdateSettingsEntity,
) -> Result<Model, DbErr>
where
    TX: ConnectionTrait,
{
    let mut active_model = ActiveModel {
        user_id: Set(user_id),
        tenant_id: Set(tenant_id),
        ..Default::default()
    };

    let mut update_columns = Vec::with_capacity(2);
    if let Some(theme) = update_data.theme {
        active_model.theme = Set(theme);
        update_columns.push(Column::Theme);
    }

    if let Some(language) = update_data.language {
        active_model.language = Set(language);
        update_columns.push(Column::Language);
    }

    if update_columns.is_empty() {
        return match Entity::find_by_id((user_id, tenant_id)).one(tx).await? {
            Some(data) => Ok(data),
            None => {
                let model: Model = active_model.clone().try_into_model()?; // or: active_model.try_into_model()?
                Ok(model)
            }
        };
    }

    Entity::insert(active_model)
        .on_conflict(
            OnConflict::columns([Column::UserId, Column::TenantId])
                .update_columns(update_columns)
                .to_owned(),
        )
        .exec_with_returning(tx)
        .await
}
