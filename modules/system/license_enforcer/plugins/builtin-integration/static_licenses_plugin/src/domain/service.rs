//! Service implementation for static licenses plugin.
//!
//! This is a trivial stub that returns fixed data for bootstrap/testing.

use license_enforcer_sdk::{
    EnabledGlobalFeatures, LicenseEnforcerError, LicenseFeatureID, global_features,
};
use modkit_security::SecurityContext;

/// Static licenses service.
///
/// Provides fixed license data. This is a bootstrap stub implementation
/// that returns the base global feature for all tenants.
pub struct Service;

impl Service {
    /// Create a new service.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Get enabled global features (stub implementation).
    ///
    /// Always returns the base global feature for bootstrap/testing purposes.
    /// Real implementations would query an external platform.
    ///
    /// # Errors
    ///
    /// Returns error if security context lacks tenant scope
    #[allow(clippy::unused_async)]
    pub async fn get_enabled_global_features(
        &self,
        ctx: &SecurityContext,
    ) -> Result<EnabledGlobalFeatures, LicenseEnforcerError> {
        // Validate tenant scope
        let tenant_id = ctx.tenant_id();
        if tenant_id.is_nil() {
            return Err(LicenseEnforcerError::MissingTenantScope);
        }

        // Stub implementation: return base feature only
        let mut features = EnabledGlobalFeatures::new();
        features.insert(LicenseFeatureID::from(global_features::BASE));

        Ok(features)
    }
}

impl Default for Service {
    fn default() -> Self {
        Self::new()
    }
}
