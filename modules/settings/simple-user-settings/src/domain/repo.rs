use async_trait::async_trait;
use modkit_security::SecurityCtx;
use simple_user_settings_sdk::models::{SimpleUserSettings, SimpleUserSettingsPatch};

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn find_by_user(&self, ctx: &SecurityCtx) -> anyhow::Result<Option<SimpleUserSettings>>;

    async fn upsert_full(
        &self,
        ctx: &SecurityCtx,
        theme: String,
        language: String,
    ) -> anyhow::Result<SimpleUserSettings>;

    async fn upsert_patch(
        &self,
        ctx: &SecurityCtx,
        patch: SimpleUserSettingsPatch,
    ) -> anyhow::Result<SimpleUserSettings>;
}
