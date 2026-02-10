//! `SimpleUserSettingsClientV1` trait definition.
//!
//! This trait defines the public API for the settings module (Version 1).
//! All methods require a `SecurityContext` for authorization and access control.

use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::errors::SettingsError;
use crate::models::{SimpleUserSettings, SimpleUserSettingsPatch, SimpleUserSettingsUpdate};

/// Public API trait for the settings module (Version 1).
///
/// This trait is registered in `ClientHub` by the settings module:
/// ```ignore
/// let settings = hub.get::<dyn SimpleUserSettingsClientV1>()?;
/// ```
///
/// All methods require a `SecurityContext` for proper authorization and access control.
#[async_trait]
pub trait SimpleUserSettingsClientV1: Send + Sync {
    /// Get settings for the current user.
    /// Returns default empty values if no settings record exists.
    async fn get_settings(
        &self,
        ctx: &SecurityContext,
    ) -> Result<SimpleUserSettings, SettingsError>;

    /// Update settings with full replacement (POST semantics).
    /// Creates a new record if none exists.
    async fn update_settings(
        &self,
        ctx: &SecurityContext,
        update: SimpleUserSettingsUpdate,
    ) -> Result<SimpleUserSettings, SettingsError>;

    /// Partially update settings (PATCH semantics).
    /// Only updates provided fields. Creates a new record if none exists.
    async fn patch_settings(
        &self,
        ctx: &SecurityContext,
        patch: SimpleUserSettingsPatch,
    ) -> Result<SimpleUserSettings, SettingsError>;
}
