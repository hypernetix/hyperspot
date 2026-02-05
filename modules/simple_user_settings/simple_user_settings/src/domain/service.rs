use std::sync::Arc;

use modkit_db::DBProvider;
use modkit_security::{AccessScope, SecurityContext};
use simple_user_settings_sdk::models::{
    SimpleUserSettings, SimpleUserSettingsPatch, SimpleUserSettingsUpdate,
};

use super::error::DomainError;
use super::fields::SettingsFields;
use super::repo::SettingsRepository;

pub(crate) type DbProvider = DBProvider<modkit_db::DbError>;

// ============================================================================
// Service Configuration
// ============================================================================

pub struct ServiceConfig {
    pub max_field_length: usize,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            max_field_length: 100,
        }
    }
}

// ============================================================================
// Service Implementation
// ============================================================================

pub struct Service<R: SettingsRepository> {
    db: Arc<DbProvider>,
    repo: Arc<R>,
    config: ServiceConfig,
}

impl<R: SettingsRepository> Service<R> {
    pub fn new(db: Arc<DbProvider>, repo: Arc<R>, config: ServiceConfig) -> Self {
        Self { db, repo, config }
    }

    pub async fn get_settings(
        &self,
        ctx: &SecurityContext,
    ) -> Result<SimpleUserSettings, DomainError> {
        let conn = self.db.conn().map_err(DomainError::from)?;
        let scope = build_scope(ctx);

        if let Some(settings) = self.repo.find_by_user(&conn, &scope, ctx).await? {
            Ok(settings)
        } else {
            let user_id = ctx.subject_id();
            let tenant_id = ctx.tenant_id();

            Ok(SimpleUserSettings {
                user_id,
                tenant_id,
                theme: None,
                language: None,
            })
        }
    }

    pub async fn update_settings(
        &self,
        ctx: &SecurityContext,
        update: SimpleUserSettingsUpdate,
    ) -> Result<SimpleUserSettings, DomainError> {
        self.validate_field(SettingsFields::THEME, &update.theme)?;
        self.validate_field(SettingsFields::LANGUAGE, &update.language)?;

        let conn = self.db.conn().map_err(DomainError::from)?;
        let scope = build_scope(ctx);

        let settings = self
            .repo
            .upsert_full(
                &conn,
                &scope,
                ctx,
                Some(update.theme),
                Some(update.language),
            )
            .await?;
        Ok(settings)
    }

    pub async fn patch_settings(
        &self,
        ctx: &SecurityContext,
        patch: SimpleUserSettingsPatch,
    ) -> Result<SimpleUserSettings, DomainError> {
        if let Some(ref theme) = patch.theme {
            self.validate_field(SettingsFields::THEME, theme)?;
        }
        if let Some(ref language) = patch.language {
            self.validate_field(SettingsFields::LANGUAGE, language)?;
        }

        let conn = self.db.conn().map_err(DomainError::from)?;
        let scope = build_scope(ctx);

        let settings = self.repo.upsert_patch(&conn, &scope, ctx, patch).await?;
        Ok(settings)
    }

    fn validate_field(&self, field: &str, value: &str) -> Result<(), DomainError> {
        if value.len() > self.config.max_field_length {
            return Err(DomainError::validation(
                field,
                format!("exceeds maximum length of {}", self.config.max_field_length),
            ));
        }
        Ok(())
    }
}

/// Build an access scope from the security context.
///
/// Settings are scoped to tenant + user (resource).
fn build_scope(ctx: &SecurityContext) -> AccessScope {
    AccessScope::both(vec![ctx.tenant_id()], vec![ctx.subject_id()])
}
