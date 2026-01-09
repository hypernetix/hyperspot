//! Public models for the settings module.
//!
//! These are transport-agnostic data structures that define the contract
//! between the settings module and its consumers.

use uuid::Uuid;

/// User settings entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub theme: String,
    pub language: String,
}

/// Partial update data for user settings.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SettingsPatch {
    pub theme: Option<String>,
    pub language: Option<String>,
}
