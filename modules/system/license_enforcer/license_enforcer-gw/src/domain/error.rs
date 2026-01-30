//! Domain errors for license enforcer gateway.

use license_enforcer_sdk::LicenseEnforcerError;
use thiserror::Error;

/// Domain-level errors for license enforcer gateway operations.
#[derive(Debug, Error)]
pub enum DomainError {
    /// Platform plugin not found for vendor
    #[error("Platform plugin not found for vendor: {vendor}")]
    PlatformPluginNotFound { vendor: String },

    /// Cache plugin not found for vendor
    #[error("Cache plugin not found for vendor: {vendor}")]
    CachePluginNotFound { vendor: String },

    /// Platform plugin unavailable (not registered yet)
    #[error("Platform plugin unavailable: {gts_id} - {reason}")]
    PlatformPluginUnavailable { gts_id: String, reason: String },

    /// Cache plugin unavailable (not registered yet)
    #[error("Cache plugin unavailable: {gts_id} - {reason}")]
    CachePluginUnavailable { gts_id: String, reason: String },

    /// Invalid plugin instance data
    #[error("Invalid plugin instance {gts_id}: {reason}")]
    InvalidPluginInstance { gts_id: String, reason: String },

    /// Types registry unavailable
    #[error("Types registry unavailable: {0}")]
    TypesRegistryUnavailable(String),

    /// Types registry operation error
    #[error("Types registry error: {0}")]
    TypesRegistryError(#[from] types_registry_sdk::TypesRegistryError),

    /// SDK error from plugin
    #[error("Plugin error: {0}")]
    PluginError(#[from] LicenseEnforcerError),
}

/// Convert domain errors to SDK errors for API boundary.
impl From<DomainError> for LicenseEnforcerError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::PlatformPluginNotFound { vendor } => LicenseEnforcerError::platform(
                format!("Platform plugin not found for vendor: {vendor}"),
            ),
            DomainError::CachePluginNotFound { vendor } => {
                LicenseEnforcerError::cache(format!("Cache plugin not found for vendor: {vendor}"))
            }
            DomainError::PlatformPluginUnavailable { gts_id, reason } => {
                LicenseEnforcerError::platform(format!(
                    "Platform plugin unavailable: {gts_id} - {reason}"
                ))
            }
            DomainError::CachePluginUnavailable { gts_id, reason } => LicenseEnforcerError::cache(
                format!("Cache plugin unavailable: {gts_id} - {reason}"),
            ),
            DomainError::InvalidPluginInstance { gts_id, reason } => {
                LicenseEnforcerError::internal(format!(
                    "Invalid plugin instance {gts_id}: {reason}"
                ))
            }
            DomainError::TypesRegistryUnavailable(msg) => {
                LicenseEnforcerError::internal(format!("Types registry unavailable: {msg}"))
            }
            DomainError::TypesRegistryError(err) => {
                LicenseEnforcerError::internal_with_source("Types registry error", err)
            }
            DomainError::PluginError(err) => err,
        }
    }
}
