//! `SettingsApi` trait definition.
//!
//! This trait defines the public API for the settings module.
//! All methods require a `SecurityContext` for authorization and access control.

use async_trait::async_trait;
use modkit_security::SecurityCtx;

use crate::errors::SettingsError;
use crate::models::{SimpleUserSettings, SimpleUserSettingsPatch};

/// Public API trait for the settings module.
///
/// This trait can be consumed by other modules via `ClientHub`:
/// ```
/// use simple_user_settings_sdk::SimpleUserSettingsApi;
/// use modkit_security::SecurityCtx;
/// use modkit::ClientHub;
///
/// # fn example(hub: &ClientHub, ctx: &SecurityCtx) -> Result<(), Box<dyn std::error::Error>> {
/// let client = hub.get::<dyn SimpleUserSettingsApi>()?;
/// # Ok(())
/// # }
/// ```
///
/// All methods require a `SecurityCtx` for proper authorization and access control.
#[async_trait]
pub trait SimpleUserSettingsApi: Send + Sync {
    /// Get settings for the current user.
    /// Returns default empty values if no settings record exists.
    async fn get_settings(&self, ctx: &SecurityCtx) -> Result<SimpleUserSettings, SettingsError>;

    /// Update settings with full replacement (POST semantics).
    /// Creates a new record if none exists.
    async fn update_settings(
        &self,
        ctx: &SecurityCtx,
        theme: String,
        language: String,
    ) -> Result<SimpleUserSettings, SettingsError>;

    /// Partially update settings (PATCH semantics).
    /// Only updates provided fields. Creates a new record if none exists.
    async fn patch_settings(
        &self,
        ctx: &SecurityCtx,
        patch: SimpleUserSettingsPatch,
    ) -> Result<SimpleUserSettings, SettingsError>;
}
