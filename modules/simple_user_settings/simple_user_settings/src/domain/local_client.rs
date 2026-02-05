use async_trait::async_trait;
use modkit_security::SecurityContext;
use simple_user_settings_sdk::{
    SettingsError, SimpleUserSettings, SimpleUserSettingsClientV1, SimpleUserSettingsPatch,
    SimpleUserSettingsUpdate,
};
use std::sync::Arc;

use crate::domain::repo::SettingsRepository;
use crate::domain::service::Service;

pub struct LocalClient<R: SettingsRepository + 'static> {
    service: Arc<Service<R>>,
}

impl<R: SettingsRepository + 'static> LocalClient<R> {
    #[must_use]
    pub fn new(service: Arc<Service<R>>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<R: SettingsRepository + 'static> SimpleUserSettingsClientV1 for LocalClient<R> {
    async fn get_settings(
        &self,
        ctx: &SecurityContext,
    ) -> Result<SimpleUserSettings, SettingsError> {
        self.service.get_settings(ctx).await.map_err(Into::into)
    }

    async fn update_settings(
        &self,
        ctx: &SecurityContext,
        update: SimpleUserSettingsUpdate,
    ) -> Result<SimpleUserSettings, SettingsError> {
        self.service
            .update_settings(ctx, update)
            .await
            .map_err(Into::into)
    }

    async fn patch_settings(
        &self,
        ctx: &SecurityContext,
        patch: SimpleUserSettingsPatch,
    ) -> Result<SimpleUserSettings, SettingsError> {
        self.service
            .patch_settings(ctx, patch)
            .await
            .map_err(Into::into)
    }
}
