use async_trait::async_trait;
use modkit_db::secure::SecureConn;
use modkit_security::{AccessScope, SecurityContext};
use sea_orm::{
    sea_query::{Expr, OnConflict, SimpleExpr},
    ActiveValue, EntityTrait,
};
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

        // Full replacement - overwrites all columns
        SettingsEntity::insert(active_model)
            .on_conflict(
                OnConflict::columns([entity::Column::TenantId, entity::Column::UserId])
                    .update_columns([entity::Column::Theme, entity::Column::Language])
                    .to_owned(),
            )
            .exec(self.db.conn())
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

        // Build COALESCE expressions: use patch value if provided, else keep existing
        // SQL: COALESCE(excluded.theme, settings.theme)
        let theme_expr = coalesce_patch_expr(entity::Column::Theme, patch.theme.as_ref());
        let language_expr = coalesce_patch_expr(entity::Column::Language, patch.language.as_ref());

        let active_model = entity::ActiveModel {
            tenant_id: ActiveValue::Set(tenant_id),
            user_id: ActiveValue::Set(user_id),
            theme: ActiveValue::Set(patch.theme.clone()),
            language: ActiveValue::Set(patch.language.clone()),
        };

        // Single atomic upsert with COALESCE - no read-before-write needed
        SettingsEntity::insert(active_model)
            .on_conflict(
                OnConflict::columns([entity::Column::TenantId, entity::Column::UserId])
                    .value(entity::Column::Theme, theme_expr)
                    .value(entity::Column::Language, language_expr)
                    .to_owned(),
            )
            .exec(self.db.conn())
            .await?;

        // NOTE: We need a second query because SeaORM's on_conflict() doesn't support
        // RETURNING clause. Ideally we'd use:
        //   INSERT ... ON CONFLICT ... DO UPDATE ... RETURNING *
        // but this requires raw SQL. The extra SELECT is acceptable since it's a
        // simple primary key lookup and settings updates are infrequent.
        let scope = AccessScope::both(vec![tenant_id], vec![user_id]);
        let result = self
            .db
            .find::<SettingsEntity>(&scope)
            .one(self.db.conn())
            .await?
            .map(Into::into)
            .ok_or_else(|| anyhow::anyhow!("row must exist after successful upsert"))?;

        Ok(result)
    }
}

/// Build COALESCE expression for patch semantics:
/// - If patch value is Some: use the new value
/// - If patch value is None: keep existing value (COALESCE(excluded.col, table.col))
fn coalesce_patch_expr<C: sea_orm::sea_query::IntoIden + Copy + 'static>(
    column: C,
    patch_value: Option<&String>,
) -> SimpleExpr {
    match patch_value {
        Some(_) => {
            // Patch has a value - use excluded (the new value from INSERT)
            Expr::col((sea_orm::sea_query::Alias::new("excluded"), column)).into()
        }
        None => {
            // Patch is None - keep existing value using COALESCE
            // COALESCE(excluded.column, settings.column)
            Expr::cust_with_exprs(
                "COALESCE($1, $2)",
                [
                    Expr::col((sea_orm::sea_query::Alias::new("excluded"), column)).into(),
                    Expr::col((entity::Entity, column)).into(),
                ],
            )
        }
    }
}
