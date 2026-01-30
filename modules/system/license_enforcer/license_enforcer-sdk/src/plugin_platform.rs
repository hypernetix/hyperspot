//! Platform plugin trait for license enforcement.
//!
//! This trait defines the interface that platform integration plugins must implement
//! to provide license data from external systems.

use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::error::LicenseEnforcerError;
use crate::models::{LicenseCheckRequest, LicenseCheckResponse};

/// Platform plugin trait for license enforcement.
///
/// Plugins implementing this trait provide license data from external platforms
/// (e.g., license servers, `SaaS` platforms, static configuration).
///
/// Plugins register with `ClientHub` using scoped registration:
/// ```ignore
/// let scope = ClientScope::gts_id(&instance_id);
/// hub.register_scoped::<dyn PlatformPluginClient>(scope, implementation);
/// ```
#[async_trait]
pub trait PlatformPluginClient: Send + Sync {
    /// Check license status for a feature on the platform.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context with tenant information
    /// * `request` - License check request
    ///
    /// # Returns
    ///
    /// License check response from the platform
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Platform is unavailable
    /// - License data is invalid
    /// - Communication with platform fails
    async fn check_license(
        &self,
        ctx: &SecurityContext,
        request: LicenseCheckRequest,
    ) -> Result<LicenseCheckResponse, LicenseEnforcerError>;
}
