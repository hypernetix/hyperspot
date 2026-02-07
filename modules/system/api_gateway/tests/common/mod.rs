use async_trait::async_trait;
use modkit_security::SecurityContext;
use tenant_resolver_sdk::{
    GetAncestorsResponse, GetDescendantsResponse, HierarchyOptions, TenantFilter, TenantId,
    TenantInfo, TenantRef, TenantResolverError, TenantResolverGatewayClient, TenantStatus,
};

pub struct MockTenantResolver;

#[async_trait]
impl TenantResolverGatewayClient for MockTenantResolver {
    async fn get_tenant(
        &self,
        _ctx: &SecurityContext,
        id: TenantId,
    ) -> std::result::Result<TenantInfo, TenantResolverError> {
        Ok(TenantInfo {
            id,
            name: format!("Tenant {id}"),
            status: TenantStatus::Active,
            tenant_type: None,
            parent_id: None,
            self_managed: false,
        })
    }

    async fn get_tenants(
        &self,
        _ctx: &SecurityContext,
        ids: &[TenantId],
        _filter: Option<&TenantFilter>,
    ) -> std::result::Result<Vec<TenantInfo>, TenantResolverError> {
        Ok(ids
            .iter()
            .map(|id| TenantInfo {
                id: *id,
                name: format!("Tenant {id}"),
                status: TenantStatus::Active,
                tenant_type: None,
                parent_id: None,
                self_managed: false,
            })
            .collect())
    }

    async fn get_ancestors(
        &self,
        _ctx: &SecurityContext,
        id: TenantId,
        _options: Option<&HierarchyOptions>,
    ) -> std::result::Result<GetAncestorsResponse, TenantResolverError> {
        Ok(GetAncestorsResponse {
            tenant: TenantRef {
                id,
                status: TenantStatus::Active,
                tenant_type: None,
                parent_id: None,
                self_managed: false,
            },
            ancestors: vec![],
        })
    }

    async fn get_descendants(
        &self,
        _ctx: &SecurityContext,
        id: TenantId,
        _filter: Option<&TenantFilter>,
        _options: Option<&HierarchyOptions>,
        _max_depth: Option<u32>,
    ) -> std::result::Result<GetDescendantsResponse, TenantResolverError> {
        Ok(GetDescendantsResponse {
            tenant: TenantRef {
                id,
                status: TenantStatus::Active,
                tenant_type: None,
                parent_id: None,
                self_managed: false,
            },
            descendants: vec![],
        })
    }

    async fn is_ancestor(
        &self,
        _ctx: &SecurityContext,
        _ancestor_id: TenantId,
        _descendant_id: TenantId,
        _options: Option<&HierarchyOptions>,
    ) -> std::result::Result<bool, TenantResolverError> {
        Ok(false)
    }
}
