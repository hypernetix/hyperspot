use std::sync::Arc;

use modkit_security::SecurityContext;
use simple_user_settings_sdk::models::{SimpleUserSettings, SimpleUserSettingsPatch};

use super::error::DomainError;
use super::fields::SettingsFields;
use super::repo::SettingsRepository;

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

pub struct Service {
    repo: Arc<dyn SettingsRepository>,
    config: ServiceConfig,
}

impl Service {
    pub fn new(repo: Arc<dyn SettingsRepository>, config: ServiceConfig) -> Self {
        Self { repo, config }
    }

    pub async fn get_settings(
        &self,
        ctx: &SecurityContext,
    ) -> Result<SimpleUserSettings, DomainError> {
        if let Some(settings) = self.repo.find_by_user(ctx).await? {
            Ok(settings)
        } else {
            let user_id = ctx.subject_id();
            let tenant_id = ctx.tenant_id();

            Ok(SimpleUserSettings {
                user_id,
                tenant_id,
                theme: String::new(),
                language: String::new(),
            })
        }
    }

    pub async fn update_settings(
        &self,
        ctx: &SecurityContext,
        theme: String,
        language: String,
    ) -> Result<SimpleUserSettings, DomainError> {
        self.validate_field(SettingsFields::THEME, &theme)?;
        self.validate_field(SettingsFields::LANGUAGE, &language)?;

        let settings = self.repo.upsert_full(ctx, theme, language).await?;
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

        let settings = self.repo.upsert_patch(ctx, patch).await?;
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
