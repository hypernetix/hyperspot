use async_trait::async_trait;
use modkit_db::DbConnTrait;
use modkit_db::secure::SecureConn;
use modkit_security::{AccessScope, SecurityContext};
use sea_orm::{
    ActiveValue, DatabaseBackend, EntityTrait,
    sea_query::{Expr, OnConflict, SimpleExpr},
};
use simple_user_settings_sdk::models::{SimpleUserSettings, SimpleUserSettingsPatch};

use crate::domain::repo::SettingsRepository;

use super::entity::{self, Entity as SettingsEntity};

#[must_use]
pub fn new_settings_repository(db: SecureConn) -> Box<dyn SettingsRepository> {
    match db.conn().get_database_backend() {
        DatabaseBackend::MySql => {
            let repo: Box<dyn SettingsRepository> = Box::new(MySqlSettingsRepository { db });
            repo
        }
        DatabaseBackend::Postgres => {
            let repo: Box<dyn SettingsRepository> = Box::new(PgSettingsRepository { db });
            repo
        }
        DatabaseBackend::Sqlite => {
            let repo: Box<dyn SettingsRepository> = Box::new(SqliteSettingsRepository { db });
            repo
        }
    }
}

pub struct PgSettingsRepository {
    db: SecureConn,
}

pub struct MySqlSettingsRepository {
    db: SecureConn,
}

pub struct SqliteSettingsRepository {
    db: SecureConn,
}

#[async_trait]
impl SettingsRepository for PgSettingsRepository {
    async fn find_by_user(
        &self,
        ctx: &SecurityContext,
    ) -> anyhow::Result<Option<SimpleUserSettings>> {
        find_by_user_impl(&self.db, ctx).await
    }

    async fn upsert_full(
        &self,
        ctx: &SecurityContext,
        theme: Option<String>,
        language: Option<String>,
    ) -> anyhow::Result<SimpleUserSettings> {
        upsert_full_impl(&self.db, ctx, theme, language).await
    }

    async fn upsert_patch(
        &self,
        ctx: &SecurityContext,
        patch: SimpleUserSettingsPatch,
    ) -> anyhow::Result<SimpleUserSettings> {
        upsert_patch_impl(&self.db, ctx, patch, coalesce_patch_expr_pg_sqlite).await
    }
}

#[async_trait]
impl SettingsRepository for SqliteSettingsRepository {
    async fn find_by_user(
        &self,
        ctx: &SecurityContext,
    ) -> anyhow::Result<Option<SimpleUserSettings>> {
        find_by_user_impl(&self.db, ctx).await
    }

    async fn upsert_full(
        &self,
        ctx: &SecurityContext,
        theme: Option<String>,
        language: Option<String>,
    ) -> anyhow::Result<SimpleUserSettings> {
        upsert_full_impl(&self.db, ctx, theme, language).await
    }

    async fn upsert_patch(
        &self,
        ctx: &SecurityContext,
        patch: SimpleUserSettingsPatch,
    ) -> anyhow::Result<SimpleUserSettings> {
        upsert_patch_impl(&self.db, ctx, patch, coalesce_patch_expr_pg_sqlite).await
    }
}

#[async_trait]
impl SettingsRepository for MySqlSettingsRepository {
    async fn find_by_user(
        &self,
        ctx: &SecurityContext,
    ) -> anyhow::Result<Option<SimpleUserSettings>> {
        find_by_user_impl(&self.db, ctx).await
    }

    async fn upsert_full(
        &self,
        ctx: &SecurityContext,
        theme: Option<String>,
        language: Option<String>,
    ) -> anyhow::Result<SimpleUserSettings> {
        upsert_full_impl(&self.db, ctx, theme, language).await
    }

    async fn upsert_patch(
        &self,
        ctx: &SecurityContext,
        patch: SimpleUserSettingsPatch,
    ) -> anyhow::Result<SimpleUserSettings> {
        upsert_patch_impl(&self.db, ctx, patch, coalesce_patch_expr_mysql).await
    }
}

