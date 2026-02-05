//! Domain service for Tenant Resolver Gateway.
//!
//! Plugin discovery is lazy: the plugin is resolved on the first API call,
//! after `types_registry` has switched to ready mode.

use std::sync::Arc;
use std::time::Duration;

use modkit::client_hub::{ClientHub, ClientScope};
use modkit::gts::BaseModkitPluginV1;
use modkit::plugins::GtsPluginSelector;
use modkit::telemetry::ThrottledLog;
use modkit_odata::{ODataQuery, Page};
use modkit_security::SecurityContext;
use tenant_resolver_example_sdk::{
    AccessOptions, GetParentsResponse, Tenant, TenantFilter, TenantResolverPluginClientV1,
    TenantResolverPluginSpecV1,
};
use tracing::info;
use types_registry_sdk::{GtsEntity, ListQuery, TypesRegistryClient};

// Note: This example gateway still uses SecurityContext in its public API methods
// because it uses an older SDK with hierarchical tenant model.

use crate::domain::error::DomainError;

/// Throttle interval for unavailable plugin warnings.
const UNAVAILABLE_LOG_THROTTLE: Duration = Duration::from_secs(10);

/// Tenant Resolver Gateway service.
///
/// Holds a reference to `ClientHub` and the configured vendor.
/// Plugin discovery is lazy and cached via `GtsPluginSelector`.
pub struct Service {
    hub: Arc<ClientHub>,
    vendor: String,
    /// Shared selector for plugin instance IDs.
    selector: GtsPluginSelector,
    /// Throttle for plugin unavailable warnings.
    unavailable_log_throttle: ThrottledLog,
}

impl Service {
    /// Creates a new Service with lazy plugin resolution.
    #[must_use]
    pub fn new(hub: Arc<ClientHub>, vendor: String) -> Self {
        Self {
            hub,
            vendor,
            selector: GtsPluginSelector::new(),
            unavailable_log_throttle: ThrottledLog::new(UNAVAILABLE_LOG_THROTTLE),
        }
    }

    /// Lazily resolves and returns the plugin client.
    ///
    /// On first call, queries `types_registry` to find the plugin instance
    /// matching the configured vendor. Result is cached for subsequent calls.
    async fn get_plugin(&self) -> Result<Arc<dyn TenantResolverPluginClientV1>, DomainError> {
        let instance_id = self.selector.get_or_init(|| self.resolve_plugin()).await?;
        let scope = ClientScope::gts_id(instance_id.as_ref());

        if let Some(client) = self
            .hub
            .try_get_scoped::<dyn TenantResolverPluginClientV1>(&scope)
        {
            Ok(client)
        } else {
            if self.unavailable_log_throttle.should_log() {
                tracing::warn!(
                    plugin_gts_id = %instance_id,
                    vendor = %self.vendor,
                    "Plugin client not registered yet"
                );
            }
            Err(DomainError::PluginUnavailable {
                gts_id: instance_id.to_string(),
                reason: "client not registered yet".into(),
            })
        }
    }

    /// Resolves the plugin instance from `types_registry`.
    #[tracing::instrument(skip_all, fields(vendor = %self.vendor))]
    async fn resolve_plugin(&self) -> Result<String, DomainError> {
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

        Ok(gts_id)
    }

    /// Returns the root tenant.
    #[tracing::instrument(skip_all)]
    pub async fn get_root_tenant(&self, ctx: &SecurityContext) -> Result<Tenant, DomainError> {
        let client = self.get_plugin().await?;
        client.get_root_tenant(ctx).await.map_err(DomainError::from)
    }

    /// Lists tenants with cursor-based pagination.
    #[tracing::instrument(skip_all)]
    pub async fn list_tenants(
        &self,
        ctx: &SecurityContext,
        filter: TenantFilter,
        query: ODataQuery,
    ) -> Result<Page<Tenant>, DomainError> {
        let client = self.get_plugin().await?;
        client
            .list_tenants(ctx, filter, query)
            .await
            .map_err(DomainError::from)
    }

    /// Returns all parents of the given tenant.
    #[tracing::instrument(skip_all, fields(tenant.id = %id))]
    pub async fn get_parents(
        &self,
        ctx: &SecurityContext,
        id: &str,
        filter: TenantFilter,
        access_options: AccessOptions,
    ) -> Result<GetParentsResponse, DomainError> {
        let client = self.get_plugin().await?;
        client
            .get_parents(ctx, id, filter, access_options)
            .await
            .map_err(DomainError::from)
    }

    /// Returns all children of the given tenant.
    #[tracing::instrument(skip_all, fields(tenant.id = %id, max_depth))]
    pub async fn get_children(
        &self,
        ctx: &SecurityContext,
        id: &str,
        filter: TenantFilter,
        access_options: AccessOptions,
        max_depth: u32,
    ) -> Result<Vec<Tenant>, DomainError> {
        let client = self.get_plugin().await?;
        client
            .get_children(ctx, id, filter, access_options, max_depth)
            .await
            .map_err(DomainError::from)
    }
}

/// Selects the best plugin instance for the given vendor.
///
/// If multiple instances match, the one with the lowest priority wins.
#[tracing::instrument(skip_all, fields(vendor, instance_count = instances.len()))]
fn choose_plugin_instance(vendor: &str, instances: &[GtsEntity]) -> Result<String, DomainError> {
    // Track best match: (gts_id, priority)
    let mut best: Option<(String, i16)> = None;

    for ent in instances {
        // Deserialize the plugin instance content using the SDK type
        let content: BaseModkitPluginV1<TenantResolverPluginSpecV1> =
            serde_json::from_value(ent.content.clone()).map_err(|e| {
                let content_str = serde_json::to_string_pretty(&ent.content)
                    .unwrap_or_else(|_| "Failed to serialize content for logging".to_owned());
                tracing::error!(
                    gts_id = %ent.gts_id,
                    error = %e,
                    content = %content_str,
                    "Failed to deserialize plugin instance content"
                );
                DomainError::InvalidPluginInstance {
                    gts_id: ent.gts_id.clone(),
                    reason: e.to_string(),
                }
            })?;

        // Ensure the instance content self-identifies with the same full instance id.
        if content.id != ent.gts_id {
            tracing::error!(
                gts_id = %ent.gts_id,
                content_id = %content.id,
                "Plugin instance content.id mismatch with GTS ID"
            );
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
