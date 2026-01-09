use async_trait::async_trait;
use modkit_security::AccessScope;
use settings_sdk::models::{Settings, SettingsPatch};

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn find_by_user(&self, scope: &AccessScope) -> anyhow::Result<Option<Settings>>;

    async fn upsert_full(
        &self,
        scope: &AccessScope,
        theme: String,
        language: String,
    ) -> anyhow::Result<Settings>;

    async fn upsert_patch(
        &self,
        scope: &AccessScope,
        patch: SettingsPatch,
    ) -> anyhow::Result<Settings>;
}
