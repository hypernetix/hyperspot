use crate::api::auth;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub theme: String,
    pub language: String,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            user_id: auth::get_user_id(),
            tenant_id: auth::get_tenant_id(),
            theme: String::default(),
            language: String::default(),
        }
    }
}

/// Partial update data for a settings
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SettingsPatch {
    pub theme: Option<String>,
    pub language: Option<String>,
}

impl SettingsPatch {
    pub fn new(theme: Option<String>, language: Option<String>) -> Self {
        Self { theme, language }
    }
}