async fn find_by_user_impl(
    db: &SecureConn,
    ctx: &SecurityContext,
) -> anyhow::Result<Option<SimpleUserSettings>> {
    let tenant_id = ctx.tenant_id();
    let scope = AccessScope::both(vec![tenant_id], vec![ctx.subject_id()]);

    let result = db.find::<SettingsEntity>(&scope).one(db.conn()).await?;

    Ok(result.map(Into::into))
}

async fn upsert_full_impl(
    db: &SecureConn,
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
        .exec(db.conn())
        .await?;

    Ok(SimpleUserSettings {
        user_id,
        tenant_id,
        theme,
        language,
    })
}

async fn upsert_patch_impl<F>(
    db: &SecureConn,
    ctx: &SecurityContext,
    patch: SimpleUserSettingsPatch,
    coalesce_fn: F,
) -> anyhow::Result<SimpleUserSettings>
where
    F: Fn(entity::Column, Option<&String>) -> SimpleExpr,
{
    let user_id = ctx.subject_id();
    let tenant_id = ctx.tenant_id();

    let theme_expr = coalesce_fn(entity::Column::Theme, patch.theme.as_ref());
    let language_expr = coalesce_fn(entity::Column::Language, patch.language.as_ref());

    let active_model = entity::ActiveModel {
        tenant_id: ActiveValue::Set(tenant_id),
        user_id: ActiveValue::Set(user_id),
        theme: ActiveValue::Set(patch.theme.clone()),
        language: ActiveValue::Set(patch.language.clone()),
    };

    SettingsEntity::insert(active_model)
        .on_conflict(
            OnConflict::columns([entity::Column::TenantId, entity::Column::UserId])
                .value(entity::Column::Theme, theme_expr)
                .value(entity::Column::Language, language_expr)
                .to_owned(),
        )
        .exec(db.conn())
        .await?;

    // NOTE: We need a second query because SeaORM's on_conflict() doesn't support
    // RETURNING clause. Ideally we'd use:
    //   INSERT ... ON CONFLICT ... DO UPDATE ... RETURNING *
    // but this requires raw SQL. The extra SELECT is acceptable since it's a
    // simple primary key lookup and settings updates are infrequent.
    let scope = AccessScope::both(vec![tenant_id], vec![user_id]);
    let result = db
        .find::<SettingsEntity>(&scope)
        .one(db.conn())
        .await?
        .map(Into::into)
        .ok_or_else(|| anyhow::anyhow!("row must exist after successful upsert"))?;

    Ok(result)
}

/// Build COALESCE expression for patch semantics:
/// - If patch value is Some: use the new value
/// - If patch value is None: keep existing value (COALESCE(excluded.col, table.col))
fn coalesce_patch_expr_pg_sqlite<C: sea_orm::sea_query::IntoIden + Copy + 'static>(
    column: C,
    patch_value: Option<&String>,
) -> SimpleExpr {
    let inserted_expr: SimpleExpr =
        Expr::col((sea_orm::sea_query::Alias::new("excluded"), column)).into();

    match patch_value {
        Some(_) => {
            // Patch has a value - use excluded (the new value from INSERT)
            inserted_expr
        }
        None => {
            // Patch is None - keep existing value
            Expr::col((entity::Entity, column)).into()
        }
    }
}

/// Build COALESCE expression for patch semantics:
/// - If patch value is Some: use the new value
/// - If patch value is None: keep existing value (COALESCE(excluded.col, table.col))
fn coalesce_patch_expr_mysql<C: sea_orm::sea_query::IntoIden + Copy + 'static>(
    column: C,
    patch_value: Option<&String>,
) -> SimpleExpr {
    let inserted_expr: SimpleExpr = Expr::cust_with_exprs("VALUES(?)", [Expr::col(column).into()]);

    match patch_value {
        Some(_) => {
            // Patch has a value - use the new value from INSERT
            inserted_expr
        }
        None => {
            // Patch is None - keep existing value
            Expr::col((entity::Entity, column)).into()
        }
    }
}
