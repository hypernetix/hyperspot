//! Cache plugin trait for license enforcement.
//!
//! This trait defines the interface that cache plugins must implement
//! to provide caching for license check results.

use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::error::LicenseEnforcerError;
use crate::models::{LicenseCheckRequest, LicenseCheckResponse};

/// Cache plugin trait for license enforcement.
///
/// Plugins implementing this trait provide caching capabilities for license
/// check results to reduce load on platform integrations.
///
/// Plugins register with `ClientHub` using scoped registration:
/// ```ignore
/// let scope = ClientScope::gts_id(&instance_id);
/// hub.register_scoped::<dyn CachePluginClient>(scope, implementation);
/// ```
#[async_trait]
pub trait CachePluginClient: Send + Sync {
    /// Get cached license check result.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context with tenant information
    /// * `request` - License check request
    ///
    /// # Returns
    ///
    /// Cached response if available, None if cache miss
    ///
    /// # Errors
    ///
    /// Returns error if cache operation fails (should not prevent fallback to platform)
    async fn get(
        &self,
        ctx: &SecurityContext,
        request: &LicenseCheckRequest,
    ) -> Result<Option<LicenseCheckResponse>, LicenseEnforcerError>;

    /// Store license check result in cache.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context with tenant information
    /// * `request` - License check request
    /// * `response` - License check response to cache
    ///
    /// # Errors
    ///
    /// Returns error if cache operation fails (non-fatal)
    async fn set(
        &self,
        ctx: &SecurityContext,
        request: &LicenseCheckRequest,
        response: &LicenseCheckResponse,
    ) -> Result<(), LicenseEnforcerError>;
}
