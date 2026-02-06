//! Local (in-process) client for the tenant resolver gateway.

use std::sync::Arc;

use async_trait::async_trait;
use modkit_security::SecurityContext;
use tenant_resolver_sdk::{
    GetAncestorsResponse, GetDescendantsResponse, HierarchyOptions, TenantFilter, TenantId,
    TenantInfo, TenantResolverError, TenantResolverGatewayClient,
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
                tracing::error!(operation = "get_tenant", error = ?e, "tenant_resolver gateway call failed");
                e.into()
            })
    }

    async fn get_tenants(
        &self,
        ctx: &SecurityContext,
        ids: &[TenantId],
        filter: Option<&TenantFilter>,
    ) -> Result<Vec<TenantInfo>, TenantResolverError> {
        self.svc
            .get_tenants(ctx, ids, filter)
            .await
            .map_err(|e: DomainError| {
                tracing::error!(operation = "get_tenants", error = ?e, "tenant_resolver gateway call failed");
                e.into()
            })
    }

    async fn get_ancestors(
        &self,
        ctx: &SecurityContext,
        id: TenantId,
        options: Option<&HierarchyOptions>,
    ) -> Result<GetAncestorsResponse, TenantResolverError> {
        self.svc
            .get_ancestors(ctx, id, options)
            .await
            .map_err(|e: DomainError| {
                tracing::error!(operation = "get_ancestors", error = ?e, "tenant_resolver gateway call failed");
                e.into()
            })
    }

    async fn get_descendants(
        &self,
        ctx: &SecurityContext,
        id: TenantId,
        filter: Option<&TenantFilter>,
        options: Option<&HierarchyOptions>,
        max_depth: Option<u32>,
    ) -> Result<GetDescendantsResponse, TenantResolverError> {
        self.svc
            .get_descendants(ctx, id, filter, options, max_depth)
            .await
            .map_err(|e: DomainError| {
                tracing::error!(operation = "get_descendants", error = ?e, "tenant_resolver gateway call failed");
                e.into()
            })
    }

    async fn is_ancestor(
        &self,
        ctx: &SecurityContext,
        ancestor_id: TenantId,
        descendant_id: TenantId,
        options: Option<&HierarchyOptions>,
    ) -> Result<bool, TenantResolverError> {
        self.svc
            .is_ancestor(ctx, ancestor_id, descendant_id, options)
            .await
            .map_err(|e: DomainError| {
                tracing::error!(operation = "is_ancestor", error = ?e, "tenant_resolver gateway call failed");
                e.into()
            })
    }
}
