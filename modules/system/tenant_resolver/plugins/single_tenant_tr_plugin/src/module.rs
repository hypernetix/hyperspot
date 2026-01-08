//! Single-tenant resolver plugin module.

use std::sync::Arc;

use async_trait::async_trait;
use hs_tenant_resolver_sdk::{
    AccessOptions, TenantFilter, TenantId, TenantInfo, TenantResolverError,
    TenantResolverPluginClient, TenantResolverPluginSpecV1, TenantStatus,
};
use modkit::client_hub::ClientScope;
use modkit::context::ModuleCtx;
use modkit::gts::BaseModkitPluginV1;
use modkit::Module;
use modkit_security::SecurityContext;
use tracing::info;
use types_registry_sdk::TypesRegistryApi;

/// Hardcoded vendor name for GTS instance registration.
const VENDOR: &str = "hyperspot";

/// Hardcoded priority (higher value = lower priority).
/// Set to 1000 so `static_tr_plugin` (priority 100) wins when both are enabled.
const PRIORITY: i16 = 1000;

/// Single-tenant resolver plugin module.
///
/// Zero-configuration plugin for single-tenant deployments.
/// Returns the tenant from security context as the only accessible tenant.
#[modkit::module(
    name = "single_tenant_tr_plugin",
    deps = ["types_registry"]
)]
pub struct SingleTenantTrPlugin;

impl Default for SingleTenantTrPlugin {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Module for SingleTenantTrPlugin {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        info!("Initializing single_tenant_tr_plugin");

        // Generate plugin instance ID
        let instance_id = TenantResolverPluginSpecV1::gts_make_instance_id(
            "hyperspot.builtin.single_tenant_resolver.plugin.v1",
        );

        // Register plugin instance in types-registry
        let registry = ctx.client_hub().get::<dyn TypesRegistryApi>()?;
        let instance = BaseModkitPluginV1::<TenantResolverPluginSpecV1> {
            id: instance_id.clone(),
            vendor: VENDOR.to_owned(),
            priority: PRIORITY,
            properties: TenantResolverPluginSpecV1,
        };
        let instance_json = serde_json::to_value(&instance)?;

        let _ = registry.register(vec![instance_json]).await?;

        // Create service and register scoped client in ClientHub
        let service = Arc::new(Service);
        let api: Arc<dyn TenantResolverPluginClient> = service;
        ctx.client_hub()
            .register_scoped::<dyn TenantResolverPluginClient>(
                ClientScope::gts_id(&instance_id),
                api,
            );

        info!(
            instance_id = %instance_id,
            vendor = VENDOR,
            priority = PRIORITY,
            "Single-tenant plugin initialized"
        );
        Ok(())
    }
}

/// Single-tenant resolver service.
///
/// Implements `TenantResolverPluginClient` with single-tenant semantics:
/// - Only the tenant from security context is accessible
/// - Cross-tenant access is never allowed
/// - Tenant name is always "Default"
struct Service;

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
                name: "Default".to_owned(),
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
        // First verify target tenant exists
        if target != ctx.tenant_id() {
            return Err(TenantResolverError::TenantNotFound { tenant_id: target });
        }
        // Cross-tenant access not allowed in single-tenant mode.
        // Self-access is handled by gateway, not plugin.
        // If we get here, it means gateway is asking about cross-tenant access,
        // which should return false (denied).
        Ok(false)
    }

    async fn get_accessible_tenants(
        &self,
        _ctx: &SecurityContext,
        _filter: Option<&TenantFilter>,
        _options: Option<&AccessOptions>,
    ) -> Result<Vec<TenantInfo>, TenantResolverError> {
        // Return empty list - gateway adds self-tenant automatically
        Ok(vec![])
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
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
        assert_eq!(info.name, "Default");
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
    async fn can_access_returns_false_for_self() {
        // Plugin doesn't handle self-access (gateway does), but if asked,
        // it returns false for cross-tenant access.
        let service = Service;
        let tenant_id = Uuid::parse_str(TENANT_A).unwrap();
        let ctx = ctx_for_tenant(tenant_id);

        let result = service.can_access(&ctx, tenant_id, None).await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn get_accessible_tenants_returns_empty() {
        let service = Service;
        let tenant_id = Uuid::parse_str(TENANT_A).unwrap();
        let ctx = ctx_for_tenant(tenant_id);

        let result = service.get_accessible_tenants(&ctx, None, None).await;

        assert!(result.is_ok());
        let items = result.unwrap();
        assert!(items.is_empty());
    }
}
