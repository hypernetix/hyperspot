use async_trait::async_trait;
use modkit_db::secure::SecureConn;
use modkit_security::{AccessScope, SecurityContext};
use sea_orm::{ActiveValue, sea_query::Expr};
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

        let result = self.db.find::<SettingsEntity>(&scope).one(&self.db).await?;

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

        let scope = AccessScope::both(vec![tenant_id], vec![user_id]);
        let db = self.db.clone();
        let db_for_tx = db.clone();
        let theme_for_tx = theme.clone();
        let language_for_tx = language.clone();

        let active_model = entity::ActiveModel {
            tenant_id: ActiveValue::Set(tenant_id),
            user_id: ActiveValue::Set(user_id),
            theme: ActiveValue::Set(theme.clone()),
            language: ActiveValue::Set(language.clone()),
        };

        // Execute in a transaction to keep read/update/insert atomic.
        db.transaction(move |tx| {
            let db = db_for_tx.clone();
            let scope = scope.clone();
            let active_model = active_model.clone();
            Box::pin(async move {
                // 1) Try update first (full replacement)
                let res = db
                    .update_many::<SettingsEntity>(&scope)
                    .col_expr(entity::Column::Theme, Expr::value(theme_for_tx.clone()))
                    .col_expr(
                        entity::Column::Language,
                        Expr::value(language_for_tx.clone()),
                    )
                    .exec(tx)
                    .await?;

                // 2) If nothing updated, insert
                if res.rows_affected == 0 {
                    let _ = db.insert::<SettingsEntity>(&scope, active_model).await?;
                }

                Ok::<(), anyhow::Error>(())
            })
        })
        .await?;

        Ok(SimpleUserSettings {
            user_id,
            tenant_id,
            theme,
            language,
        })
    }

    async fn upsert_patch(
        &self,
        ctx: &SecurityContext,
        patch: SimpleUserSettingsPatch,
    ) -> anyhow::Result<SimpleUserSettings> {
        let user_id = ctx.subject_id();
        let tenant_id = ctx.tenant_id();

        let scope = AccessScope::both(vec![tenant_id], vec![user_id]);
        let db = self.db.clone();
        let db_for_tx = db.clone();

        let active_model = entity::ActiveModel {
            tenant_id: ActiveValue::Set(tenant_id),
            user_id: ActiveValue::Set(user_id),
            theme: ActiveValue::Set(patch.theme.clone()),
            language: ActiveValue::Set(patch.language.clone()),
        };

        // Execute in a transaction to keep update/insert atomic.
        let patch_theme = patch.theme.clone();
        let patch_language = patch.language.clone();

        let result = db
            .transaction(move |tx| {
                let db = db_for_tx.clone();
                let scope = scope.clone();
                let active_model = active_model.clone();
                Box::pin(async move {
                    let mut upd = db.update_many::<SettingsEntity>(&scope);
                    if let Some(v) = patch_theme.clone() {
                        upd = upd.col_expr(entity::Column::Theme, Expr::value(v));
                    }
                    if let Some(v) = patch_language.clone() {
                        upd = upd.col_expr(entity::Column::Language, Expr::value(v));
                    }

                    let res = upd.exec(tx).await?;
                    if res.rows_affected == 0 {
                        let _ = db.insert::<SettingsEntity>(&scope, active_model).await?;
                    }

                    let row = db
                        .find::<SettingsEntity>(&scope)
                        .one(tx)
                        .await?
                        .map(Into::into)
                        .ok_or_else(|| anyhow::anyhow!("row must exist after successful upsert"))?;

                    Ok::<SimpleUserSettings, anyhow::Error>(row)
                })
            })
            .await?;

        Ok(result)
    }
}
