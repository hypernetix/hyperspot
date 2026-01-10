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
use types_registry_sdk::{GtsEntity, ListQuery, TypesRegistryApi};

use super::error::DomainError;

/// Cached result of plugin resolution.
struct ResolvedPlugin {
    gts_id: String,
    scope: ClientScope,
}

/// Tenant resolver gateway service.
///
/// Discovers plugins via types-registry and delegates API calls.
/// Applies cross-cutting concerns like self-access check.
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
            .get::<dyn TypesRegistryApi>()
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
    /// - `TenantNotFound` if tenant doesn't exist
    /// - Plugin resolution errors
    #[tracing::instrument(skip_all, fields(tenant.id = %id))]
    pub async fn get_tenant(
        &self,
        ctx: &SecurityContext,
        id: TenantId,
    ) -> Result<TenantInfo, DomainError> {
        let plugin = self.get_plugin().await?;
        plugin.get_tenant(ctx, id).await.map_err(DomainError::from)
    }

    /// Check if current tenant can access target tenant.
    ///
    /// Gateway enforces self-access rule before delegating to plugin.
    /// Access rules (including status-based and permission-based) are plugin-determined.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if access is allowed
    /// - `Ok(false)` if target exists but access is denied
    ///
    /// # Errors
    ///
    /// - `TenantNotFound` if target tenant doesn't exist
    pub async fn can_access(
        &self,
        ctx: &SecurityContext,
        target: TenantId,
        options: Option<&AccessOptions>,
    ) -> Result<bool, DomainError> {
        let source = ctx.tenant_id();

        // Gateway: Self-access is always allowed (with no permission check)
        if source == target {
            return Ok(true);
        }

        let plugin = self.get_plugin().await?;
        plugin
            .can_access(ctx, target, options)
            .await
            .map_err(DomainError::from)
    }

    /// Get all tenants accessible by the current tenant.
    ///
    /// Gateway ensures the source tenant is included in the result
    /// (if it matches the filter criteria).
    ///
    /// # Errors
    ///
    /// - Plugin resolution errors
    pub async fn get_accessible_tenants(
        &self,
        ctx: &SecurityContext,
        filter: Option<&TenantFilter>,
        options: Option<&AccessOptions>,
    ) -> Result<Vec<TenantInfo>, DomainError> {
        let source = ctx.tenant_id();

        let plugin = self.get_plugin().await?;
        let mut tenants = plugin
            .get_accessible_tenants(ctx, filter, options)
            .await
            .map_err(DomainError::from)?;

        // Gateway: Ensure self-tenant is included (if it matches filter)
        if !tenants.iter().any(|t| t.id == source) {
            // Try to get source tenant info
            if let Ok(self_info) = plugin.get_tenant(ctx, source).await {
                // Check if it matches filter criteria
                if matches_filter(&self_info, filter) {
                    tenants.insert(0, self_info);
                }
            }
            // If tenant doesn't exist or doesn't match filter, don't add it
        }

        Ok(tenants)
    }
}

/// Checks if a tenant matches the filter criteria.
fn matches_filter(tenant: &TenantInfo, filter: Option<&TenantFilter>) -> bool {
    let Some(filter) = filter else {
        return true;
    };

    // Check ID filter
    if !filter.id.is_empty() && !filter.id.contains(&tenant.id) {
        return false;
    }

    // Check status filter
    if !filter.status.is_empty() && !filter.status.contains(&tenant.status) {
        return false;
    }

    true
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
