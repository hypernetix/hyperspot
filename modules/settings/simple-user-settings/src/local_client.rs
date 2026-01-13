use async_trait::async_trait;
use modkit_security::SecurityCtx;
use simple_user_settings_sdk::{
    SettingsError, SimpleUserSettings, SimpleUserSettingsApi, SimpleUserSettingsPatch,
};
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
impl SimpleUserSettingsApi for LocalClient {
    async fn get_settings(&self, ctx: &SecurityCtx) -> Result<SimpleUserSettings, SettingsError> {
        self.service.get_settings(ctx).await.map_err(Into::into)
    }

    async fn update_settings(
        &self,
        ctx: &SecurityCtx,
        theme: String,
        language: String,
    ) -> Result<SimpleUserSettings, SettingsError> {
        self.service
            .update_settings(ctx, theme, language)
            .await
            .map_err(Into::into)
    }

    async fn patch_settings(
        &self,
        ctx: &SecurityCtx,
        patch: SimpleUserSettingsPatch,
    ) -> Result<SimpleUserSettings, SettingsError> {
        self.service
            .patch_settings(ctx, patch)
            .await
            .map_err(Into::into)
    }
}
