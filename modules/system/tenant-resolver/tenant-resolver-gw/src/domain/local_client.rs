//! Local (in-process) client for the tenant resolver gateway.

use std::sync::Arc;

use async_trait::async_trait;
use modkit_security::SecurityContext;
use tenant_resolver_sdk::{
    AccessOptions, TenantFilter, TenantId, TenantInfo, TenantResolverError,
    TenantResolverGatewayClient,
};

use super::{DomainError, Service};

/// Local client wrapping the gateway service.
///
/// Registered in `ClientHub` by the gateway module during `init()`.
pub struct TenantResolverGwLocalClient {
    svc: Arc<Service>,
}

impl TenantResolverGwLocalClient {
    #[must_use]
    pub fn new(svc: Arc<Service>) -> Self {
        Self { svc }
    }
}

#[async_trait]
impl TenantResolverGatewayClient for TenantResolverGwLocalClient {
    async fn get_tenant(
        &self,
        ctx: &SecurityContext,
        id: TenantId,
    ) -> Result<TenantInfo, TenantResolverError> {
        self.svc
            .get_tenant(ctx, id)
            .await
            .map_err(|e: DomainError| {
                tracing::error!(error = ?e, "tenant_resolver gateway call failed");
                e.into()
            })
    }

    async fn can_access(
        &self,
        ctx: &SecurityContext,
        target: TenantId,
        options: Option<&AccessOptions>,
    ) -> Result<bool, TenantResolverError> {
        self.svc
            .can_access(ctx, target, options)
            .await
            .map_err(|e: DomainError| {
                tracing::error!(error = ?e, "tenant_resolver gateway call failed");
                e.into()
            })
    }

    async fn get_accessible_tenants(
        &self,
        ctx: &SecurityContext,
        filter: Option<&TenantFilter>,
        options: Option<&AccessOptions>,
    ) -> Result<Vec<TenantInfo>, TenantResolverError> {
        self.svc
            .get_accessible_tenants(ctx, filter, options)
            .await
            .map_err(|e: DomainError| {
                tracing::error!(error = ?e, "tenant_resolver gateway call failed");
                e.into()
            })
    }
}
