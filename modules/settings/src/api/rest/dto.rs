use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// REST DTO for settings
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SettingsDTO {
    pub theme: String,
    pub language: String,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
}

/// REST DTO for updating a settings
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct UpdateSettingsReq {
    pub theme: Option<String>,
    pub language: Option<String>,
}
