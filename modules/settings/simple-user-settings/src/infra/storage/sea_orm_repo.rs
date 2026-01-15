use async_trait::async_trait;
use modkit_db::secure::SecureConn;
use modkit_security::{AccessScope, SecurityContext};
use sea_orm::{ActiveModelTrait, ActiveValue};
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
        theme: String,
        language: String,
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
            let active_model = entity::ActiveModel {
                tenant_id: ActiveValue::Unchanged(existing.tenant_id),
                user_id: ActiveValue::Unchanged(existing.user_id),
                theme: ActiveValue::Set(theme),
                language: ActiveValue::Set(language),
            };
            active_model.update(self.db.conn()).await?
        } else {
            let active_model = entity::ActiveModel {
                tenant_id: ActiveValue::Set(tenant_id),
                user_id: ActiveValue::Set(user_id),
                theme: ActiveValue::Set(theme),
                language: ActiveValue::Set(language),
            };
            active_model.insert(self.db.conn()).await?
        };

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
            let active_model = entity::ActiveModel {
                tenant_id: ActiveValue::Unchanged(existing.tenant_id),
                user_id: ActiveValue::Unchanged(existing.user_id),
                theme: if let Some(theme) = patch.theme {
                    ActiveValue::Set(theme)
                } else {
                    ActiveValue::Unchanged(existing.theme)
                },
                language: if let Some(language) = patch.language {
                    ActiveValue::Set(language)
                } else {
                    ActiveValue::Unchanged(existing.language)
                },
            };
            active_model.update(self.db.conn()).await?
        } else {
            let active_model = entity::ActiveModel {
                tenant_id: ActiveValue::Set(tenant_id),
                user_id: ActiveValue::Set(user_id),
                theme: ActiveValue::Set(patch.theme.unwrap_or_default()),
                language: ActiveValue::Set(patch.language.unwrap_or_default()),
            };
            active_model.insert(self.db.conn()).await?
        };

        Ok(model.into())
    }
}
