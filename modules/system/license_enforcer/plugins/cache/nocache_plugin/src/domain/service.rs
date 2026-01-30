//! Service implementation for no-cache plugin.
//!
//! This is a trivial stub that provides no caching (always cache miss).

use license_enforcer_sdk::{
    LicenseCheckRequest, LicenseCheckResponse, LicenseEnforcerError,
};
use modkit_security::SecurityContext;

/// No-cache service.
///
/// Provides no caching - always returns None for `get()` and succeeds for `set()`.
/// This is a bootstrap stub implementation.
pub struct Service;

impl Service {
    /// Create a new service.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Get cached value (stub - always returns None).
    ///
    /// # Errors
    ///
    /// This stub implementation never returns errors
    #[allow(clippy::unused_async)]
    pub async fn get(
        &self,
        _ctx: &SecurityContext,
        _request: &LicenseCheckRequest,
    ) -> Result<Option<LicenseCheckResponse>, LicenseEnforcerError> {
        // Stub implementation: always cache miss
        Ok(None)
    }

    /// Set cached value (stub - no-op).
    ///
    /// # Errors
    ///
    /// This stub implementation never returns errors
    #[allow(clippy::unused_async)]
    pub async fn set(
        &self,
        _ctx: &SecurityContext,
        _request: &LicenseCheckRequest,
        _response: &LicenseCheckResponse,
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
