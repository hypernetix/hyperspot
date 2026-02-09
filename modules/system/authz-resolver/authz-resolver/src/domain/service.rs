//! Domain service for the `AuthZ` resolver.

use std::sync::Arc;
use std::time::Duration;

use authz_resolver_sdk::{
    AuthZResolverPluginClient, AuthZResolverPluginSpecV1, EvaluationRequest, EvaluationResponse,
};
use modkit::client_hub::{ClientHub, ClientScope};
use modkit::gts::BaseModkitPluginV1;
use modkit::plugins::GtsPluginSelector;
use modkit::telemetry::ThrottledLog;
use modkit_macros::domain_model;
use tracing::info;
use types_registry_sdk::{GtsEntity, ListQuery, TypesRegistryClient};

use super::error::DomainError;

/// Throttle interval for unavailable plugin warnings.
const UNAVAILABLE_LOG_THROTTLE: Duration = Duration::from_secs(10);

/// `AuthZ` resolver service.
#[domain_model]
pub struct Service {
    hub: Arc<ClientHub>,
    vendor: String,
    selector: GtsPluginSelector,
    unavailable_log_throttle: ThrottledLog,
}

impl Service {
    #[must_use]
    pub fn new(hub: Arc<ClientHub>, vendor: String) -> Self {
        Self {
            hub,
            vendor,
            selector: GtsPluginSelector::new(),
            unavailable_log_throttle: ThrottledLog::new(UNAVAILABLE_LOG_THROTTLE),
        }
    }

    async fn get_plugin(&self) -> Result<Arc<dyn AuthZResolverPluginClient>, DomainError> {
        let instance_id = self.selector.get_or_init(|| self.resolve_plugin()).await?;
        let scope = ClientScope::gts_id(instance_id.as_ref());

        if let Some(client) = self
            .hub
            .try_get_scoped::<dyn AuthZResolverPluginClient>(&scope)
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

    #[tracing::instrument(skip_all, fields(vendor = %self.vendor))]
    async fn resolve_plugin(&self) -> Result<String, DomainError> {
        info!("Resolving authz_resolver plugin");

        let registry = self
            .hub
            .get::<dyn TypesRegistryClient>()
            .map_err(|e| DomainError::TypesRegistryUnavailable(e.to_string()))?;

        let plugin_type_id = AuthZResolverPluginSpecV1::gts_schema_id().clone();

        let instances = registry
            .list(
                ListQuery::new()
                    .with_pattern(format!("{plugin_type_id}*"))
                    .with_is_type(false),
            )
            .await?;

        let gts_id = choose_plugin_instance(&self.vendor, &instances)?;
        info!(plugin_gts_id = %gts_id, "Selected authz_resolver plugin instance");

        Ok(gts_id)
    }

    /// Evaluate an authorization request via the selected plugin.
    ///
    /// # Errors
    ///
    /// - Plugin resolution errors
    /// - Plugin evaluation errors
    #[tracing::instrument(skip_all)]
    pub async fn evaluate(
        &self,
        request: EvaluationRequest,
    ) -> Result<EvaluationResponse, DomainError> {
        let plugin = self.get_plugin().await?;
        plugin.evaluate(request).await.map_err(DomainError::from)
    }
}

#[tracing::instrument(skip_all, fields(vendor, instance_count = instances.len()))]
fn choose_plugin_instance(vendor: &str, instances: &[GtsEntity]) -> Result<String, DomainError> {
    let mut best: Option<(String, i16)> = None;

    for ent in instances {
        let content: BaseModkitPluginV1<AuthZResolverPluginSpecV1> =
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
