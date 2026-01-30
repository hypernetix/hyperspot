//! Service implementation for static licenses plugin.
//!
//! This is a trivial stub that returns fixed data for bootstrap/testing.

use license_enforcer_sdk::{
    LicenseCheckRequest, LicenseCheckResponse, LicenseEnforcerError, LicenseStatus,
};
use modkit_security::SecurityContext;

/// Static licenses service.
///
/// Provides fixed license data. This is a bootstrap stub implementation
/// that always returns Active status for any feature.
pub struct Service;

impl Service {
    /// Create a new service.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Check license (stub implementation).
    ///
    /// Always returns Active status for bootstrap/testing purposes.
    ///
    /// # Errors
    ///
    /// This stub implementation never returns errors
    #[allow(clippy::unused_async)]
    pub async fn check_license(
        &self,
        _ctx: &SecurityContext,
        _request: LicenseCheckRequest,
    ) -> Result<LicenseCheckResponse, LicenseEnforcerError> {
        // Stub implementation: always allow access
        Ok(LicenseCheckResponse {
            allowed: true,
            status: LicenseStatus::Active,
            reason: None,
        })
    }
}

impl Default for Service {
    fn default() -> Self {
        Self::new()
    }
}
