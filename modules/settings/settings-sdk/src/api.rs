//! `SettingsApi` trait definition.
//!
//! This trait defines the public API for the settings module.
//! All methods require a `SecurityContext` for authorization and access control.

use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::errors::SettingsError;
use crate::models::{Settings, SettingsPatch};

/// Public API trait for the settings module.
///
/// This trait can be consumed by other modules via `ClientHub`:
/// ```ignore
/// let client = hub.get::<dyn SettingsApi>()?;
/// let settings = client.get_settings(&ctx).await?;
/// ```
///
/// All methods require a `SecurityContext` for proper authorization and access control.
#[async_trait]
pub trait SettingsApi: Send + Sync {
    /// Get settings for the current user.
    /// Returns default empty values if no settings record exists.
    async fn get_settings(&self, ctx: &SecurityContext) -> Result<Settings, SettingsError>;

    /// Update settings with full replacement (POST semantics).
    /// Creates a new record if none exists.
    async fn update_settings(
        &self,
        ctx: &SecurityContext,
        theme: String,
        language: String,
    ) -> Result<Settings, SettingsError>;

    /// Partially update settings (PATCH semantics).
    /// Only updates provided fields. Creates a new record if none exists.
    async fn patch_settings(
        &self,
        ctx: &SecurityContext,
        patch: SettingsPatch,
    ) -> Result<Settings, SettingsError>;
}
