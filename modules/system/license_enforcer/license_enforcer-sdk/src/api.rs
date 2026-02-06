//! Public gateway API trait for license enforcement.
//!
//! This trait defines the public API that consumers use to interact with
//! the license enforcement system. The gateway implementation routes requests
//! to appropriate plugins.

use async_trait::async_trait;
use modkit_security::SecurityContext;

use crate::{EnabledGlobalFeatures, error::LicenseEnforcerError, models::LicenseFeatureId};

/// Public API trait for license enforcement gateway.
///
/// This trait is registered unscoped in `ClientHub` and consumed by other modules.
/// The gateway implementation discovers and delegates to platform and cache plugins.
///
/// Methods require both `SecurityContext` and an explicit tenant ID. The explicit
/// tenant ID parameter ensures callers cannot accidentally use the wrong tenant scope.
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
/// # let tenant_id = Uuid::new_v4();
/// # let ctx = SecurityContext::builder().tenant_id(tenant_id).subject_id(Uuid::new_v4()).build();
///
/// // Check a single feature
/// let feature = global_features::CyberChatFeature;
/// let enabled = client.is_global_feature_enabled(&ctx, tenant_id, &feature).await?;
///
/// // List all enabled features
/// let features = client.list_enabled_global_features(&ctx, tenant_id).await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait LicenseEnforcerGatewayClient: Send + Sync {
    /// Check if a single global feature is enabled for the tenant.
    ///
    /// Requires both `SecurityContext` and explicit tenant ID for security.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context for authentication and authorization
    /// * `tenant_id` - Explicit tenant ID to check features for
    /// * `feature_id` - Global feature ID to check
    ///
    /// # Returns
    ///
    /// `true` if the feature is enabled, `false` otherwise
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Platform plugin is unavailable
    /// - Platform query fails
    async fn is_global_feature_enabled(
        &self,
        ctx: &SecurityContext,
        tenant_id: uuid::Uuid,
        feature_id: &dyn LicenseFeatureId,
    ) -> Result<bool, LicenseEnforcerError>;

    /// List all enabled global features for the tenant.
    ///
    /// Requires both `SecurityContext` and explicit tenant ID for security.
    ///
    /// Returns the complete set of enabled features without pagination.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context for authentication and authorization
    /// * `tenant_id` - Explicit tenant ID to check features for
    ///
    /// # Returns
    ///
    /// Set of all enabled global features for the tenant
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Platform plugin is unavailable
    /// - Platform query fails
    async fn list_enabled_global_features(
        &self,
        ctx: &SecurityContext,
        tenant_id: uuid::Uuid,
    ) -> Result<EnabledGlobalFeatures, LicenseEnforcerError>;
}
