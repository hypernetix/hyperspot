//! Client implementation of the platform plugin trait.

use std::sync::Arc;

use async_trait::async_trait;
use license_enforcer_sdk::{EnabledGlobalFeatures, LicenseEnforcerError, PlatformPluginClient};
use modkit_security::SecurityContext;

use super::service::Service;

/// Client implementation for static licenses plugin.
pub struct Client {
    service: Arc<Service>,
}

impl Client {
    /// Create a new client wrapping the service.
    #[must_use]
    pub fn new(service: Arc<Service>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl PlatformPluginClient for Client {
    async fn get_enabled_global_features(
        &self,
        ctx: &SecurityContext,
    ) -> Result<EnabledGlobalFeatures, LicenseEnforcerError> {
        self.service.get_enabled_global_features(ctx).await
    }
}
