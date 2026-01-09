use async_trait::async_trait;
use modkit_security::SecurityContext;
use settings_sdk::{Settings, SettingsApi, SettingsError, SettingsPatch};
use std::sync::Arc;

use crate::domain::service::Service;

pub struct LocalClient {
    service: Arc<Service>,
}

impl LocalClient {
    #[must_use]
    pub fn new(service: Arc<Service>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl SettingsApi for LocalClient {
    async fn get_settings(&self, ctx: &SecurityContext) -> Result<Settings, SettingsError> {
        self.service.get_settings(ctx).await.map_err(Into::into)
    }

    async fn update_settings(
        &self,
        ctx: &SecurityContext,
        theme: String,
        language: String,
    ) -> Result<Settings, SettingsError> {
        self.service
            .update_settings(ctx, theme, language)
            .await
            .map_err(Into::into)
    }

    async fn patch_settings(
        &self,
        ctx: &SecurityContext,
        patch: SettingsPatch,
    ) -> Result<Settings, SettingsError> {
        self.service
            .patch_settings(ctx, patch)
            .await
            .map_err(Into::into)
    }
}
