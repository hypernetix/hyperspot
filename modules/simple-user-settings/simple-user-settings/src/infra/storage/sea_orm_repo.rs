use async_trait::async_trait;
use modkit_db::secure::{DBRunner, ScopeError, SecureEntityExt, SecureInsertExt, SecureOnConflict};
use modkit_security::AccessScope;
use sea_orm::{ActiveValue, EntityTrait};
use simple_user_settings_sdk::models::{SimpleUserSettings, SimpleUserSettingsPatch};
use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::repo::SettingsRepository;

use super::entity::{self, Entity as SettingsEntity};

pub struct SeaOrmSettingsRepository;

impl SeaOrmSettingsRepository {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for SeaOrmSettingsRepository {
    fn default() -> Self {
        Self::new()
    }
}

/// Map scope errors to domain errors.
fn map_scope_error(e: ScopeError) -> DomainError {
    match e {
        ScopeError::Denied(msg) => DomainError::forbidden(msg),
        ScopeError::Invalid(msg) => DomainError::internal(format!("scope invalid: {msg}")),
        ScopeError::Db(e) => DomainError::internal(format!("database error: {e}")),
        ScopeError::TenantNotInScope { tenant_id } => {
            DomainError::forbidden(format!("tenant {tenant_id} not in scope"))
        }
    }
}

#[async_trait]
impl SettingsRepository for SeaOrmSettingsRepository {
    async fn find_by_user<C: DBRunner>(
        &self,
        conn: &C,
        scope: &AccessScope,
    ) -> Result<Option<SimpleUserSettings>, DomainError> {
        let result = SettingsEntity::find()
            .secure()
            .scope_with(scope)
            .one(conn)
            .await
            .map_err(map_scope_error)?;

        Ok(result.map(Into::into))
    }

    async fn upsert_full<C: DBRunner>(
        &self,
        conn: &C,
        scope: &AccessScope,
        user_id: Uuid,
        tenant_id: Uuid,
        theme: Option<String>,
        language: Option<String>,
    ) -> Result<SimpleUserSettings, DomainError> {
        let active_model = entity::ActiveModel {
            tenant_id: ActiveValue::Set(tenant_id),
            user_id: ActiveValue::Set(user_id),
            theme: ActiveValue::Set(theme.clone()),
            language: ActiveValue::Set(language.clone()),
        };

        // Full replacement - overwrites all columns (SecureOnConflict validates tenant immutability)
        let on_conflict = SecureOnConflict::<SettingsEntity>::columns([
            entity::Column::TenantId,
            entity::Column::UserId,
        ])
        .update_columns([entity::Column::Theme, entity::Column::Language])
        .map_err(map_scope_error)?;

        SettingsEntity::insert(active_model)
            .secure()
            .scope_with_model(
                scope,
                &entity::ActiveModel {
                    tenant_id: ActiveValue::Set(tenant_id),
                    user_id: ActiveValue::Set(user_id),
                    theme: ActiveValue::Set(theme.clone()),
                    language: ActiveValue::Set(language.clone()),
                },
            )
            .map_err(map_scope_error)?
            .on_conflict(on_conflict)
            .exec(conn)
            .await
            .map_err(map_scope_error)?;

        Ok(SimpleUserSettings {
            user_id,
            tenant_id,
            theme,
            language,
        })
    }

    async fn upsert_patch<C: DBRunner>(
        &self,
        conn: &C,
        scope: &AccessScope,
        user_id: Uuid,
        tenant_id: Uuid,
        patch: SimpleUserSettingsPatch,
    ) -> Result<SimpleUserSettings, DomainError> {
        // Read existing settings to merge with patch
        // This approach is database-agnostic and avoids SQLite COALESCE type issues
        let existing = SettingsEntity::find()
            .secure()
            .scope_with(scope)
            .one(conn)
            .await
            .map_err(map_scope_error)?;

        // Merge patch with existing values
        let (theme, language) = match existing {
            Some(e) => {
                let theme = patch.theme.or(e.theme);
                let language = patch.language.or(e.language);
                (theme, language)
            }
            None => {
                // No existing record - use patch values directly
                (patch.theme, patch.language)
            }
        };

        // Use upsert_full with merged values
        self.upsert_full(conn, scope, user_id, tenant_id, theme, language)
            .await
    }
}
