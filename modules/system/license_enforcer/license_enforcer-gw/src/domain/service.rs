//! Domain service for the license enforcer gateway.
//!
//! Plugin discovery is lazy: resolved on first API call after
//! types-registry is ready.

use std::sync::Arc;
use std::time::Duration;

use license_enforcer_sdk::{
    CachePluginClient, EnabledGlobalFeatures, LicenseCachePluginSpecV1, LicenseFeatureID,
    LicensePlatformPluginSpecV1, PlatformPluginClient,
};
use modkit::client_hub::{ClientHub, ClientScope};
use modkit::gts::BaseModkitPluginV1;
use modkit::plugins::GtsPluginSelector;
use modkit::telemetry::ThrottledLog;
use modkit_security::SecurityContext;
use tracing::info;
use types_registry_sdk::{GtsEntity, ListQuery, TypesRegistryClient};

use super::error::DomainError;

/// Throttle interval for unavailable plugin warnings.
const UNAVAILABLE_LOG_THROTTLE: Duration = Duration::from_secs(10);

/// License enforcer gateway service.
///
/// Discovers platform and cache plugins via types-registry and delegates API calls.
pub struct Service {
    hub: Arc<ClientHub>,
    vendor: String,
    /// Shared selector for platform plugin instance IDs.
    platform_selector: GtsPluginSelector,
    /// Shared selector for cache plugin instance IDs.
    cache_selector: GtsPluginSelector,
    /// Throttle for platform plugin unavailable warnings.
    platform_unavailable_log: ThrottledLog,
    /// Throttle for cache plugin unavailable warnings.
    cache_unavailable_log: ThrottledLog,
}

impl Service {
    /// Creates a new service with lazy plugin resolution.
    #[must_use]
    pub fn new(hub: Arc<ClientHub>, vendor: String) -> Self {
        Self {
            hub,
            vendor,
            platform_selector: GtsPluginSelector::new(),
            cache_selector: GtsPluginSelector::new(),
            platform_unavailable_log: ThrottledLog::new(UNAVAILABLE_LOG_THROTTLE),
            cache_unavailable_log: ThrottledLog::new(UNAVAILABLE_LOG_THROTTLE),
        }
    }

    /// Lazily resolves and returns the platform plugin client.
    async fn get_platform_plugin(&self) -> Result<Arc<dyn PlatformPluginClient>, DomainError> {
        let instance_id = self
            .platform_selector
            .get_or_init(|| self.resolve_platform_plugin())
            .await?;
        let scope = ClientScope::gts_id(instance_id.as_ref());

        if let Some(client) = self.hub.try_get_scoped::<dyn PlatformPluginClient>(&scope) {
            Ok(client)
        } else {
            if self.platform_unavailable_log.should_log() {
                tracing::warn!(
                    plugin_gts_id = %instance_id,
                    vendor = %self.vendor,
                    "Platform plugin client not registered yet"
                );
            }
            Err(DomainError::PlatformPluginUnavailable {
                gts_id: instance_id.to_string(),
                reason: "client not registered yet".into(),
            })
        }
    }

    /// Lazily resolves and returns the cache plugin client.
    async fn get_cache_plugin(&self) -> Result<Arc<dyn CachePluginClient>, DomainError> {
        let instance_id = self
            .cache_selector
            .get_or_init(|| self.resolve_cache_plugin())
            .await?;
        let scope = ClientScope::gts_id(instance_id.as_ref());

        if let Some(client) = self.hub.try_get_scoped::<dyn CachePluginClient>(&scope) {
            Ok(client)
        } else {
            if self.cache_unavailable_log.should_log() {
                tracing::warn!(
                    plugin_gts_id = %instance_id,
                    vendor = %self.vendor,
                    "Cache plugin client not registered yet"
                );
            }
            Err(DomainError::CachePluginUnavailable {
                gts_id: instance_id.to_string(),
                reason: "client not registered yet".into(),
            })
        }
    }

    /// Resolves the platform plugin instance from types-registry.
    #[tracing::instrument(skip_all, fields(vendor = %self.vendor))]
    async fn resolve_platform_plugin(&self) -> Result<String, DomainError> {
        info!("Resolving platform plugin");

        let registry = self
            .hub
            .get::<dyn TypesRegistryClient>()
            .map_err(|e| DomainError::TypesRegistryUnavailable(e.to_string()))?;

        let plugin_type_id = LicensePlatformPluginSpecV1::gts_schema_id().clone();

        let instances = registry
            .list(
                ListQuery::new()
                    .with_pattern(format!("{plugin_type_id}*"))
                    .with_is_type(false),
            )
            .await?;

        let gts_id =
            choose_plugin_instance::<LicensePlatformPluginSpecV1>(&self.vendor, &instances)?;
        info!(plugin_gts_id = %gts_id, "Selected platform plugin instance");

        Ok(gts_id)
    }

