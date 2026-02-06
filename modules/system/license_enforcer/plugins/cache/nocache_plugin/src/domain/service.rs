//! Service implementation for no-cache plugin.
//!
//! This is a trivial stub that provides no caching (always cache miss).

use license_enforcer_sdk::{EnabledGlobalFeatures, LicenseEnforcerError};
use modkit_security::SecurityContext;

/// No-cache service.
///
/// Provides no caching - always returns None for gets and succeeds for sets.
/// This is a bootstrap stub implementation.
pub struct Service;

impl Service {
    /// Create a new service.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Get cached tenant features (stub - always returns None).
    ///
    /// # Errors
    ///
    /// This function currently never returns an error but is defined to return
    /// a `Result` for consistency with the plugin trait interface.
    #[allow(clippy::unused_async)]
    pub async fn get_tenant_features(
        &self,
        _ctx: &SecurityContext,
        _tenant_id: uuid::Uuid,
    ) -> Result<Option<EnabledGlobalFeatures>, LicenseEnforcerError> {
        // Stub implementation: always cache miss
        Ok(None)
    }

    /// Set cached tenant features (stub - no-op).
    ///
    /// # Errors
    ///
    /// This function currently never returns an error but is defined to return
    /// a `Result` for consistency with the plugin trait interface.
    #[allow(clippy::unused_async)]
    pub async fn set_tenant_features(
        &self,
        _ctx: &SecurityContext,
        _tenant_id: uuid::Uuid,
        _features: &EnabledGlobalFeatures,
    ) -> Result<(), LicenseEnforcerError> {
        // Stub implementation: no-op
        Ok(())
    }
}

impl Default for Service {
    fn default() -> Self {
        Self::new()
    }
}
