//! Local (in-process) client for the tenant resolver module.

use std::sync::Arc;

use async_trait::async_trait;
use modkit_macros::domain_model;
use modkit_security::SecurityContext;
use tenant_resolver_sdk::{
    GetAncestorsOptions, GetAncestorsResponse, GetDescendantsOptions, GetDescendantsResponse,
    GetTenantsOptions, IsAncestorOptions, TenantId, TenantInfo, TenantResolverClient,
    TenantResolverError,
};

use super::{DomainError, Service};

/// Local client wrapping the TR service.
///
/// Registered in `ClientHub` by the TR module during `init()`.
#[domain_model]
pub struct TenantResolverLocalClient {
    svc: Arc<Service>,
}

impl TenantResolverLocalClient {
    #[must_use]
    pub fn new(svc: Arc<Service>) -> Self {
        Self { svc }
    }
}

fn log_and_convert(op: &str, e: DomainError) -> TenantResolverError {
    tracing::error!(operation = op, error = ?e, "tenant-resolver call failed");
    e.into()
}

#[async_trait]
impl TenantResolverClient for TenantResolverLocalClient {
    async fn get_tenant(
        &self,
        ctx: &SecurityContext,
        id: TenantId,
    ) -> Result<TenantInfo, TenantResolverError> {
        self.svc
            .get_tenant(ctx, id)
            .await
            .map_err(|e| log_and_convert("get_tenant", e))
    }

    async fn get_tenants(
        &self,
        ctx: &SecurityContext,
        ids: &[TenantId],
        options: &GetTenantsOptions,
    ) -> Result<Vec<TenantInfo>, TenantResolverError> {
        self.svc
            .get_tenants(ctx, ids, options)
            .await
            .map_err(|e| log_and_convert("get_tenants", e))
    }

    async fn get_ancestors(
        &self,
        ctx: &SecurityContext,
        id: TenantId,
        options: &GetAncestorsOptions,
    ) -> Result<GetAncestorsResponse, TenantResolverError> {
        self.svc
            .get_ancestors(ctx, id, options)
            .await
            .map_err(|e| log_and_convert("get_ancestors", e))
    }

    async fn get_descendants(
        &self,
        ctx: &SecurityContext,
        id: TenantId,
        options: &GetDescendantsOptions,
    ) -> Result<GetDescendantsResponse, TenantResolverError> {
        self.svc
            .get_descendants(ctx, id, options)
            .await
            .map_err(|e| log_and_convert("get_descendants", e))
    }

    async fn is_ancestor(
        &self,
        ctx: &SecurityContext,
        ancestor_id: TenantId,
        descendant_id: TenantId,
        options: &IsAncestorOptions,
    ) -> Result<bool, TenantResolverError> {
        self.svc
            .is_ancestor(ctx, ancestor_id, descendant_id, options)
            .await
            .map_err(|e| log_and_convert("is_ancestor", e))
    }
}
