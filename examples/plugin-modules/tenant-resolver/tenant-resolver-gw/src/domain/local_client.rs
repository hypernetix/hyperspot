use std::sync::Arc;

use async_trait::async_trait;
use modkit_odata::{ODataQuery, Page};
use modkit_security::SecurityContext;
use tenant_resolver_example_sdk::{
    GetParentsResponse, Tenant, TenantFilter, TenantResolverClient, TenantResolverError,
};

use crate::domain::error::DomainError;
use crate::domain::service::Service;

/// Local (in-process) client for the tenant resolver gateway.
///
/// Registered in `ClientHub` by the gateway module during `init()`.
pub struct TenantResolverGwClient {
    svc: Arc<Service>,
}

impl TenantResolverGwClient {
    #[must_use]
    pub fn new(svc: Arc<Service>) -> Self {
        Self { svc }
    }
}

#[async_trait]
impl TenantResolverClient for TenantResolverGwClient {
    async fn get_root_tenant(&self, ctx: &SecurityContext) -> Result<Tenant, TenantResolverError> {
        self.svc
            .get_root_tenant(ctx)
            .await
            .map_err(|e: DomainError| {
                tracing::error!(error = ?e, "tenant_resolver_gateway call failed");
                e.into()
            })
    }

    async fn list_tenants(
        &self,
        ctx: &SecurityContext,
        filter: TenantFilter,
        query: ODataQuery,
    ) -> Result<Page<Tenant>, TenantResolverError> {
        self.svc
            .list_tenants(ctx, filter, query)
            .await
            .map_err(|e: DomainError| {
                tracing::error!(error = ?e, "tenant_resolver_gateway call failed");
                e.into()
            })
    }

    async fn get_parents(
        &self,
        ctx: &SecurityContext,
        id: &str,
        filter: TenantFilter,
        access_options: tenant_resolver_example_sdk::AccessOptions,
    ) -> Result<GetParentsResponse, TenantResolverError> {
        self.svc
            .get_parents(ctx, id, filter, access_options)
            .await
            .map_err(|e: DomainError| {
                tracing::error!(error = ?e, "tenant_resolver_gateway call failed");
                e.into()
            })
    }

    async fn get_children(
        &self,
        ctx: &SecurityContext,
        id: &str,
        filter: TenantFilter,
        access_options: tenant_resolver_example_sdk::AccessOptions,
        max_depth: u32,
    ) -> Result<Vec<Tenant>, TenantResolverError> {
        self.svc
            .get_children(ctx, id, filter, access_options, max_depth)
            .await
            .map_err(|e: DomainError| {
                tracing::error!(error = ?e, "tenant_resolver_gateway call failed");
                e.into()
            })
    }
}
