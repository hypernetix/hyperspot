use async_trait::async_trait;
use modkit_security::SecurityContext;
use simple_user_settings_sdk::models::{SimpleUserSettings, SimpleUserSettingsPatch};

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn find_by_user(
        &self,
        ctx: &SecurityContext,
    ) -> anyhow::Result<Option<SimpleUserSettings>>;

    async fn upsert_full(
        &self,
        ctx: &SecurityContext,
        theme: Option<String>,
        language: Option<String>,
    ) -> anyhow::Result<SimpleUserSettings>;

    async fn upsert_patch(
        &self,
        ctx: &SecurityContext,
        patch: SimpleUserSettingsPatch,
    ) -> anyhow::Result<SimpleUserSettings>;
}
