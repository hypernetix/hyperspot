//! Cache plugin trait for license enforcement.
//!
//! This trait defines the interface that cache plugins must implement
//! to provide caching for tenant-scoped feature sets.

use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::error::LicenseEnforcerError;
use crate::models::EnabledGlobalFeatures;

/// Cache plugin trait for license enforcement.
///
/// Plugins implementing this trait provide caching capabilities for tenant-scoped
/// feature sets to reduce load on platform integrations.
///
/// Methods require both `SecurityContext` and explicit tenant ID. The cache is
/// keyed by the explicit tenant ID parameter.
///
/// Plugins register with `ClientHub` using scoped registration:
/// ```
/// # use modkit::client_hub::{ClientHub, ClientScope};
/// # use license_enforcer_sdk::CachePluginClient;
/// # use std::sync::Arc;
/// # let hub = Arc::new(ClientHub::new());
/// # let instance_id = "gts.x.core.modkit.plugin.v1~x.core.license_cache.plugin.v1~example";
/// # struct MyPlugin;
/// # #[async_trait::async_trait]
/// # impl CachePluginClient for MyPlugin {
/// #     async fn get_tenant_features(&self, _ctx: &modkit_security::SecurityContext, _tenant_id: uuid::Uuid) -> Result<Option<license_enforcer_sdk::EnabledGlobalFeatures>, license_enforcer_sdk::LicenseEnforcerError> { Ok(None) }
/// #     async fn set_tenant_features(&self, _ctx: &modkit_security::SecurityContext, _tenant_id: uuid::Uuid, _features: &license_enforcer_sdk::EnabledGlobalFeatures) -> Result<(), license_enforcer_sdk::LicenseEnforcerError> { Ok(()) }
/// # }
/// # let implementation = Arc::new(MyPlugin);
/// let scope = ClientScope::gts_id(instance_id);
/// hub.register_scoped::<dyn CachePluginClient>(scope, implementation);
/// ```
#[async_trait]
pub trait CachePluginClient: Send + Sync {
    /// Get cached enabled global features for the tenant.
    ///
    /// Returns the cached feature set if available, or None for a cache miss.
    /// Requires both `SecurityContext` and explicit tenant ID for security.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context for authentication and authorization
    /// * `tenant_id` - Explicit tenant ID to query cache for
    ///
    /// # Returns
    ///
    /// Cached feature set if available, None if cache miss
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Cache operation fails (should not prevent fallback to platform)
    async fn get_tenant_features(
        &self,
        ctx: &SecurityContext,
        tenant_id: uuid::Uuid,
    ) -> Result<Option<EnabledGlobalFeatures>, LicenseEnforcerError>;

    /// Store tenant's enabled global features in cache.
    ///
    /// Caches the complete feature set for the specified tenant.
    /// Requires both `SecurityContext` and explicit tenant ID for security.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context for authentication and authorization
    /// * `tenant_id` - Explicit tenant ID to cache features for
    /// * `features` - Enabled global features to cache
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Cache operation fails (non-fatal)
    async fn set_tenant_features(
        &self,
        ctx: &SecurityContext,
        tenant_id: uuid::Uuid,
        features: &EnabledGlobalFeatures,
    ) -> Result<(), LicenseEnforcerError>;
}
