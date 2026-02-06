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
/// Methods require both `SecurityContext` and explicit tenant ID. The explicit
/// tenant ID parameter ensures callers cannot accidentally use the wrong tenant scope.
///
/// Plugins register with `ClientHub` using scoped registration:
/// ```
/// # use modkit::client_hub::{ClientHub, ClientScope};
/// # use license_enforcer_sdk::PlatformPluginClient;
/// # use std::sync::Arc;
/// # let hub = Arc::new(ClientHub::new());
/// # let instance_id = "gts.x.core.modkit.plugin.v1~x.core.license_resolver.plugin.v1~example";
/// # struct MyPlugin;
/// # #[async_trait::async_trait]
/// # impl PlatformPluginClient for MyPlugin {
/// #     async fn get_enabled_global_features(&self, _ctx: &modkit_security::SecurityContext, _tenant_id: uuid::Uuid) -> Result<license_enforcer_sdk::EnabledGlobalFeatures, license_enforcer_sdk::LicenseEnforcerError> { Ok(license_enforcer_sdk::EnabledGlobalFeatures::new()) }
/// # }
/// # let implementation = Arc::new(MyPlugin);
/// let scope = ClientScope::gts_id(instance_id);
/// hub.register_scoped::<dyn PlatformPluginClient>(scope, implementation);
/// ```
#[async_trait]
pub trait PlatformPluginClient: Send + Sync {
    /// Get the enabled global features for the tenant.
    ///
    /// Returns the complete set of global features that are enabled for the tenant.
    /// Requires both `SecurityContext` and explicit tenant ID for security.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context for authentication and authorization
    /// * `tenant_id` - Explicit tenant ID to query features for
    ///
    /// # Returns
    ///
    /// Set of enabled global features mapped to `LicenseFeatureID`
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Platform is unavailable
    /// - License data is invalid
    /// - Communication with platform fails
    async fn get_enabled_global_features(
        &self,
        ctx: &SecurityContext,
        tenant_id: uuid::Uuid,
    ) -> Result<EnabledGlobalFeatures, LicenseEnforcerError>;
}
