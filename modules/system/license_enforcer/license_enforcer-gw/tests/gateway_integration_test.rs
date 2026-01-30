//! Integration tests for license enforcer gateway wiring.
//!
//! These tests boot the gateway module with bootstrap plugins and verify
//! end-to-end wiring through `ClientHub` and types-registry.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use inmemory_cache_plugin::InMemoryCachePlugin;
use license_enforcer_gw::LicenseEnforcerGateway;
use license_enforcer_sdk::{
    LicenseCachePluginSpecV1, LicenseCheckRequest, LicenseEnforcerGatewayClient, LicenseFeature,
    global_features,
};
use modkit::config::ConfigProvider;
use modkit::{ClientHub, Module, ModuleCtx};
use modkit_security::SecurityContext;
use nocache_plugin::NoCachePlugin;
use serde_json::json;
use static_licenses_plugin::StaticLicensesPlugin;
use tokio_util::sync::CancellationToken;
use types_registry_sdk::{
    GtsEntity, ListQuery, RegisterResult, TypesRegistryClient, TypesRegistryError,
};
use uuid::Uuid;

/// Mock types-registry that tracks schema and instance registrations.
#[derive(Clone)]
struct MockTypesRegistry {
    registered: Arc<Mutex<Vec<serde_json::Value>>>,
}

