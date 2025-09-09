use crate::contract::model::Settings;
use crate::infra::storage::entity::Model as SettingsEntity;

/// Convert a database entity to a contract model (owned version)
impl From<SettingsEntity> for Settings {
    fn from(rhs: SettingsEntity) -> Self {
        Self {
            user_id: rhs.user_id,
            tenant_id: rhs.tenant_id,
            theme: rhs.theme,
            language: rhs.language,
        }
    }
}

/// Convert a database entity to a contract model (by-ref version)
impl From<&SettingsEntity> for Settings {
    fn from(rhs: &SettingsEntity) -> Self {
        Self {
            user_id: rhs.user_id,
            tenant_id: rhs.tenant_id,
            theme: rhs.theme.clone(),
            language: rhs.language.clone(),
        }
    }
}
