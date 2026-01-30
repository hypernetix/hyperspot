//! Client implementation of the cache plugin trait.

use std::sync::Arc;

use async_trait::async_trait;
use license_enforcer_sdk::{
    CachePluginClient, LicenseCheckRequest, LicenseCheckResponse, LicenseEnforcerError,
};
use modkit_security::SecurityContext;

use super::service::Service;

/// Client implementation for no-cache plugin.
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
    async fn get(
        &self,
        ctx: &SecurityContext,
        request: &LicenseCheckRequest,
    ) -> Result<Option<LicenseCheckResponse>, LicenseEnforcerError> {
        self.service.get(ctx, request).await
    }

    async fn set(
        &self,
        ctx: &SecurityContext,
        request: &LicenseCheckRequest,
        response: &LicenseCheckResponse,
    ) -> Result<(), LicenseEnforcerError> {
        self.service.set(ctx, request, response).await
    }
}
