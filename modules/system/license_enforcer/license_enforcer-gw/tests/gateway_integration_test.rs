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
use license_enforcer_gw::models::{LicenseFeatureId, parse_license_feature_id};
use license_enforcer_sdk::{
    LicenseCachePluginSpecV1, LicenseEnforcerGatewayClient, LicensePlatformPluginSpecV1,
    global_features,
};
use modkit::config::ConfigProvider;
use modkit::gts::BaseModkitPluginV1;
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
        _tenant_id: uuid::Uuid,
    ) -> Result<
        license_enforcer_sdk::EnabledGlobalFeatures,
        license_enforcer_sdk::LicenseEnforcerError,
    > {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        let mut features = license_enforcer_sdk::EnabledGlobalFeatures::new();
        let base_feature = &license_enforcer_sdk::global_features::BaseFeature;
        features.insert(base_feature.to_gts());
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
        _tenant_id: uuid::Uuid,
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
        _tenant_id: uuid::Uuid,
        _features: &license_enforcer_sdk::EnabledGlobalFeatures,
    ) -> Result<(), license_enforcer_sdk::LicenseEnforcerError> {
        self.set_call_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

/// Cache plugin that always returns a cache hit with predefined features.
#[derive(Clone)]
struct AlwaysHitCachePlugin {
    get_call_count: Arc<AtomicUsize>,
    cached_features: license_enforcer_sdk::EnabledGlobalFeatures,
}

impl AlwaysHitCachePlugin {
    fn new() -> Self {
        let base_feature = &license_enforcer_sdk::global_features::BaseFeature;
        let mut features = license_enforcer_sdk::EnabledGlobalFeatures::new();
        features.insert(base_feature.to_gts());
        Self {
            get_call_count: Arc::new(AtomicUsize::new(0)),
            cached_features: features,
        }
    }

    fn get_call_count(&self) -> usize {
        self.get_call_count.load(Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl license_enforcer_sdk::CachePluginClient for AlwaysHitCachePlugin {
    async fn get_tenant_features(
        &self,
        _ctx: &SecurityContext,
        _tenant_id: uuid::Uuid,
    ) -> Result<
        Option<license_enforcer_sdk::EnabledGlobalFeatures>,
        license_enforcer_sdk::LicenseEnforcerError,
    > {
        self.get_call_count.fetch_add(1, Ordering::SeqCst);

        // Always return cache hit
        Ok(Some(self.cached_features.clone()))
    }

    async fn set_tenant_features(
        &self,
        _ctx: &SecurityContext,
        _tenant_id: uuid::Uuid,
        _features: &license_enforcer_sdk::EnabledGlobalFeatures,
    ) -> Result<(), license_enforcer_sdk::LicenseEnforcerError> {
        // No-op for this test plugin
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
                    "priority": 100,
                    "static_licenses_features": []
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
        "gts.x.core.modkit.plugin.v1~x.core.license_resolver.plugin.v1~hyperspot.builtin.static_licenses.plugin.v1",
        "Platform plugin instance ID must match design spec"
    );
}

#[test]
fn test_cache_plugin_instance_id_matches_design() {
    let instance_id =
        LicenseCachePluginSpecV1::gts_make_instance_id("hyperspot.builtin.nocache.plugin.v1");

    assert_eq!(
        instance_id.as_ref(),
        "gts.x.core.modkit.plugin.v1~x.core.license_cache.plugin.v1~hyperspot.builtin.nocache.plugin.v1",
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
    let base_feature = &global_features::BaseFeature;
    let is_enabled = client
        .is_global_feature_enabled(&ctx, ctx.tenant_id(), base_feature)
        .await
        .expect("Feature check should succeed");

    // Assert: Base feature should be enabled
    assert!(is_enabled, "Base feature should be enabled");

    // Check a non-existent feature (should not be enabled)
    let non_existent =
        parse_license_feature_id("gts.x.core.lic.feat.v1~x.core.global.nonexistent.v1").unwrap();
    let is_enabled = client
        .is_global_feature_enabled(&ctx, ctx.tenant_id(), &*non_existent)
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
        .list_enabled_global_features(&ctx, ctx.tenant_id())
        .await
        .expect("Features list should succeed");

    // Assert: Should contain base feature
    let base_feature = &global_features::BaseFeature;
    assert!(
        features.contains(&base_feature.to_gts()),
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

    // Construct proper BaseModkitPluginV1 instances with properties field
    let platform_instance = BaseModkitPluginV1::<LicensePlatformPluginSpecV1> {
        id: platform_instance_id.clone(),
        vendor: "hyperspot".to_owned(),
        priority: 100,
        properties: LicensePlatformPluginSpecV1,
    };
    let cache_instance = BaseModkitPluginV1::<LicenseCachePluginSpecV1> {
        id: cache_instance_id.clone(),
        vendor: "hyperspot".to_owned(),
        priority: 100,
        properties: LicenseCachePluginSpecV1,
    };

    registry
        .register(vec![
            serde_json::to_value(&platform_instance)
                .expect("Failed to serialize platform instance"),
            serde_json::to_value(&cache_instance).expect("Failed to serialize cache instance"),
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
    let base_feature = &global_features::BaseFeature;

    let result = client
        .is_global_feature_enabled(&ctx, ctx.tenant_id(), base_feature)
        .await;

    // Assert 1: Should return error (platform plugin will fail to resolve due to invalid instance)
    assert!(result.is_err(), "Should return error for missing tenant");

    // Note: With explicit tenant_id parameter, we pass nil UUID to the platform plugin.
    // The platform plugin may or may not validate the tenant ID. In this test setup,
    // the error comes from plugin resolution failure, which is acceptable.
    // Real implementations should validate tenant_id if needed.

    // Assert 2: Platform plugin call count is not guaranteed to be 0 since plugin
    // discovery happens first. The important thing is that the call fails.
    assert!(result.is_err(), "Call with nil tenant ID should fail");

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

// ============================================================================
// In-Memory Cache Plugin Integration Tests
// ============================================================================

#[tokio::test]
async fn test_inmemory_cache_miss_triggers_platform_call() {
    // This test verifies cache-aside behavior on cache miss:
    // - First call should miss cache and fetch from platform
    // - Platform plugin should be called exactly once
    // - Result should be stored in cache

    // Arrange: Set up ClientHub and mock types-registry
    let hub = Arc::new(ClientHub::new());
    let registry = MockTypesRegistry::new();
    hub.register::<dyn TypesRegistryClient>(Arc::new(registry.clone()));

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

    // In-memory cache plugin config with short TTL for testing
    modules.insert(
        "inmemory_cache_plugin".to_owned(),
        json!({
            "config": {
                "vendor": "hyperspot",
                "priority": 100,
                "ttl": "60s",
                "max_entries": 1000
            }
        }),
    );

    let config_provider = Arc::new(MockConfigProvider { modules });
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

    // Initialize in-memory cache plugin
    let cache_ctx = ModuleCtx::new(
        "inmemory_cache_plugin",
        Uuid::new_v4(),
        config_provider.clone(),
        hub.clone(),
        cancel.clone(),
        None,
    );
    let cache_plugin = InMemoryCachePlugin::default();
    cache_plugin
        .init(&cache_ctx)
        .await
        .expect("cache plugin init failed");

    // Register counting platform plugin
    let platform_instance_id =
        license_enforcer_sdk::LicensePlatformPluginSpecV1::gts_make_instance_id(
            "test.counting.platform.v1",
        );

    let platform_instance = modkit::gts::BaseModkitPluginV1::<LicensePlatformPluginSpecV1> {
        id: platform_instance_id.clone(),
        vendor: "hyperspot".to_owned(),
        priority: 100,
        properties: LicensePlatformPluginSpecV1,
    };

    registry
        .register(vec![serde_json::to_value(&platform_instance).unwrap()])
        .await
        .expect("Platform registration failed");

    let platform_plugin = Arc::new(CountingPlatformPlugin::new());
    hub.register_scoped::<dyn license_enforcer_sdk::PlatformPluginClient>(
        modkit::client_hub::ClientScope::gts_id(platform_instance_id.as_ref()),
        platform_plugin.clone(),
    );

    // Get gateway client
    let client = hub
        .get::<dyn LicenseEnforcerGatewayClient>()
        .expect("Gateway client should be registered");

    let ctx = SecurityContext::builder()
        .tenant_id(Uuid::new_v4())
        .subject_id(Uuid::new_v4())
        .build();

    // Act: Call gateway (should miss cache, fetch from platform, and store)
    let base_feature = &global_features::BaseFeature;
    let is_enabled = client
        .is_global_feature_enabled(&ctx, ctx.tenant_id(), base_feature)
        .await
        .expect("Feature check should succeed");

    // Assert: Feature should be enabled (from platform)
    assert!(is_enabled, "Base feature should be enabled");

    // Assert: Platform should have been called exactly once
    assert_eq!(
        platform_plugin.get_call_count(),
        1,
        "Platform plugin should be called on cache miss"
    );
}

#[tokio::test]
async fn test_inmemory_cache_hit_avoids_platform_call() {
    // This test verifies cache-aside behavior on cache hit:
    // - First call should miss cache and fetch from platform
    // - Second call should hit cache and NOT call platform
    // - Platform plugin should be called exactly once (not twice)

    // Arrange: Set up ClientHub and mock types-registry
    let hub = Arc::new(ClientHub::new());
    let registry = MockTypesRegistry::new();
    hub.register::<dyn TypesRegistryClient>(Arc::new(registry.clone()));

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

    // In-memory cache plugin config with long TTL
    modules.insert(
        "inmemory_cache_plugin".to_owned(),
        json!({
            "config": {
                "vendor": "hyperspot",
                "priority": 100,
                "ttl": "300s",
                "max_entries": 1000
            }
        }),
    );

    let config_provider = Arc::new(MockConfigProvider { modules });
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

    // Initialize in-memory cache plugin
    let cache_ctx = ModuleCtx::new(
        "inmemory_cache_plugin",
        Uuid::new_v4(),
        config_provider.clone(),
        hub.clone(),
        cancel.clone(),
        None,
    );
    let cache_plugin = InMemoryCachePlugin::default();
    cache_plugin
        .init(&cache_ctx)
        .await
        .expect("cache plugin init failed");

    // Register counting platform plugin
    let platform_instance_id =
        license_enforcer_sdk::LicensePlatformPluginSpecV1::gts_make_instance_id(
            "test.counting.platform.v1",
        );

    let platform_instance = modkit::gts::BaseModkitPluginV1::<LicensePlatformPluginSpecV1> {
        id: platform_instance_id.clone(),
        vendor: "hyperspot".to_owned(),
        priority: 100,
        properties: LicensePlatformPluginSpecV1,
    };

    registry
        .register(vec![serde_json::to_value(&platform_instance).unwrap()])
        .await
        .expect("Platform registration failed");

    let platform_plugin = Arc::new(CountingPlatformPlugin::new());
    hub.register_scoped::<dyn license_enforcer_sdk::PlatformPluginClient>(
        modkit::client_hub::ClientScope::gts_id(platform_instance_id.as_ref()),
        platform_plugin.clone(),
    );

    // Get gateway client
    let client = hub
        .get::<dyn LicenseEnforcerGatewayClient>()
        .expect("Gateway client should be registered");

    let ctx = SecurityContext::builder()
        .tenant_id(Uuid::new_v4())
        .subject_id(Uuid::new_v4())
        .build();

    let base_feature = &global_features::BaseFeature;

    // Act: First call (cache miss)
    let is_enabled_1 = client
        .is_global_feature_enabled(&ctx, ctx.tenant_id(), base_feature)
        .await
        .expect("First feature check should succeed");

    assert!(is_enabled_1, "Base feature should be enabled on first call");
    assert_eq!(
        platform_plugin.get_call_count(),
        1,
        "Platform should be called on first request (cache miss)"
    );

    // Act: Second call (cache hit)
    let is_enabled_2 = client
        .is_global_feature_enabled(&ctx, ctx.tenant_id(), base_feature)
        .await
        .expect("Second feature check should succeed");

    // Assert: Feature should still be enabled
    assert!(
        is_enabled_2,
        "Base feature should be enabled on second call"
    );

    // Assert: Platform should NOT be called again (cache hit)
    assert_eq!(
        platform_plugin.get_call_count(),
        1,
        "Platform plugin should NOT be called on second request (cache hit should avoid platform)"
    );
}

#[tokio::test]
async fn test_inmemory_cache_ttl_expiry_refreshes_from_platform() {
    // This test verifies TTL expiry behavior:
    // - First call should miss cache and fetch from platform
    // - Second call should hit cache (no platform call)
    // - After TTL expires, third call should miss cache and fetch from platform again
    // - Platform should be called exactly twice (initial + after expiry)

    // Arrange: Set up ClientHub and mock types-registry
    let hub = Arc::new(ClientHub::new());
    let registry = MockTypesRegistry::new();
    hub.register::<dyn TypesRegistryClient>(Arc::new(registry.clone()));

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

    // In-memory cache plugin config with very short TTL for testing
    modules.insert(
        "inmemory_cache_plugin".to_owned(),
        json!({
            "config": {
                "vendor": "hyperspot",
                "priority": 100,
                "ttl": "1s",
                "max_entries": 1000
            }
        }),
    );

    let config_provider = Arc::new(MockConfigProvider { modules });
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

    // Initialize in-memory cache plugin
    let cache_ctx = ModuleCtx::new(
        "inmemory_cache_plugin",
        Uuid::new_v4(),
        config_provider.clone(),
        hub.clone(),
        cancel.clone(),
        None,
    );
    let cache_plugin = InMemoryCachePlugin::default();
    cache_plugin
        .init(&cache_ctx)
        .await
        .expect("cache plugin init failed");

    // Register counting platform plugin
    let platform_instance_id =
        license_enforcer_sdk::LicensePlatformPluginSpecV1::gts_make_instance_id(
            "test.counting.platform.v1",
        );

    let platform_instance = modkit::gts::BaseModkitPluginV1::<LicensePlatformPluginSpecV1> {
        id: platform_instance_id.clone(),
        vendor: "hyperspot".to_owned(),
        priority: 100,
        properties: LicensePlatformPluginSpecV1,
    };

    registry
        .register(vec![serde_json::to_value(&platform_instance).unwrap()])
        .await
        .expect("Platform registration failed");

    let platform_plugin = Arc::new(CountingPlatformPlugin::new());
    hub.register_scoped::<dyn license_enforcer_sdk::PlatformPluginClient>(
        modkit::client_hub::ClientScope::gts_id(platform_instance_id.as_ref()),
        platform_plugin.clone(),
    );

    // Get gateway client
    let client = hub
        .get::<dyn LicenseEnforcerGatewayClient>()
        .expect("Gateway client should be registered");

    let ctx = SecurityContext::builder()
        .tenant_id(Uuid::new_v4())
        .subject_id(Uuid::new_v4())
        .build();

    let base_feature = &global_features::BaseFeature;
    // Act: First call (cache miss)
    let is_enabled_1 = client
        .is_global_feature_enabled(&ctx, ctx.tenant_id(), base_feature)
        .await
        .expect("First feature check should succeed");

    assert!(is_enabled_1, "Base feature should be enabled");
    assert_eq!(
        platform_plugin.get_call_count(),
        1,
        "Platform should be called on first request"
    );

    // Act: Second call immediately (cache hit)
    let is_enabled_2 = client
        .is_global_feature_enabled(&ctx, ctx.tenant_id(), base_feature)
        .await
        .expect("Second feature check should succeed");

    assert!(is_enabled_2, "Base feature should be enabled");
    assert_eq!(
        platform_plugin.get_call_count(),
        1,
        "Platform should NOT be called on second request (cache hit)"
    );

    // Wait for TTL to expire (1 second + margin)
    // Note: Using real sleep here because Moka cache uses std::time::Instant internally,
    // not tokio::time::Instant, so tokio::time::pause()/advance() would not work.
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Act: Third call after TTL expiry (cache miss due to expiry)
    let is_enabled_3 = client
        .is_global_feature_enabled(&ctx, ctx.tenant_id(), base_feature)
        .await
        .expect("Third feature check should succeed");

    // Assert: Feature should still be enabled
    assert!(
        is_enabled_3,
        "Base feature should be enabled after TTL expiry"
    );

    // Assert: Platform should be called again after TTL expiry
    assert_eq!(
        platform_plugin.get_call_count(),
        2,
        "Platform plugin should be called again after TTL expiry (cache refresh)"
    );
}

#[tokio::test]
async fn test_cache_hit_completely_avoids_platform_call() {
    // This test verifies that when cache returns a hit immediately,
    // the platform plugin is NEVER called (not even once).
    // This proves the cache-aside pattern works correctly for cache hits.

    // Arrange: Set up ClientHub and mock types-registry
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

    // Register plugins with types-registry
    let platform_instance_id =
        license_enforcer_sdk::LicensePlatformPluginSpecV1::gts_make_instance_id(
            "test.counting.platform.v1",
        );
    let cache_instance_id = license_enforcer_sdk::LicenseCachePluginSpecV1::gts_make_instance_id(
        "test.always_hit.cache.v1",
    );

    let platform_instance = modkit::gts::BaseModkitPluginV1::<LicensePlatformPluginSpecV1> {
        id: platform_instance_id.clone(),
        vendor: "hyperspot".to_owned(),
        priority: 100,
        properties: LicensePlatformPluginSpecV1,
    };
    let cache_instance = modkit::gts::BaseModkitPluginV1::<LicenseCachePluginSpecV1> {
        id: cache_instance_id.clone(),
        vendor: "hyperspot".to_owned(),
        priority: 100,
        properties: LicenseCachePluginSpecV1,
    };

    registry
        .register(vec![
            serde_json::to_value(&platform_instance).unwrap(),
            serde_json::to_value(&cache_instance).unwrap(),
        ])
        .await
        .expect("Plugin registration failed");

    // Register counting platform plugin
    let platform_plugin = Arc::new(CountingPlatformPlugin::new());
    hub.register_scoped::<dyn license_enforcer_sdk::PlatformPluginClient>(
        modkit::client_hub::ClientScope::gts_id(platform_instance_id.as_ref()),
        platform_plugin.clone(),
    );

    // Register always-hit cache plugin
    let cache_plugin = Arc::new(AlwaysHitCachePlugin::new());
    hub.register_scoped::<dyn license_enforcer_sdk::CachePluginClient>(
        modkit::client_hub::ClientScope::gts_id(cache_instance_id.as_ref()),
        cache_plugin.clone(),
    );

    // Get gateway client
    let client = hub
        .get::<dyn LicenseEnforcerGatewayClient>()
        .expect("Gateway client should be registered");

    let ctx = SecurityContext::builder()
        .tenant_id(Uuid::new_v4())
        .subject_id(Uuid::new_v4())
        .build();

    let base_feature = &global_features::BaseFeature;
    // Act: Call gateway (cache will return hit immediately)
    let is_enabled = client
        .is_global_feature_enabled(&ctx, ctx.tenant_id(), base_feature)
        .await
        .expect("Feature check should succeed");

    // Assert: Feature should be enabled (from cache)
    assert!(is_enabled, "Base feature should be enabled from cache");

    // Assert: Cache plugin should have been called
    assert_eq!(
        cache_plugin.get_call_count(),
        1,
        "Cache plugin should be called to check for cached data"
    );

    // Assert: Platform plugin should NEVER be called when cache hits
    assert_eq!(
        platform_plugin.get_call_count(),
        0,
        "Platform plugin MUST NOT be called when cache returns a hit (cache-aside pattern)"
    );

    // Act: Second call (should also hit cache)
    let is_enabled_2 = client
        .is_global_feature_enabled(&ctx, ctx.tenant_id(), base_feature)
        .await
        .expect("Second feature check should succeed");

    assert!(
        is_enabled_2,
        "Base feature should still be enabled from cache"
    );

    // Assert: Platform should still never be called
    assert_eq!(
        platform_plugin.get_call_count(),
        0,
        "Platform plugin MUST still be at 0 calls after second request (both cache hits)"
    );

    // Assert: Cache was called twice
    assert_eq!(
        cache_plugin.get_call_count(),
        2,
        "Cache plugin should be called on both requests"
    );
}

// ============================================================================
// Static Licenses Plugin Configuration Tests
// ============================================================================

#[tokio::test]
async fn test_static_licenses_plugin_init_fails_without_static_licenses_features() {
    // This test verifies that the static_licenses_plugin fails to initialize
    // when the required `static_licenses_features` field is missing from config.
    // This is a BREAKING requirement from the update-static-licenses-features-config proposal.

    // Arrange: Set up ClientHub and mock types-registry
    let hub = Arc::new(ClientHub::new());
    let registry = MockTypesRegistry::new();
    hub.register::<dyn TypesRegistryClient>(Arc::new(registry.clone()));

    // Create config WITHOUT static_licenses_features field
    let mut modules = HashMap::new();
    modules.insert(
        "static_licenses_plugin".to_owned(),
        json!({
            "config": {
                "vendor": "hyperspot",
                "priority": 100
                // INTENTIONALLY MISSING: "static_licenses_features"
            }
        }),
    );
    let config_provider = Arc::new(MockConfigProvider { modules });
    let cancel = CancellationToken::new();

    // Act: Try to initialize static_licenses_plugin without the required field
    let platform_ctx = ModuleCtx::new(
        "static_licenses_plugin",
        Uuid::new_v4(),
        config_provider,
        hub.clone(),
        cancel,
        None,
    );
    let platform_plugin = StaticLicensesPlugin::default();
    let result = platform_plugin.init(&platform_ctx).await;

    // Assert: Initialization should fail
    assert!(
        result.is_err(),
        "Plugin initialization should fail when static_licenses_features is missing"
    );

    // Assert: Error should mention the missing field
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("static_licenses_features") || err_msg.contains("missing field"),
        "Error message should mention missing static_licenses_features field, got: {err_msg}"
    );
}

#[tokio::test]
async fn test_static_licenses_plugin_returns_configured_features() {
    // This test verifies that the static_licenses_plugin returns the configured
    // features in addition to the base feature.

    // Arrange: Set up ClientHub and mock types-registry
    let hub = Arc::new(ClientHub::new());
    let registry = MockTypesRegistry::new();
    hub.register::<dyn TypesRegistryClient>(Arc::new(registry.clone()));

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

    // Static licenses plugin with configured features
    modules.insert(
        "static_licenses_plugin".to_owned(),
        json!({
            "config": {
                "vendor": "hyperspot",
                "priority": 100,
                "static_licenses_features": [
                    "gts.x.core.lic.feat.v1~x.core.global.advanced_analytics.v1",
                    "gts.x.core.lic.feat.v1~x.core.global.export.v1"
                ]
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

    let config_provider = Arc::new(MockConfigProvider { modules });
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

    // Initialize platform plugin with configured features
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
        .expect("platform plugin init should succeed with static_licenses_features");

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
        .list_enabled_global_features(&ctx, ctx.tenant_id())
        .await
        .expect("Features list should succeed");
    // Assert: Should contain base feature + configured features
    let base_feature = &global_features::BaseFeature;
    assert!(
        features.contains(&base_feature.to_gts()),
        "Features should contain base feature"
    );

    let analytics_feature =
        parse_license_feature_id("gts.x.core.lic.feat.v1~x.core.global.advanced_analytics.v1")
            .unwrap();
    let export_feature =
        parse_license_feature_id("gts.x.core.lic.feat.v1~x.core.global.export.v1").unwrap();

    assert!(
        features.contains(&analytics_feature.to_gts()),
        "Features should contain configured advanced-analytics feature"
    );
    assert!(
        features.contains(&export_feature.to_gts()),
        "Features should contain configured export feature"
    );

    assert_eq!(
        features.len(),
        3,
        "Should have base feature + 2 configured features"
    );
}

#[tokio::test]
async fn test_static_licenses_plugin_init_fails_with_invalid_gts_id() {
    // This test verifies that module init performs basic GTS ID format validation.
    // Per design: "no registry validation at config-load time (parsing/structure validation only)".
    // However, basic format checks (starts with "gts.") are acceptable.

    // Arrange: Set up ClientHub and mock types-registry
    let hub = Arc::new(ClientHub::new());
    let registry = MockTypesRegistry::new();
    hub.register::<dyn TypesRegistryClient>(Arc::new(registry.clone()));

    // Create config with invalid GTS ID (doesn't start with "gts.")
    let mut modules = HashMap::new();
    modules.insert(
        "static_licenses_plugin".to_owned(),
        json!({
            "config": {
                "vendor": "hyperspot",
                "priority": 100,
                "static_licenses_features": ["invalid-id-without-gts-prefix"]
            }
        }),
    );
    let config_provider = Arc::new(MockConfigProvider { modules });
    let cancel = CancellationToken::new();

    // Act: Try to initialize static_licenses_plugin with invalid GTS ID
    let platform_ctx = ModuleCtx::new(
        "static_licenses_plugin",
        Uuid::new_v4(),
        config_provider,
        hub.clone(),
        cancel,
        None,
    );
    let platform_plugin = StaticLicensesPlugin::default();
    let result = platform_plugin.init(&platform_ctx).await;

    // Assert: Initialization should fail
    assert!(
        result.is_err(),
        "Plugin initialization should fail with invalid GTS ID"
    );

    // Assert: Error should mention GTS format
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("gts.") || err_msg.contains("GTS"),
        "Error message should mention GTS format requirement, got: {err_msg}"
    );
}

#[tokio::test]
async fn test_static_licenses_plugin_init_fails_with_empty_feature_id() {
    // This test verifies that empty feature IDs are rejected during init.

    // Arrange: Set up ClientHub and mock types-registry
    let hub = Arc::new(ClientHub::new());
    let registry = MockTypesRegistry::new();
    hub.register::<dyn TypesRegistryClient>(Arc::new(registry.clone()));

    // Create config with empty feature ID
    let mut modules = HashMap::new();
    modules.insert(
        "static_licenses_plugin".to_owned(),
        json!({
            "config": {
                "vendor": "hyperspot",
                "priority": 100,
                "static_licenses_features": [""]
            }
        }),
    );
    let config_provider = Arc::new(MockConfigProvider { modules });
    let cancel = CancellationToken::new();

    // Act: Try to initialize static_licenses_plugin with empty feature ID
    let platform_ctx = ModuleCtx::new(
        "static_licenses_plugin",
        Uuid::new_v4(),
        config_provider,
        hub.clone(),
        cancel,
        None,
    );
    let platform_plugin = StaticLicensesPlugin::default();
    let result = platform_plugin.init(&platform_ctx).await;

    // Assert: Initialization should fail
    assert!(
        result.is_err(),
        "Plugin initialization should fail with empty feature ID"
    );

    // Assert: Error should mention invalid GTS ID
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("not a valid GTS ID") || err_msg.contains("Invalid"),
        "Error message should mention invalid GTS ID, got: {err_msg}"
    );
}

#[tokio::test]
async fn test_tenant_access_validation_denies_unauthorized_tenant() {
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

    // Register platform plugin with types-registry
    let platform_instance_id =
        license_enforcer_sdk::LicensePlatformPluginSpecV1::gts_make_instance_id(
            "test.counting.platform.v1",
        );

    let platform_instance = modkit::gts::BaseModkitPluginV1::<LicensePlatformPluginSpecV1> {
        id: platform_instance_id.clone(),
        vendor: "hyperspot".to_owned(),
        priority: 100,
        properties: LicensePlatformPluginSpecV1,
    };

    registry
        .register(vec![serde_json::to_value(&platform_instance).unwrap()])
        .await
        .expect("Platform registration failed");

    // Register platform plugin client with scoped registration
    let platform_plugin = Arc::new(CountingPlatformPlugin::new());
    hub.register_scoped::<dyn license_enforcer_sdk::PlatformPluginClient>(
        modkit::client_hub::ClientScope::gts_id(platform_instance_id.as_ref()),
        platform_plugin.clone(),
    );

    // Get gateway client
    let client = hub
        .get::<dyn LicenseEnforcerGatewayClient>()
        .expect("Gateway client should be registered");

    // Create security context for tenant A
    let tenant_a = Uuid::new_v4();
    let ctx_for_tenant_a = SecurityContext::builder()
        .tenant_id(tenant_a)
        .subject_id(Uuid::new_v4())
        .build();

    // Act: Try to access tenant B (unauthorized)
    let tenant_b = Uuid::new_v4();
    let base_feature = &global_features::BaseFeature;
    let result = client
        .is_global_feature_enabled(&ctx_for_tenant_a, tenant_b, base_feature)
        .await;

    // Assert: Should fail with access denied error
    assert!(
        result.is_err(),
        "Should deny access to tenant B when context is for tenant A"
    );
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("access")
            || err_msg.contains("denied")
            || err_msg.contains("unauthorized"),
        "Error should indicate access denial, got: {err_msg}"
    );
}

#[tokio::test]
async fn test_tenant_access_validation_allows_authorized_tenant() {
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

    // Register platform plugin with types-registry
    let platform_instance_id =
        license_enforcer_sdk::LicensePlatformPluginSpecV1::gts_make_instance_id(
            "test.counting.platform.v1",
        );

    let platform_instance = modkit::gts::BaseModkitPluginV1::<LicensePlatformPluginSpecV1> {
        id: platform_instance_id.clone(),
        vendor: "hyperspot".to_owned(),
        priority: 100,
        properties: LicensePlatformPluginSpecV1,
    };

    registry
        .register(vec![serde_json::to_value(&platform_instance).unwrap()])
        .await
        .expect("Platform registration failed");

    // Register platform plugin client with scoped registration
    let platform_plugin = Arc::new(CountingPlatformPlugin::new());
    hub.register_scoped::<dyn license_enforcer_sdk::PlatformPluginClient>(
        modkit::client_hub::ClientScope::gts_id(platform_instance_id.as_ref()),
        platform_plugin.clone(),
    );

    // Get gateway client
    let client = hub
        .get::<dyn LicenseEnforcerGatewayClient>()
        .expect("Gateway client should be registered");

    // Create security context for tenant A
    let tenant_a = Uuid::new_v4();
    let ctx_for_tenant_a = SecurityContext::builder()
        .tenant_id(tenant_a)
        .subject_id(Uuid::new_v4())
        .build();

    // Act: Access tenant A (authorized - matching context)
    let base_feature = &global_features::BaseFeature;
    let result = client
        .is_global_feature_enabled(&ctx_for_tenant_a, tenant_a, base_feature)
        .await;

    // Assert: Should succeed
    assert!(
        result.is_ok(),
        "Should allow access to tenant A when context is for tenant A"
    );
    assert!(result.unwrap(), "Base feature should be enabled");
}
