//! Domain service for the tenant resolver gateway.
//!
//! Plugin discovery is lazy: resolved on first API call after
//! types-registry is ready.

use std::sync::Arc;

use hs_tenant_resolver_sdk::{
    AccessOptions, TenantFilter, TenantId, TenantInfo, TenantResolverPluginClient,
    TenantResolverPluginSpecV1,
};
use modkit::client_hub::{ClientHub, ClientScope};
use modkit::gts::BaseModkitPluginV1;
use modkit_security::SecurityContext;
use tokio::sync::OnceCell;
use tracing::info;
use types_registry_sdk::{GtsEntity, ListQuery, TypesRegistryClient};
use uuid::Uuid;

use super::error::DomainError;

/// Cached result of plugin resolution.
struct ResolvedPlugin {
    gts_id: String,
    scope: ClientScope,
}

/// Tenant resolver gateway service.
///
/// Discovers plugins via types-registry and delegates API calls.
pub struct Service {
    hub: Arc<ClientHub>,
    vendor: String,
    /// Lazily resolved plugin (cached after first call).
    resolved: OnceCell<ResolvedPlugin>,
}

impl Service {
    /// Creates a new service with lazy plugin resolution.
    #[must_use]
    pub fn new(hub: Arc<ClientHub>, vendor: String) -> Self {
        Self {
            hub,
            vendor,
            resolved: OnceCell::new(),
        }
    }

    /// Lazily resolves and returns the plugin client.
    async fn get_plugin(&self) -> Result<Arc<dyn TenantResolverPluginClient>, DomainError> {
        let resolved = self
            .resolved
            .get_or_try_init(|| self.resolve_plugin())
            .await?;

        self.hub
            .get_scoped::<dyn TenantResolverPluginClient>(&resolved.scope)
            .map_err(|_| DomainError::PluginClientNotFound {
                gts_id: resolved.gts_id.clone(),
            })
    }

    /// Resolves the plugin instance from types-registry.
    #[tracing::instrument(skip_all, fields(vendor = %self.vendor))]
    async fn resolve_plugin(&self) -> Result<ResolvedPlugin, DomainError> {
        info!("Resolving tenant resolver plugin");

        let registry = self
            .hub
            .get::<dyn TypesRegistryClient>()
            .map_err(|e| DomainError::TypesRegistryUnavailable(e.to_string()))?;

        let plugin_type_id = TenantResolverPluginSpecV1::gts_schema_id().clone();

        let instances = registry
            .list(
                ListQuery::new()
                    .with_pattern(format!("{plugin_type_id}*"))
                    .with_is_type(false),
            )
            .await?;

        let gts_id = choose_plugin_instance(&self.vendor, &instances)?;
        info!(plugin_gts_id = %gts_id, "Selected tenant resolver plugin instance");

        let scope = ClientScope::gts_id(&gts_id);
        Ok(ResolvedPlugin { gts_id, scope })
    }

    /// Get tenant information by ID.
    ///
    /// Returns tenant info regardless of status - the consumer can decide
    /// how to handle different statuses.
    ///
    /// # Errors
    ///
    /// - `Unauthorized` if security context has no tenant
    /// - `TenantNotFound` if tenant doesn't exist
    /// - Plugin resolution errors
    #[tracing::instrument(skip_all, fields(tenant.id = %id))]
    pub async fn get_tenant(
        &self,
        ctx: &SecurityContext,
        id: TenantId,
    ) -> Result<TenantInfo, DomainError> {
        require_tenant_context(ctx)?;
        let plugin = self.get_plugin().await?;
        plugin.get_tenant(ctx, id).await.map_err(DomainError::from)
    }

    /// Check if current tenant can access target tenant.
    ///
    /// Access rules (including self-access, status-based, and permission-based)
    /// are plugin-determined.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if access is allowed
    /// - `Ok(false)` if target exists but access is denied
    ///
    /// # Errors
    ///
    /// - `Unauthorized` if security context has no tenant
    /// - `TenantNotFound` if target tenant doesn't exist
    pub async fn can_access(
        &self,
        ctx: &SecurityContext,
        target: TenantId,
        options: Option<&AccessOptions>,
    ) -> Result<bool, DomainError> {
        require_tenant_context(ctx)?;
        let plugin = self.get_plugin().await?;
        plugin
            .can_access(ctx, target, options)
            .await
            .map_err(DomainError::from)
    }

