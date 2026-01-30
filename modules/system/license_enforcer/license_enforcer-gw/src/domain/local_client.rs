//! Local client implementation of the gateway API.

use std::sync::Arc;

use async_trait::async_trait;
use license_enforcer_sdk::{
    LicenseCheckRequest, LicenseCheckResponse, LicenseEnforcerError, LicenseEnforcerGatewayClient,
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
        tenant_id = %request.tenant_id,
        feature = %request.feature.gts_id
    ))]
    async fn check_license(
        &self,
        ctx: &SecurityContext,
        request: LicenseCheckRequest,
    ) -> Result<LicenseCheckResponse, LicenseEnforcerError> {
        self.service
            .check_license(ctx, request)
            .await
            .map_err(Into::into)
    }
}
