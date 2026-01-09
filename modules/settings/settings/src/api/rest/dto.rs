use serde::{Deserialize, Serialize};
use settings_sdk::models::{Settings, SettingsPatch};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    #[schema(value_type = String)]
    pub user_id: Uuid,
    #[schema(value_type = String)]
    pub tenant_id: Uuid,
    pub theme: String,
    pub language: String,
}

impl From<Settings> for SettingsDto {
    fn from(settings: Settings) -> Self {
        Self {
            user_id: settings.user_id,
            tenant_id: settings.tenant_id,
            theme: settings.theme,
            language: settings.language,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsRequest {
    pub theme: String,
    pub language: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PatchSettingsRequest {
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
}

impl From<PatchSettingsRequest> for SettingsPatch {
    fn from(req: PatchSettingsRequest) -> Self {
        Self {
            theme: req.theme,
            language: req.language,
        }
    }
}
