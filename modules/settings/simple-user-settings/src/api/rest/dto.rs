use serde::{Deserialize, Serialize};
use simple_user_settings_sdk::models::{SimpleUserSettings, SimpleUserSettingsPatch};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct SimpleUserSettingsDto {
    #[schema(value_type = String)]
    pub user_id: Uuid,
    #[schema(value_type = String)]
    pub tenant_id: Uuid,
    pub theme: String,
    pub language: String,
}

impl From<SimpleUserSettings> for SimpleUserSettingsDto {
    fn from(settings: SimpleUserSettings) -> Self {
        Self {
            user_id: settings.user_id,
            tenant_id: settings.tenant_id,
            theme: settings.theme,
            language: settings.language,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct UpdateSimpleUserSettingsRequest {
    pub theme: String,
    pub language: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct PatchSimpleUserSettingsRequest {
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
}

impl From<PatchSimpleUserSettingsRequest> for SimpleUserSettingsPatch {
    fn from(req: PatchSimpleUserSettingsRequest) -> Self {
        Self {
            theme: req.theme,
            language: req.language,
        }
    }
}
