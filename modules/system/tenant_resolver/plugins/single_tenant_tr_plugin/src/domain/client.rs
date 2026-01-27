//! Client implementation for the single-tenant resolver plugin.
//!
//! Implements `TenantResolverPluginClient` using single-tenant semantics.

use async_trait::async_trait;
use modkit_security::SecurityContext;
use tenant_resolver_sdk::{
    AccessOptions, TenantFilter, TenantId, TenantInfo, TenantResolverError,
    TenantResolverPluginClient, TenantStatus,
};

use super::service::Service;

// Tenant name for single-tenant mode.
const TENANT_NAME: &str = "Default";

#[async_trait]
impl TenantResolverPluginClient for Service {
    async fn get_tenant(
        &self,
        ctx: &SecurityContext,
        id: TenantId,
    ) -> Result<TenantInfo, TenantResolverError> {
        // Only return tenant info if ID matches security context
        if id == ctx.tenant_id() {
            Ok(TenantInfo {
                id,
                name: TENANT_NAME.to_owned(),
                status: TenantStatus::Active,
                tenant_type: None,
            })
        } else {
            Err(TenantResolverError::TenantNotFound { tenant_id: id })
        }
    }

    async fn can_access(
        &self,
        ctx: &SecurityContext,
        target: TenantId,
        _options: Option<&AccessOptions>,
    ) -> Result<bool, TenantResolverError> {
        // In single-tenant mode, only self-access is valid
        if target == ctx.tenant_id() {
            return Ok(true);
        }
        // Other tenants don't exist
        Err(TenantResolverError::TenantNotFound { tenant_id: target })
    }

    async fn get_accessible_tenants(
        &self,
        ctx: &SecurityContext,
        filter: Option<&TenantFilter>,
        _options: Option<&AccessOptions>,
    ) -> Result<Vec<TenantInfo>, TenantResolverError> {
        // Build self-tenant info
        let self_info = TenantInfo {
            id: ctx.tenant_id(),
            name: TENANT_NAME.to_owned(),
            status: TenantStatus::Active,
            tenant_type: None,
        };

        // Apply filter
        if let Some(f) = filter {
            if !f.id.is_empty() && !f.id.contains(&self_info.id) {
                return Ok(vec![]);
            }
            if !f.status.is_empty() && !f.status.contains(&self_info.status) {
                return Ok(vec![]);
            }
        }

        Ok(vec![self_info])
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use tenant_resolver_sdk::TenantStatus;
    use uuid::Uuid;

    fn ctx_for_tenant(tenant_id: Uuid) -> SecurityContext {
        SecurityContext::builder().tenant_id(tenant_id).build()
    }

    const TENANT_A: &str = "11111111-1111-1111-1111-111111111111";
    const TENANT_B: &str = "22222222-2222-2222-2222-222222222222";

    #[tokio::test]
    async fn get_tenant_returns_info_for_matching_id() {
        let service = Service;
        let tenant_id = Uuid::parse_str(TENANT_A).unwrap();
        let ctx = ctx_for_tenant(tenant_id);

        let result = service.get_tenant(&ctx, tenant_id).await;

        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.id, tenant_id);
        assert_eq!(info.name, TENANT_NAME);
        assert_eq!(info.status, TenantStatus::Active);
        assert!(info.tenant_type.is_none());
    }

    #[tokio::test]
    async fn get_tenant_returns_error_for_different_id() {
        let service = Service;
        let ctx_tenant = Uuid::parse_str(TENANT_A).unwrap();
        let query_tenant = Uuid::parse_str(TENANT_B).unwrap();
        let ctx = ctx_for_tenant(ctx_tenant);

        let result = service.get_tenant(&ctx, query_tenant).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            TenantResolverError::TenantNotFound { tenant_id } => {
                assert_eq!(tenant_id, query_tenant);
            }
            other => panic!("Expected TenantNotFound, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn can_access_returns_error_for_different_tenant() {
        let service = Service;
        let ctx_tenant = Uuid::parse_str(TENANT_A).unwrap();
        let target_tenant = Uuid::parse_str(TENANT_B).unwrap();
        let ctx = ctx_for_tenant(ctx_tenant);

        let result = service.can_access(&ctx, target_tenant, None).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            TenantResolverError::TenantNotFound { tenant_id } => {
                assert_eq!(tenant_id, target_tenant);
            }
            other => panic!("Expected TenantNotFound, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn can_access_returns_true_for_self() {
        // Plugin handles self-access: returns true
        let service = Service;
        let tenant_id = Uuid::parse_str(TENANT_A).unwrap();
        let ctx = ctx_for_tenant(tenant_id);

        let result = service.can_access(&ctx, tenant_id, None).await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn get_accessible_tenants_returns_self() {
        let service = Service;
        let tenant_id = Uuid::parse_str(TENANT_A).unwrap();
        let ctx = ctx_for_tenant(tenant_id);

        let result = service.get_accessible_tenants(&ctx, None, None).await;

        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, tenant_id);
        assert_eq!(items[0].name, TENANT_NAME);
        assert_eq!(items[0].status, TenantStatus::Active);
    }
}
