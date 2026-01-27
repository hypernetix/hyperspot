use simple_user_settings_sdk::models::SimpleUserSettings;

use super::entity;

impl From<entity::Model> for SimpleUserSettings {
    fn from(entity: entity::Model) -> Self {
        Self {
            user_id: entity.user_id,
            tenant_id: entity.tenant_id,
            theme: entity.theme,
            language: entity.language,
        }
    }
}
