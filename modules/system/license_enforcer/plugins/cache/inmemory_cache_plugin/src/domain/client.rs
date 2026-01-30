//! Client implementation of the cache plugin trait.

use std::sync::Arc;

use async_trait::async_trait;
use license_enforcer_sdk::{CachePluginClient, EnabledGlobalFeatures, LicenseEnforcerError};
use modkit_security::SecurityContext;

use super::service::Service;

/// Client implementation for in-memory cache plugin.
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
impl CachePluginClient for Client {
    async fn get_tenant_features(
        &self,
        ctx: &SecurityContext,
    ) -> Result<Option<EnabledGlobalFeatures>, LicenseEnforcerError> {
        self.service.get_tenant_features(ctx).await
    }

    async fn set_tenant_features(
        &self,
        ctx: &SecurityContext,
        features: &EnabledGlobalFeatures,
    ) -> Result<(), LicenseEnforcerError> {
        self.service.set_tenant_features(ctx, features).await
    }
}
