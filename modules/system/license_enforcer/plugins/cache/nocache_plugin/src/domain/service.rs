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
    /// Returns error if security context lacks tenant scope
    #[allow(clippy::unused_async)]
    pub async fn get_tenant_features(
        &self,
        ctx: &SecurityContext,
    ) -> Result<Option<EnabledGlobalFeatures>, LicenseEnforcerError> {
        // Validate tenant scope
        let tenant_id = ctx.tenant_id();
        if tenant_id.is_nil() {
            return Err(LicenseEnforcerError::MissingTenantScope);
        }

        // Stub implementation: always cache miss
        Ok(None)
    }

    /// Set cached tenant features (stub - no-op).
    ///
    /// # Errors
    ///
    /// Returns error if security context lacks tenant scope
    #[allow(clippy::unused_async)]
    pub async fn set_tenant_features(
        &self,
        ctx: &SecurityContext,
        _features: &EnabledGlobalFeatures,
    ) -> Result<(), LicenseEnforcerError> {
        // Validate tenant scope
        let tenant_id = ctx.tenant_id();
        if tenant_id.is_nil() {
            return Err(LicenseEnforcerError::MissingTenantScope);
        }

        // Stub implementation: no-op
        Ok(())
    }
}

impl Default for Service {
    fn default() -> Self {
        Self::new()
    }
}
