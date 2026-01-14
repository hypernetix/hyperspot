//! Contoso plugin service implementing `TenantResolverPluginClient`.

use async_trait::async_trait;
use modkit_odata::{ODataQuery, Page, PageInfo};
use modkit_security::SecurityContext;
use tenant_resolver_sdk::{
    AccessOptions, GetParentsResponse, Tenant, TenantFilter, TenantResolverError,
    TenantResolverPluginClient, TenantSpecV1, TenantStatus,
};

/// Contoso plugin service implementing the tenant resolver plugin API.
///
/// This is a stub implementation that only supports `get_root_tenant`.
/// Other methods return appropriate errors or empty results.
pub struct Service;

impl Service {
    /// Returns the hardcoded root tenant for Contoso.
    fn root_tenant() -> Tenant {
        Tenant {
            id: "00000000000000000000000000000000".to_owned(),
            parent_id: String::new(),
            status: TenantStatus::Active,
            r#type: TenantSpecV1::<()>::gts_schema_id().clone(),
            is_accessible_by_parent: true,
        }
    }
}

#[async_trait]
impl TenantResolverPluginClient for Service {
    async fn get_root_tenant(&self, _ctx: &SecurityContext) -> Result<Tenant, TenantResolverError> {
        Ok(Self::root_tenant())
    }

    async fn list_tenants(
        &self,
        _ctx: &SecurityContext,
        filter: TenantFilter,
        query: ODataQuery,
    ) -> Result<Page<Tenant>, TenantResolverError> {
        tracing::debug!(
            limit = query.limit,
            has_cursor = query.cursor.is_some(),
            "Listing tenants (Contoso stub)"
        );
        // Stub: only returns root if it matches filter; cursor is ignored.
        let limit = query.limit.unwrap_or(100);
        let root = Self::root_tenant();
        let items = if filter.matches(root.status) {
            vec![root]
        } else {
            vec![]
        };
        Ok(Page::new(
            items,
            PageInfo {
                next_cursor: None,
                prev_cursor: None,
                limit,
            },
        ))
    }

    async fn get_parents(
        &self,
        _ctx: &SecurityContext,
        id: &str,
        _filter: TenantFilter,
        _access_options: AccessOptions,
    ) -> Result<GetParentsResponse, TenantResolverError> {
        let root = Self::root_tenant();

        // Stub: only root tenant exists, with no parents
        if id == root.id {
            Ok(GetParentsResponse {
                tenant: root,
                parents: vec![],
            })
        } else {
            Err(TenantResolverError::NotFound(id.to_owned()))
        }
    }

    async fn get_children(
        &self,
        _ctx: &SecurityContext,
        id: &str,
        _filter: TenantFilter,
        _access_options: AccessOptions,
        _max_depth: u32,
    ) -> Result<Vec<Tenant>, TenantResolverError> {
        let root = Self::root_tenant();

        // Stub: only root exists, with no children
        if id == root.id {
            Ok(vec![])
        } else {
            Err(TenantResolverError::NotFound(id.to_owned()))
        }
    }
}
