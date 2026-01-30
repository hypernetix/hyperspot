//! Public gateway API trait for license enforcement.
//!
//! This trait defines the public API that consumers use to interact with
//! the license enforcement system. The gateway implementation routes requests
//! to appropriate plugins.

use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::error::LicenseEnforcerError;
use crate::models::{LicenseCheckRequest, LicenseCheckResponse};

/// Public API trait for license enforcement gateway.
///
/// This trait is registered unscoped in `ClientHub` and consumed by other modules.
/// The gateway implementation discovers and delegates to platform and cache plugins.
///
/// # Example
///
/// ```ignore
/// use license_enforcer_sdk::{LicenseEnforcerGatewayClient, LicenseCheckRequest, LicenseFeature};
///
/// let client = hub.get::<dyn LicenseEnforcerGatewayClient>()?;
/// let request = LicenseCheckRequest {
///     tenant_id: ctx.tenant_id(),
///     feature: LicenseFeature::new("gts.x.core.lic.feat.v1~...".to_string()),
/// };
/// let response = client.check_license(&ctx, request).await?;
/// ```
#[async_trait]
pub trait LicenseEnforcerGatewayClient: Send + Sync {
    /// Check if a license allows access to a feature.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context with tenant information
    /// * `request` - License check request with tenant ID and feature
    ///
    /// # Returns
    ///
    /// License check response indicating whether access is allowed
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Platform plugin is unavailable
    /// - Cache operation fails (non-fatal, falls through to platform)
    /// - License validation fails
    async fn check_license(
        &self,
        ctx: &SecurityContext,
        request: LicenseCheckRequest,
    ) -> Result<LicenseCheckResponse, LicenseEnforcerError>;
}
