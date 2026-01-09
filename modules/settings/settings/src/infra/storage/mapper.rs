use settings_sdk::models::Settings;

use super::entity;

impl From<entity::Model> for Settings {
    fn from(entity: entity::Model) -> Self {
        Self {
            user_id: entity.user_id,
            tenant_id: entity.tenant_id,
            theme: entity.theme,
            language: entity.language,
        }
    }
}
