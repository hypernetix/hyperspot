//! Public models for the settings module.
//!
//! These are transport-agnostic data structures that define the contract
//! between the settings module and its consumers.

use uuid::Uuid;

/// User settings entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleUserSettings {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub theme: Option<String>,
    pub language: Option<String>,
}

/// Partial update data for user settings.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SimpleUserSettingsPatch {
    pub theme: Option<String>,
    pub language: Option<String>,
}

/// Full update data for user settings.
///
/// Unlike `SimpleUserSettingsPatch`, all fields are required and represent a full replacement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleUserSettingsUpdate {
    pub theme: String,
    pub language: String,
}
