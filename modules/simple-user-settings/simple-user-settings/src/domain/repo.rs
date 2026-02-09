use async_trait::async_trait;
use modkit::domain::DomainModel;
use modkit_db::secure::DBRunner;
use modkit_security::AccessScope;
use simple_user_settings_sdk::models::{SimpleUserSettings, SimpleUserSettingsPatch};
use uuid::Uuid;

use super::error::DomainError;

#[async_trait]
pub trait SettingsRepository: Send + Sync
where
    SimpleUserSettings: DomainModel,
    SimpleUserSettingsPatch: DomainModel,
{
    async fn find_by_user<C: DBRunner>(
        &self,
        conn: &C,
        scope: &AccessScope,
    ) -> Result<Option<SimpleUserSettings>, DomainError>;

    async fn upsert_full<C: DBRunner>(
        &self,
        conn: &C,
        scope: &AccessScope,
        user_id: Uuid,
        tenant_id: Uuid,
        theme: Option<String>,
        language: Option<String>,
    ) -> Result<SimpleUserSettings, DomainError>;

    async fn upsert_patch<C: DBRunner>(
        &self,
        conn: &C,
        scope: &AccessScope,
        user_id: Uuid,
        tenant_id: Uuid,
        patch: SimpleUserSettingsPatch,
    ) -> Result<SimpleUserSettings, DomainError>;
}
