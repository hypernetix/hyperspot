//! Platform plugin trait for license enforcement.
//!
//! This trait defines the interface that platform integration plugins must implement
//! to provide license data from external systems.

use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::error::LicenseEnforcerError;
use crate::models::EnabledGlobalFeatures;

/// Platform plugin trait for license enforcement.
///
/// Plugins implementing this trait provide license data from external platforms
/// (e.g., license servers, `SaaS` platforms, static configuration).
///
/// The tenant scope is derived exclusively from `SecurityContext`. Plugins should
/// extract the tenant ID from the context and return the full set of enabled
/// global features for that tenant.
///
/// Plugins register with `ClientHub` using scoped registration:
/// ```
/// # use modkit::client_hub::{ClientHub, ClientScope};
/// # use license_enforcer_sdk::PlatformPluginClient;
/// # use std::sync::Arc;
/// # let hub = Arc::new(ClientHub::new());
/// # let instance_id = "gts.x.core.modkit.plugin.v1~x.core.license_enforcer.integration.plugin.v1~example";
/// # struct MyPlugin;
/// # #[async_trait::async_trait]
/// # impl PlatformPluginClient for MyPlugin {
/// #     async fn get_enabled_global_features(&self, _ctx: &modkit_security::SecurityContext) -> Result<license_enforcer_sdk::EnabledGlobalFeatures, license_enforcer_sdk::LicenseEnforcerError> { Ok(license_enforcer_sdk::EnabledGlobalFeatures::new()) }
/// # }
/// # let implementation = Arc::new(MyPlugin);
/// let scope = ClientScope::gts_id(instance_id);
/// hub.register_scoped::<dyn PlatformPluginClient>(scope, implementation);
/// ```
#[async_trait]
pub trait PlatformPluginClient: Send + Sync {
    /// Get the enabled global features for the tenant.
    ///
    /// Returns the complete set of global features that are enabled for the tenant
    /// in the `SecurityContext`. The tenant ID is extracted from the context.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context with tenant information
    ///
    /// # Returns
    ///
    /// Set of enabled global features mapped to `LicenseFeatureID`
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Security context lacks tenant scope
    /// - Platform is unavailable
    /// - License data is invalid
    /// - Communication with platform fails
    async fn get_enabled_global_features(
        &self,
        ctx: &SecurityContext,
    ) -> Result<EnabledGlobalFeatures, LicenseEnforcerError>;
}