    /// Resolves the cache plugin instance from types-registry.
    #[tracing::instrument(skip_all, fields(vendor = %self.vendor))]
    async fn resolve_cache_plugin(&self) -> Result<String, DomainError> {
        info!("Resolving cache plugin");

        let registry = self
            .hub
            .get::<dyn TypesRegistryClient>()
            .map_err(|e| DomainError::TypesRegistryUnavailable(e.to_string()))?;

        let plugin_type_id = LicenseCachePluginSpecV1::gts_schema_id().clone();

        let instances = registry
            .list(
                ListQuery::new()
                    .with_pattern(format!("{plugin_type_id}*"))
                    .with_is_type(false),
            )
            .await?;

        let gts_id = choose_plugin_instance::<LicenseCachePluginSpecV1>(&self.vendor, &instances)?;
        info!(plugin_gts_id = %gts_id, "Selected cache plugin instance");

        Ok(gts_id)
    }

    /// Extract tenant ID from security context.
    ///
    /// # Errors
    ///
    /// Returns error if security context lacks tenant scope
    fn extract_tenant_id(ctx: &SecurityContext) -> Result<uuid::Uuid, DomainError> {
        let tenant_id = ctx.tenant_id();
        if tenant_id.is_nil() {
            return Err(DomainError::MissingTenantScope);
        }
        Ok(tenant_id)
    }

    /// Get enabled global features with cache-aside pattern.
    ///
    /// 1. Validate tenant scope in security context
    /// 2. Try cache plugin (non-blocking on failure)
    /// 3. If cache miss, call platform plugin
    /// 4. Store result in cache (non-blocking on failure)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Security context lacks tenant scope
    /// - Platform plugin is unavailable
    /// - Platform query fails
    #[tracing::instrument(skip_all, fields(tenant_id = tracing::field::Empty))]
    async fn get_tenant_features(
        &self,
        ctx: &SecurityContext,
    ) -> Result<EnabledGlobalFeatures, DomainError> {
        // Validate tenant scope first
        let tenant_id = Self::extract_tenant_id(ctx)?;
        tracing::Span::current().record("tenant_id", tracing::field::display(tenant_id));

        // Try cache first (non-blocking on failure)
        if let Ok(cache) = self.get_cache_plugin().await
            && let Ok(Some(cached)) = cache.get_tenant_features(ctx).await
        {
            tracing::debug!("Cache hit for tenant features");
            return Ok(cached);
        }

        // Cache miss or unavailable, query platform
        tracing::debug!("Cache miss, fetching from platform");
        let platform = self.get_platform_plugin().await?;
        let features = platform.get_enabled_global_features(ctx).await?;

        // Store in cache (non-blocking on failure)
        if let Ok(cache) = self.get_cache_plugin().await {
            let _ = cache.set_tenant_features(ctx, &features).await;
        }

        Ok(features)
    }

    /// Check if a single global feature is enabled for the tenant.
    ///
    /// Uses cache-aside pattern to minimize platform calls.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Security context lacks tenant scope
    /// - Platform plugin is unavailable
    /// - Platform query fails
    #[tracing::instrument(skip_all, fields(
        tenant_id = tracing::field::Empty,
        feature = %feature_id.as_str()
    ))]
    pub async fn is_global_feature_enabled(
        &self,
        ctx: &SecurityContext,
        feature_id: &LicenseFeatureID,
    ) -> Result<bool, DomainError> {
        let tenant_id = Self::extract_tenant_id(ctx)?;
        tracing::Span::current().record("tenant_id", tracing::field::display(tenant_id));

        let features = self.get_tenant_features(ctx).await?;
        Ok(features.contains(feature_id))
    }

    /// List all enabled global features for the tenant.
    ///
    /// Uses cache-aside pattern to minimize platform calls.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Security context lacks tenant scope
    /// - Platform plugin is unavailable
    /// - Platform query fails
    #[tracing::instrument(skip_all, fields(tenant_id = tracing::field::Empty))]
    pub async fn enabled_global_features(
        &self,
        ctx: &SecurityContext,
    ) -> Result<EnabledGlobalFeatures, DomainError> {
        let tenant_id = Self::extract_tenant_id(ctx)?;
        tracing::Span::current().record("tenant_id", tracing::field::display(tenant_id));

        self.get_tenant_features(ctx).await
    }
}

/// Selects the best plugin instance for the given vendor.
///
/// If multiple instances match, the one with lowest priority wins.
#[tracing::instrument(skip_all, fields(vendor, instance_count = instances.len()))]
fn choose_plugin_instance<T>(vendor: &str, instances: &[GtsEntity]) -> Result<String, DomainError>
where
    T: serde::de::DeserializeOwned + gts::GtsSchema,
{
    let mut best: Option<(String, i16)> = None;

    for ent in instances {
        let content: BaseModkitPluginV1<T> =
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

    best.map(|(gts_id, _)| gts_id).ok_or_else(|| {
        // Determine which plugin type based on T
        let type_name = std::any::type_name::<T>();
        if type_name.contains("Platform") {
            DomainError::PlatformPluginNotFound {
                vendor: vendor.to_owned(),
            }
        } else {
            DomainError::CachePluginNotFound {
                vendor: vendor.to_owned(),
            }
        }
    })
}
