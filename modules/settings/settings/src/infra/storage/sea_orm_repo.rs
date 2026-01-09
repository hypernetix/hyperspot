use async_trait::async_trait;
use modkit_db::secure::SecureConn;
use modkit_security::AccessScope;
use sea_orm::{ActiveModelTrait, ActiveValue};
use settings_sdk::models::{Settings, SettingsPatch};

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
    async fn find_by_user(&self, scope: &AccessScope) -> anyhow::Result<Option<Settings>> {
        let result = self
            .db
            .find::<SettingsEntity>(scope)
            .one(self.db.conn())
            .await?;

        Ok(result.map(Into::into))
    }

    async fn upsert_full(
        &self,
        scope: &AccessScope,
        theme: String,
        language: String,
    ) -> anyhow::Result<Settings> {
        let tenant_id = scope
            .tenant_ids()
            .first()
            .copied()
            .ok_or_else(|| anyhow::anyhow!("No tenant in scope"))?;
        let user_id = scope
            .resource_ids()
            .first()
            .copied()
            .ok_or_else(|| anyhow::anyhow!("No user in scope"))?;

        let existing = self
            .db
            .find::<SettingsEntity>(scope)
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
        scope: &AccessScope,
        patch: SettingsPatch,
    ) -> anyhow::Result<Settings> {
        let tenant_id = scope
            .tenant_ids()
            .first()
            .copied()
            .ok_or_else(|| anyhow::anyhow!("No tenant in scope"))?;
        let user_id = scope
            .resource_ids()
            .first()
            .copied()
            .ok_or_else(|| anyhow::anyhow!("No user in scope"))?;

        let existing = self
            .db
            .find::<SettingsEntity>(scope)
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
