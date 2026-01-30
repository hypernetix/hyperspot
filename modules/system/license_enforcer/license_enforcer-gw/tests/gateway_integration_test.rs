//! Integration tests for license enforcer gateway wiring.
//!
//! These tests boot the gateway module with bootstrap plugins and verify
//! end-to-end wiring through `ClientHub` and types-registry.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use inmemory_cache_plugin::InMemoryCachePlugin;
use license_enforcer_gw::LicenseEnforcerGateway;
use license_enforcer_sdk::{
    LicenseCachePluginSpecV1, LicenseEnforcerGatewayClient, LicensePlatformPluginSpecV1,
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

/// Counting platform plugin that tracks invocations.
#[derive(Clone)]
struct CountingPlatformPlugin {
    call_count: Arc<AtomicUsize>,
}

impl CountingPlatformPlugin {
    fn new() -> Self {
        Self {
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn get_call_count(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl license_enforcer_sdk::PlatformPluginClient for CountingPlatformPlugin {
    async fn get_enabled_global_features(
        &self,
        _ctx: &SecurityContext,
    ) -> Result<
        license_enforcer_sdk::EnabledGlobalFeatures,
        license_enforcer_sdk::LicenseEnforcerError,
    > {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        let mut features = license_enforcer_sdk::EnabledGlobalFeatures::new();
        features.insert(license_enforcer_sdk::global_features::to_feature_id(
            license_enforcer_sdk::global_features::BASE,
        ));
        Ok(features)
    }
}

/// Counting cache plugin that tracks invocations.
#[derive(Clone)]
struct CountingCachePlugin {
    get_call_count: Arc<AtomicUsize>,
    set_call_count: Arc<AtomicUsize>,
}

impl CountingCachePlugin {
    fn new() -> Self {
        Self {
            get_call_count: Arc::new(AtomicUsize::new(0)),
            set_call_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn get_get_call_count(&self) -> usize {
        self.get_call_count.load(Ordering::SeqCst)
    }

    fn get_set_call_count(&self) -> usize {
        self.set_call_count.load(Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl license_enforcer_sdk::CachePluginClient for CountingCachePlugin {
    async fn get_tenant_features(
        &self,
        _ctx: &SecurityContext,
    ) -> Result<
        Option<license_enforcer_sdk::EnabledGlobalFeatures>,
        license_enforcer_sdk::LicenseEnforcerError,
    > {
        self.get_call_count.fetch_add(1, Ordering::SeqCst);
        Ok(None)
    }

    async fn set_tenant_features(
        &self,
        _ctx: &SecurityContext,
        _features: &license_enforcer_sdk::EnabledGlobalFeatures,
    ) -> Result<(), license_enforcer_sdk::LicenseEnforcerError> {
        self.set_call_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
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
        "hyperspot.builtin.static_licenses.plugin.v1",
    );

    assert_eq!(
        instance_id.as_ref(),
        "gts.x.core.modkit.plugin.v1~x.core.license_enforcer.integration.plugin.v1~hyperspot.builtin.static_licenses.plugin.v1",
        "Platform plugin instance ID must match design spec"
    );
}

#[test]
fn test_cache_plugin_instance_id_matches_design() {
    let instance_id =
        LicenseCachePluginSpecV1::gts_make_instance_id("hyperspot.builtin.nocache.plugin.v1");

    assert_eq!(
        instance_id.as_ref(),
        "gts.x.core.modkit.plugin.v1~x.core.license_enforcer.cache.plugin.v1~hyperspot.builtin.nocache.plugin.v1",
        "Cache plugin instance ID must match design spec"
    );
}

#[tokio::test]
async fn test_is_global_feature_enabled() {
    // Arrange: Set up ClientHub and mock types-registry
    let hub = Arc::new(ClientHub::new());
    let registry = MockTypesRegistry::new();
    hub.register::<dyn TypesRegistryClient>(Arc::new(registry.clone()));

    let config_provider = Arc::new(MockConfigProvider::new());
    let cancel = CancellationToken::new();

    // Initialize gateway module
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

    // Initialize platform plugin
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

    // Initialize cache plugin
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

    // Act: Get gateway client and check feature
    let client = hub
        .get::<dyn LicenseEnforcerGatewayClient>()
        .expect("Gateway client should be registered");

    let ctx = SecurityContext::builder()
        .tenant_id(Uuid::new_v4())
        .subject_id(Uuid::new_v4())
        .build();

    // Check base feature (should be enabled by stub implementation)
    let base_feature = global_features::to_feature_id(global_features::BASE);
    let is_enabled = client
        .is_global_feature_enabled(&ctx, &base_feature)
        .await
        .expect("Feature check should succeed");

    // Assert: Base feature should be enabled
    assert!(is_enabled, "Base feature should be enabled");

    // Check a non-existent feature (should not be enabled)
    let non_existent =
        global_features::to_feature_id("gts.x.core.lic.feat.v1~x.core.global.nonexistent.v1");
    let is_enabled = client
        .is_global_feature_enabled(&ctx, &non_existent)
        .await
        .expect("Feature check should succeed");

    assert!(!is_enabled, "Non-existent feature should not be enabled");
}

#[tokio::test]
async fn test_enabled_global_features() {
    // Arrange: Set up ClientHub and mock types-registry
    let hub = Arc::new(ClientHub::new());
    let registry = MockTypesRegistry::new();
    hub.register::<dyn TypesRegistryClient>(Arc::new(registry.clone()));

    let config_provider = Arc::new(MockConfigProvider::new());
    let cancel = CancellationToken::new();

    // Initialize gateway module
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

    // Initialize platform plugin
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

    // Initialize cache plugin
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

    // Act: Get gateway client and list features
    let client = hub
        .get::<dyn LicenseEnforcerGatewayClient>()
        .expect("Gateway client should be registered");

    let ctx = SecurityContext::builder()
        .tenant_id(Uuid::new_v4())
        .subject_id(Uuid::new_v4())
        .build();

    let features = client
        .enabled_global_features(&ctx)
        .await
        .expect("Features list should succeed");

    // Assert: Should contain base feature
    let base_feature = global_features::to_feature_id(global_features::BASE);
    assert!(
        features.contains(&base_feature),
        "Features should contain base feature"
    );
    assert_eq!(features.len(), 1, "Should only have one feature");
}

#[tokio::test]
async fn test_missing_tenant_scope() {
    // Arrange: Set up ClientHub with counting plugins
    let hub = Arc::new(ClientHub::new());
    let registry = MockTypesRegistry::new();
    hub.register::<dyn TypesRegistryClient>(Arc::new(registry.clone()));

    let config_provider = Arc::new(MockConfigProvider::new());
    let cancel = CancellationToken::new();

    // Initialize gateway module (registers schemas)
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

    // Register counting plugins with types-registry
    let platform_instance_id =
        license_enforcer_sdk::LicensePlatformPluginSpecV1::gts_make_instance_id(
            "test.counting.platform.v1",
        );
    let cache_instance_id = license_enforcer_sdk::LicenseCachePluginSpecV1::gts_make_instance_id(
        "test.counting.cache.v1",
    );

    registry
        .register(vec![
            serde_json::json!({
                "id": platform_instance_id.as_ref(),
                "vendor": "hyperspot",
                "priority": 100
            }),
            serde_json::json!({
                "id": cache_instance_id.as_ref(),
                "vendor": "hyperspot",
                "priority": 100
            }),
        ])
        .await
        .expect("Plugin registration failed");

    // Register counting plugin clients with scoped registration
    let platform_plugin = Arc::new(CountingPlatformPlugin::new());
    let cache_plugin = Arc::new(CountingCachePlugin::new());

    hub.register_scoped::<dyn license_enforcer_sdk::PlatformPluginClient>(
        modkit::client_hub::ClientScope::gts_id(platform_instance_id.as_ref()),
        platform_plugin.clone(),
    );
    hub.register_scoped::<dyn license_enforcer_sdk::CachePluginClient>(
        modkit::client_hub::ClientScope::gts_id(cache_instance_id.as_ref()),
        cache_plugin.clone(),
    );

    // Get gateway client
    let client = hub
        .get::<dyn LicenseEnforcerGatewayClient>()
        .expect("Gateway client should be registered");

    // Act: Call with anonymous context (nil tenant UUID)
    let ctx = SecurityContext::anonymous();
    let base_feature = global_features::to_feature_id(global_features::BASE);

    let result = client.is_global_feature_enabled(&ctx, &base_feature).await;

    // Assert 1: Should return missing tenant error
    assert!(result.is_err(), "Should return error for missing tenant");
    match result {
        Err(license_enforcer_sdk::LicenseEnforcerError::MissingTenantScope) => {
            // Expected
        }
        other => panic!("Expected MissingTenantScope error, got: {other:?}"),
    }

    // Assert 2: Platform plugin MUST NOT have been called (per spec requirement)
    assert_eq!(
        platform_plugin.get_call_count(),
        0,
        "Platform plugin MUST NOT be called when tenant scope is missing (spec violation)"
    );

    // Assert 3: Cache plugin MUST NOT have been called (per spec requirement)
    assert_eq!(
        cache_plugin.get_get_call_count(),
        0,
        "Cache plugin get MUST NOT be called when tenant scope is missing (spec violation)"
    );
    assert_eq!(
        cache_plugin.get_set_call_count(),
        0,
        "Cache plugin set MUST NOT be called when tenant scope is missing (spec violation)"
    );
}
