use async_trait::async_trait;
use modkit_db::secure::SecureConn;
use modkit_security::{AccessScope, SecurityContext};
use sea_orm::{sea_query::OnConflict, ActiveModelTrait, ActiveValue, EntityTrait};
use simple_user_settings_sdk::models::{SimpleUserSettings, SimpleUserSettingsPatch};

use crate::domain::repo::SettingsRepository;

use super::entity::{self, Entity as SettingsEntity};

pub struct SeaOrmSettingsRepository {
    db: SecureConn,
}

impl SeaOrmSettingsRepository {
    #[must_use]
    pub fn new(db: SecureConn) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SettingsRepository for SeaOrmSettingsRepository {
    async fn find_by_user(
        &self,
        ctx: &SecurityContext,
    ) -> anyhow::Result<Option<SimpleUserSettings>> {
        let tenant_id = ctx.tenant_id();
        let scope = AccessScope::both(vec![tenant_id], vec![ctx.subject_id()]);

        let result = self
            .db
            .find::<SettingsEntity>(&scope)
            .one(self.db.conn())
            .await?;

        Ok(result.map(Into::into))
    }

    async fn upsert_full(
        &self,
        ctx: &SecurityContext,
        theme: Option<String>,
        language: Option<String>,
    ) -> anyhow::Result<SimpleUserSettings> {
        let user_id = ctx.subject_id();
        let tenant_id = ctx.tenant_id();

        let active_model = entity::ActiveModel {
            tenant_id: ActiveValue::Set(tenant_id),
            user_id: ActiveValue::Set(user_id),
            theme: ActiveValue::Set(theme.clone()),
            language: ActiveValue::Set(language.clone()),
        };

        SettingsEntity::insert(active_model)
            .on_conflict(
                OnConflict::columns([entity::Column::TenantId, entity::Column::UserId])
                    .update_columns([entity::Column::Theme, entity::Column::Language])
                    .to_owned(),
            )
            .exec(self.db.conn())
            .await?;

        let scope = AccessScope::both(vec![tenant_id], vec![user_id]);
        let model = self
            .db
            .find::<SettingsEntity>(&scope)
            .one(self.db.conn())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Record should exist after upsert"))?;

        Ok(model.into())
    }

    async fn upsert_patch(
        &self,
        ctx: &SecurityContext,
        patch: SimpleUserSettingsPatch,
    ) -> anyhow::Result<SimpleUserSettings> {
        let user_id = ctx.subject_id();
        let tenant_id = ctx.tenant_id();
        let scope = AccessScope::both(vec![tenant_id], vec![user_id]);

        let existing = self
            .db
            .find::<SettingsEntity>(&scope)
            .one(self.db.conn())
            .await?;

        let model = if let Some(existing) = existing {
            let mut columns_to_update = Vec::new();
            let mut active_model = entity::ActiveModel {
                tenant_id: ActiveValue::Set(tenant_id),
                user_id: ActiveValue::Set(user_id),
                theme: ActiveValue::Set(existing.theme.clone()),
                language: ActiveValue::Set(existing.language.clone()),
            };

            if let Some(theme) = patch.theme {
                active_model.theme = ActiveValue::Set(Some(theme));
                columns_to_update.push(entity::Column::Theme);
            }
            if let Some(language) = patch.language {
                active_model.language = ActiveValue::Set(Some(language));
                columns_to_update.push(entity::Column::Language);
            }

            if !columns_to_update.is_empty() {
                SettingsEntity::insert(active_model)
                    .on_conflict(
                        OnConflict::columns([entity::Column::TenantId, entity::Column::UserId])
                            .update_columns(columns_to_update)
                            .to_owned(),
                    )
                    .exec(self.db.conn())
                    .await?;
            }

            self.db
                .find::<SettingsEntity>(&scope)
                .one(self.db.conn())
                .await?
                .ok_or_else(|| anyhow::anyhow!("Record should exist after upsert"))?
        } else {
            let active_model = entity::ActiveModel {
                tenant_id: ActiveValue::Set(tenant_id),
                user_id: ActiveValue::Set(user_id),
                theme: ActiveValue::Set(patch.theme),
                language: ActiveValue::Set(patch.language),
            };
            active_model.insert(self.db.conn()).await?
        };

        Ok(model.into())
    }
}
