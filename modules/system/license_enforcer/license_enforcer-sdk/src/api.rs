//! Public gateway API trait for license enforcement.
//!
//! This trait defines the public API that consumers use to interact with
//! the license enforcement system. The gateway implementation routes requests
//! to appropriate plugins.

use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::error::LicenseEnforcerError;
use crate::models::{EnabledGlobalFeatures, LicenseFeatureID};

/// Public API trait for license enforcement gateway.
///
/// This trait is registered unscoped in `ClientHub` and consumed by other modules.
/// The gateway implementation discovers and delegates to platform and cache plugins.
///
/// The tenant scope is derived exclusively from `SecurityContext`. If the context
/// lacks tenant scope, requests will return an error.
///
/// # Example
///
/// ```
/// # use license_enforcer_sdk::{LicenseEnforcerGatewayClient, global_features};
/// # use modkit::client_hub::ClientHub;
/// # use modkit_security::SecurityContext;
/// # use std::sync::Arc;
/// # use uuid::Uuid;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let hub = Arc::new(ClientHub::new());
/// # // Note: In real usage, the gateway would be registered by the gateway module
/// let client = hub.get::<dyn LicenseEnforcerGatewayClient>()?;
/// # let ctx = SecurityContext::builder().tenant_id(Uuid::new_v4()).subject_id(Uuid::new_v4()).build();
///
/// // Check a single feature
/// let feature = global_features::to_feature_id(global_features::CYBER_CHAT);
/// let enabled = client.is_global_feature_enabled(&ctx, &feature).await?;
///
/// // List all enabled features
/// let features = client.enabled_global_features(&ctx).await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait LicenseEnforcerGatewayClient: Send + Sync {
    /// Check if a single global feature is enabled for the tenant.
    ///
    /// The tenant is derived from `SecurityContext`. If the context lacks tenant
    /// scope, this method returns a missing-tenant error.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context with tenant information
    /// * `feature_id` - Global feature ID to check
    ///
    /// # Returns
    ///
    /// `true` if the feature is enabled, `false` otherwise
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Security context lacks tenant scope
    /// - Platform plugin is unavailable
    /// - Platform query fails
    async fn is_global_feature_enabled(
        &self,
        ctx: &SecurityContext,
        feature_id: &LicenseFeatureID,
    ) -> Result<bool, LicenseEnforcerError>;

    /// List all enabled global features for the tenant.
    ///
    /// The tenant is derived from `SecurityContext`. If the context lacks tenant
    /// scope, this method returns a missing-tenant error.
    ///
    /// Returns the complete set of enabled features without pagination.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context with tenant information
    ///
    /// # Returns
    ///
    /// Set of all enabled global features for the tenant
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Security context lacks tenant scope
    /// - Platform plugin is unavailable
    /// - Platform query fails
    async fn enabled_global_features(
        &self,
        ctx: &SecurityContext,
    ) -> Result<EnabledGlobalFeatures, LicenseEnforcerError>;
}
