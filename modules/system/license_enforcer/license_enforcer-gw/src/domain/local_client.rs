//! Local client implementation of the gateway API.

use std::sync::Arc;

use async_trait::async_trait;
use license_enforcer_sdk::{
    EnabledGlobalFeatures, LicenseEnforcerError, LicenseEnforcerGatewayClient, LicenseFeatureID,
};
use modkit_security::SecurityContext;

use super::service::Service;

/// Local implementation of the license enforcer gateway client.
///
/// This adapter wraps the domain service and implements the SDK trait
/// for registration in `ClientHub`.
pub struct LocalClient {
    service: Arc<Service>,
}

impl LocalClient {
    /// Create a new local client wrapping the service.
    #[must_use]
    pub fn new(service: Arc<Service>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl LicenseEnforcerGatewayClient for LocalClient {
    #[tracing::instrument(skip_all, fields(
        tenant_id = tracing::field::Empty,
        feature = %feature_id.as_str()
    ))]
    async fn is_global_feature_enabled(
        &self,
        ctx: &SecurityContext,
        feature_id: &LicenseFeatureID,
    ) -> Result<bool, LicenseEnforcerError> {
        self.service
            .is_global_feature_enabled(ctx, feature_id)
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip_all, fields(tenant_id = tracing::field::Empty))]
    async fn enabled_global_features(
        &self,
        ctx: &SecurityContext,
    ) -> Result<EnabledGlobalFeatures, LicenseEnforcerError> {
        self.service
            .enabled_global_features(ctx)
            .await
            .map_err(Into::into)
    }
}