    /// Get all tenants accessible by the current tenant.
    ///
    /// # Errors
    ///
    /// - `Unauthorized` if security context has no tenant
    /// - Plugin resolution errors
    pub async fn get_accessible_tenants(
        &self,
        ctx: &SecurityContext,
        filter: Option<&TenantFilter>,
        options: Option<&AccessOptions>,
    ) -> Result<Vec<TenantInfo>, DomainError> {
        require_tenant_context(ctx)?;
        let plugin = self.get_plugin().await?;
        plugin
            .get_accessible_tenants(ctx, filter, options)
            .await
            .map_err(DomainError::from)
    }
}

/// Validates that the security context has a tenant ID.
///
/// Returns `Unauthorized` if the context has no tenant (nil UUID).
fn require_tenant_context(ctx: &SecurityContext) -> Result<(), DomainError> {
    if ctx.tenant_id() == Uuid::nil() {
        return Err(DomainError::Unauthorized);
    }
    Ok(())
}

/// Selects the best plugin instance for the given vendor.
///
/// If multiple instances match, the one with lowest priority wins.
#[tracing::instrument(skip_all, fields(vendor, instance_count = instances.len()))]
fn choose_plugin_instance(vendor: &str, instances: &[GtsEntity]) -> Result<String, DomainError> {
    let mut best: Option<(String, i16)> = None;

    for ent in instances {
        let content: BaseModkitPluginV1<TenantResolverPluginSpecV1> =
            serde_json::from_value(ent.content.clone()).map_err(|e| {
                tracing::error!(
                    gts_id = %ent.gts_id,
                    error = %e,
                    "Failed to deserialize plugin instance content"
                );
                DomainError::InvalidPluginInstance {
                    gts_id: ent.gts_id.clone(),
                    reason: e.to_string(),
                }
            })?;

        if content.id != ent.gts_id {
            return Err(DomainError::InvalidPluginInstance {
                gts_id: ent.gts_id.clone(),
                reason: format!(
                    "content.id mismatch: expected {:?}, got {:?}",
                    ent.gts_id, content.id
                ),
            });
        }

        if content.vendor != vendor {
            continue;
        }

        match &best {
            None => best = Some((ent.gts_id.clone(), content.priority)),
            Some((_, cur_priority)) => {
                if content.priority < *cur_priority {
                    best = Some((ent.gts_id.clone(), content.priority));
                }
            }
        }
    }

    best.map(|(gts_id, _)| gts_id)
        .ok_or_else(|| DomainError::PluginNotFound {
            vendor: vendor.to_owned(),
        })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    fn anonymous_ctx() -> SecurityContext {
        SecurityContext::anonymous()
    }

    #[test]
    fn require_tenant_context_rejects_anonymous() {
        let ctx = anonymous_ctx();
        let result = require_tenant_context(&ctx);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::Unauthorized));
    }

    #[test]
    fn require_tenant_context_accepts_valid_tenant() {
        let ctx = SecurityContext::builder().tenant_id(Uuid::new_v4()).build();
        let result = require_tenant_context(&ctx);

        assert!(result.is_ok());
    }

    // Integration tests for Service methods require a mock plugin setup.
    // These tests verify the early-return behavior for anonymous context.

    mod service_anonymous_context {
        use super::*;
        use modkit::client_hub::ClientHub;
        use std::sync::Arc;

        fn create_service() -> Service {
            // Create a minimal Service. Plugin resolution will never be reached
            // because anonymous context check fails first.
            let hub = Arc::new(ClientHub::new());
            Service::new(hub, "test-vendor".to_owned())
        }

        #[tokio::test]
        async fn get_tenant_rejects_anonymous_context() {
            let service = create_service();
            let ctx = anonymous_ctx();
            let tenant_id = Uuid::new_v4();

            let result = service.get_tenant(&ctx, tenant_id).await;

            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), DomainError::Unauthorized));
        }

        #[tokio::test]
        async fn can_access_rejects_anonymous_context() {
            let service = create_service();
            let ctx = anonymous_ctx();
            let target_id = Uuid::new_v4();

            let result = service.can_access(&ctx, target_id, None).await;

            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), DomainError::Unauthorized));
        }

        #[tokio::test]
        async fn get_accessible_tenants_rejects_anonymous_context() {
            let service = create_service();
            let ctx = anonymous_ctx();

            let result = service.get_accessible_tenants(&ctx, None, None).await;

            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), DomainError::Unauthorized));
        }
    }
}