impl MockTypesRegistry {
    fn new() -> Self {
        Self {
            registered: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_registered(&self) -> Vec<serde_json::Value> {
        self.registered.lock().unwrap().clone()
    }

    fn find_schema(&self, schema_id: &str) -> Option<serde_json::Value> {
        let with_prefix = format!("gts://{schema_id}");
        self.get_registered()
            .into_iter()
            .find(|v| {
                v.get("$id")
                    .and_then(|id| id.as_str())
                    .is_some_and(|id| id == with_prefix || id == schema_id)
            })
    }

    fn find_instance(&self, instance_id: &str) -> Option<serde_json::Value> {
        self.get_registered()
            .into_iter()
            .find(|v| v.get("id").and_then(|id| id.as_str()) == Some(instance_id))
    }
}

#[async_trait::async_trait]
impl TypesRegistryClient for MockTypesRegistry {
    async fn register(
        &self,
        entities: Vec<serde_json::Value>,
    ) -> Result<Vec<RegisterResult>, TypesRegistryError> {
        let mut registered = self.registered.lock().unwrap();
        let mut results = Vec::new();
        for entity in entities {
            registered.push(entity.clone());
            // For the mock, we just return Ok for all entities
            // In a real implementation, this would parse and validate the entity
            results.push(RegisterResult::Ok(GtsEntity {
                id: Uuid::new_v4(),
                gts_id: entity
                    .get("gts_id")
                    .and_then(|v| v.as_str())
                    .or_else(|| entity.get("id").and_then(|v| v.as_str()))
                    .unwrap_or("unknown")
                    .to_owned(),
                segments: vec![],
                is_schema: entity.get("gts_id").is_some(),
                content: entity,
                description: None,
            }));
        }
        Ok(results)
    }

    async fn list(&self, query: ListQuery) -> Result<Vec<GtsEntity>, TypesRegistryError> {
        let registered = self.get_registered();
        let pattern = query.pattern.unwrap_or_default();
        let prefix = pattern.trim_end_matches('*');

        let mut results = Vec::new();
        for entity in registered {
            // Check if this is an instance (has "id" field matching pattern)
            if let Some(id) = entity.get("id").and_then(|v| v.as_str())
                && id.starts_with(prefix)
                && !query.is_type.unwrap_or(false)
            {
                // This is an instance matching the pattern
                // Create a minimal GtsEntity for the test
                let gts_entity = GtsEntity {
                    id: Uuid::new_v5(&Uuid::NAMESPACE_URL, id.as_bytes()),
                    gts_id: id.to_owned(),
                    segments: vec![],
                    is_schema: false,
                    content: entity.clone(),
                    description: None,
                };
                results.push(gts_entity);
            }
        }
        Ok(results)
    }

    async fn get(&self, _gts_id: &str) -> Result<GtsEntity, TypesRegistryError> {
        unimplemented!("get not needed for this test")
    }
}

/// Mock config provider for test modules.
struct MockConfigProvider {
    modules: HashMap<String, serde_json::Value>,
}

impl MockConfigProvider {
    fn new() -> Self {
        let mut modules = HashMap::new();

        // Gateway config
        modules.insert(
            "license_enforcer_gateway".to_owned(),
            json!({
                "config": {
                    "vendor": "hyperspot"
                }
            }),
        );

        // Platform plugin config
        modules.insert(
            "static_licenses_plugin".to_owned(),
            json!({
                "config": {
                    "vendor": "hyperspot",
                    "priority": 100
                }
            }),
        );

        // Cache plugin config
        modules.insert(
            "nocache_plugin".to_owned(),
            json!({
                "config": {
                    "vendor": "hyperspot",
                    "priority": 100
                }
            }),
        );

        Self { modules }
    }
}

impl ConfigProvider for MockConfigProvider {
    fn get_module_config(&self, module_name: &str) -> Option<&serde_json::Value> {
        self.modules.get(module_name)
    }
}

#[tokio::test]
async fn test_gateway_end_to_end_wiring() {
    // Arrange: Set up ClientHub and mock types-registry
    let hub = Arc::new(ClientHub::new());
    let registry = MockTypesRegistry::new();
    hub.register::<dyn TypesRegistryClient>(Arc::new(registry.clone()));

    let config_provider = Arc::new(MockConfigProvider::new());
    let cancel = CancellationToken::new();

    // Initialize gateway module first (registers schemas)
    let gateway_ctx = ModuleCtx::new(
        "license_enforcer_gateway",
        Uuid::new_v4(),
        config_provider.clone(),
        hub.clone(),
        cancel.clone(),
        None,
    );
    let gateway = LicenseEnforcerGateway::default();
    gateway
        .init(&gateway_ctx)
        .await
        .expect("gateway init failed");

    // Assert: Verify both plugin schemas were registered
    let platform_schema_id = LicensePlatformPluginSpecV1::gts_schema_id();
    let cache_schema_id = LicenseCachePluginSpecV1::gts_schema_id();

    assert!(
        registry.find_schema(platform_schema_id.as_ref()).is_some(),
        "Platform plugin schema should be registered"
    );
    assert!(
        registry.find_schema(cache_schema_id.as_ref()).is_some(),
        "Cache plugin schema should be registered"
    );

    // Initialize platform plugin (registers instance and scoped client)
    let platform_ctx = ModuleCtx::new(
        "static_licenses_plugin",
        Uuid::new_v4(),
        config_provider.clone(),
        hub.clone(),
        cancel.clone(),
        None,
    );
    let platform_plugin = StaticLicensesPlugin::default();
    platform_plugin
        .init(&platform_ctx)
        .await
        .expect("platform plugin init failed");

    // Initialize cache plugin (registers instance and scoped client)
    let cache_ctx = ModuleCtx::new(
        "nocache_plugin",
        Uuid::new_v4(),
        config_provider,
        hub.clone(),
        cancel,
        None,
    );
    let cache_plugin = NoCachePlugin::default();
    cache_plugin
        .init(&cache_ctx)
        .await
        .expect("cache plugin init failed");

    // Assert: Verify plugin instances were registered
    let platform_instance_id = LicensePlatformPluginSpecV1::gts_make_instance_id(
        "hyperspot.builtin.static_licenses.integration.plugin.v1",
    );
    let cache_instance_id = LicenseCachePluginSpecV1::gts_make_instance_id(
        "hyperspot.builtin.nocache.cache.plugin.v1",
    );

    assert!(
        registry
            .find_instance(platform_instance_id.as_ref())
            .is_some(),
        "Platform plugin instance should be registered"
    );
    assert!(
        registry.find_instance(cache_instance_id.as_ref()).is_some(),
        "Cache plugin instance should be registered"
    );

    // Act: Get gateway client from ClientHub and make a license check
    let client = hub
        .get::<dyn LicenseEnforcerGatewayClient>()
        .expect("Gateway client should be registered");

    let tenant_id = Uuid::new_v4();
    let ctx = SecurityContext::builder()
        .tenant_id(tenant_id)
        .subject_id(Uuid::new_v4())
        .build();

    let request = LicenseCheckRequest {
        tenant_id,
        feature: LicenseFeature::new("gts.x.core.lic.feat.v1~x.core.test.feature.v1".to_owned()),
    };

    let response = client
        .check_license(&ctx, request)
        .await
        .expect("License check should succeed");

    // Assert: Verify response from platform plugin stub
    assert!(
        response.allowed,
        "Static licenses plugin should allow access"
    );
    assert_eq!(
        response.status,
        license_enforcer_sdk::LicenseStatus::Active,
        "License status should be Active"
    );
}

#[tokio::test]
async fn test_gateway_client_registered_in_hub() {
    // This test verifies that gateway module registers its client in ClientHub
    let hub = Arc::new(ClientHub::new());
    let registry = MockTypesRegistry::new();
    hub.register::<dyn TypesRegistryClient>(Arc::new(registry));

    let mut modules = HashMap::new();
    modules.insert(
        "license_enforcer_gateway".to_owned(),
        json!({
            "config": {
                "vendor": "hyperspot"
            }
        }),
    );
    let config_provider = Arc::new(MockConfigProvider { modules });

    let ctx = ModuleCtx::new(
        "license_enforcer_gateway",
        Uuid::new_v4(),
        config_provider,
        hub.clone(),
        CancellationToken::new(),
        None,
    );

    let gateway = LicenseEnforcerGateway::default();
    gateway.init(&ctx).await.expect("init failed");

    // Assert: Gateway client should be available
    assert!(
        hub.get::<dyn LicenseEnforcerGatewayClient>().is_ok(),
        "Gateway client should be registered in ClientHub"
    );
}

#[test]
fn test_platform_plugin_instance_id_matches_design() {
    let instance_id = LicensePlatformPluginSpecV1::gts_make_instance_id(
        "hyperspot.builtin.static_licenses.integration.plugin.v1",
    );

    assert_eq!(
        instance_id.as_ref(),
        "gts.x.core.modkit.plugin.v1~x.core.license_enforcer.integration.plugin.v1~hyperspot.builtin.static_licenses.integration.plugin.v1",
        "Platform plugin instance ID must match design spec"
    );
}

#[test]
fn test_cache_plugin_instance_id_matches_design() {
    let instance_id = LicenseCachePluginSpecV1::gts_make_instance_id(
        "hyperspot.builtin.nocache.cache.plugin.v1",
    );

    assert_eq!(
        instance_id.as_ref(),
        "gts.x.core.modkit.plugin.v1~x.core.license_enforcer.cache.plugin.v1~hyperspot.builtin.nocache.cache.plugin.v1",
        "Cache plugin instance ID must match design spec"
    );
}
