//! Service implementation for in-memory cache plugin.

use license_enforcer_sdk::{EnabledGlobalFeatures, LicenseEnforcerError};
use modkit_security::SecurityContext;
use moka::future::Cache;
use std::time::Duration;
use uuid::Uuid;

/// In-memory cache service with TTL support.
pub struct Service {
    cache: Cache<Uuid, EnabledGlobalFeatures>,
}

impl Service {
    /// Create a new service with the specified TTL.
    #[must_use]
    pub fn new(ttl: Duration, max_entries: usize) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_entries as u64)
            .time_to_live(ttl)
            .build();

        Self { cache }
    }

    /// Get cached tenant features.
    ///
    /// Returns None if no entry exists or entry has expired.
    ///
    /// # Errors
    ///
    /// This function currently never returns an error but is defined to return
    /// a `Result` for consistency with the plugin trait interface.
    pub async fn get_tenant_features(
        &self,
        _ctx: &SecurityContext,
        tenant_id: uuid::Uuid,
    ) -> Result<Option<EnabledGlobalFeatures>, LicenseEnforcerError> {
        // Get from cache (returns None if not found or expired)
        Ok(self.cache.get(&tenant_id).await)
    }

    /// Set cached tenant features with TTL.
    ///
    /// # Errors
    ///
    /// This function currently never returns an error but is defined to return
    /// a `Result` for consistency with the plugin trait interface.
    pub async fn set_tenant_features(
        &self,
        _ctx: &SecurityContext,
        tenant_id: uuid::Uuid,
        features: &EnabledGlobalFeatures,
    ) -> Result<(), LicenseEnforcerError> {
        // Store in cache with TTL
        self.cache.insert(tenant_id, features.clone()).await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use license_enforcer_sdk::{global_features, models::LicenseFeatureId};
    use std::collections::HashSet;
    use uuid::Uuid;

    fn make_test_context(tenant_id: Uuid) -> SecurityContext {
        SecurityContext::builder().tenant_id(tenant_id).build()
    }

    fn make_test_features() -> EnabledGlobalFeatures {
        let mut features = HashSet::new();
        features.insert(global_features::BaseFeature.to_gts());
        features.insert(global_features::CyberChatFeature.to_gts());
        features
    }

    #[tokio::test]
    async fn test_cache_miss_when_no_entry_exists() {
        // Arrange
        let service = Service::new(Duration::from_secs(60), 10_000);
        let tenant_id = Uuid::new_v4();
        let ctx = make_test_context(tenant_id);

        // Act
        let result = service.get_tenant_features(&ctx, tenant_id).await;

        // Assert
        assert!(
            result.is_ok(),
            "get_tenant_features should not error on cache miss"
        );
        assert!(
            result.unwrap().is_none(),
            "Should return None when no entry exists (cache miss)"
        );
    }

    #[tokio::test]
    async fn test_cache_hit_after_set() {
        // Arrange
        let service = Service::new(Duration::from_secs(60), 10_000);
        let tenant_id = Uuid::new_v4();
        let ctx = make_test_context(tenant_id);
        let features = make_test_features();

        // Act - Set features
        let set_result = service
            .set_tenant_features(&ctx, tenant_id, &features)
            .await;
        assert!(set_result.is_ok(), "set_tenant_features should succeed");

        // Act - Get features
        let get_result = service.get_tenant_features(&ctx, tenant_id).await;

        // Assert
        assert!(get_result.is_ok(), "get_tenant_features should not error");
        let cached_features = get_result.unwrap();
        assert!(cached_features.is_some(), "Should return Some (cache hit)");
        assert_eq!(
            cached_features.unwrap(),
            features,
            "Cached features should match stored features"
        );
    }

    #[tokio::test]
    async fn test_cache_expires_after_ttl() {
        // Arrange - Use a short TTL for testing
        let service = Service::new(Duration::from_secs(1), 10_000); // 1 second TTL
        let tenant_id = Uuid::new_v4();
        let ctx = make_test_context(tenant_id);
        let features = make_test_features();

        // Act - Set features
        service
            .set_tenant_features(&ctx, tenant_id, &features)
            .await
            .unwrap();

        // Verify cache hit before expiry
        let result_before = service.get_tenant_features(&ctx, tenant_id).await.unwrap();
        assert!(
            result_before.is_some(),
            "Should be cached before TTL expires"
        );

        // Wait for TTL to expire
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Act - Get features after expiry
        let result_after = service.get_tenant_features(&ctx, tenant_id).await;

        // Assert
        assert!(
            result_after.is_ok(),
            "get_tenant_features should not error after expiry"
        );
        assert!(
            result_after.unwrap().is_none(),
            "Should return None after TTL expires (cache miss)"
        );
    }

    #[tokio::test]
    async fn test_tenant_isolation() {
        // Arrange
        let service = Service::new(Duration::from_secs(60), 10_000);
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();
        let ctx_a = make_test_context(tenant_a);
        let ctx_b = make_test_context(tenant_b);
        let features_a = make_test_features();

        // Act - Set features for tenant A
        service
            .set_tenant_features(&ctx_a, tenant_a, &features_a)
            .await
            .unwrap();

        // Verify tenant A can read its own features
        let result_a = service.get_tenant_features(&ctx_a, tenant_a).await.unwrap();
        assert!(result_a.is_some(), "Tenant A should have cached features");

        // Act - Try to get features for tenant B
        let result_b = service.get_tenant_features(&ctx_b, tenant_b).await;

        // Assert
        assert!(
            result_b.is_ok(),
            "get_tenant_features should not error for tenant B"
        );
        assert!(
            result_b.unwrap().is_none(),
            "Tenant B should not access tenant A's cached features (tenant isolation)"
        );
    }

    #[tokio::test]
    async fn test_overwrite_resets_ttl() {
        // Arrange - Use a 2 second TTL
        let service = Service::new(Duration::from_secs(2), 10_000);
        let tenant_id = Uuid::new_v4();
        let ctx = make_test_context(tenant_id);

        let mut features1 = HashSet::new();
        features1.insert(global_features::BaseFeature.to_gts());

        let mut features2 = HashSet::new();
        features2.insert(global_features::CyberChatFeature.to_gts());

        // Act - Set initial features
        service
            .set_tenant_features(&ctx, tenant_id, &features1)
            .await
            .unwrap();

        // Wait 1 second (halfway to expiry)
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Overwrite with new features (should reset TTL)
        service
            .set_tenant_features(&ctx, tenant_id, &features2)
            .await
            .unwrap();

        // Wait another 1.5 seconds (would have expired if TTL wasn't reset)
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

        // Act - Get features
        let result = service.get_tenant_features(&ctx, tenant_id).await.unwrap();

        // Assert - Should still be cached with new value
        assert!(
            result.is_some(),
            "Entry should still be cached (TTL was reset)"
        );
        assert_eq!(result.unwrap(), features2, "Should have updated value");
    }

    #[tokio::test]
    async fn test_sub_second_ttl_precision() {
        // Test that sub-second TTL values are preserved (not truncated to 0)
        use std::time::Duration;

        // Arrange - Use 500ms TTL
        let service = Service::new(Duration::from_millis(500), 10_000);
        let tenant_id = Uuid::new_v4();
        let ctx = make_test_context(tenant_id);
        let features = make_test_features();

        // Act - Set features
        service
            .set_tenant_features(&ctx, tenant_id, &features)
            .await
            .unwrap();

        // Verify cached before 500ms
        tokio::time::sleep(Duration::from_millis(200)).await;
        let result_before = service.get_tenant_features(&ctx, tenant_id).await.unwrap();
        assert!(result_before.is_some(), "Should be cached at 200ms");

        // Wait for expiry
        tokio::time::sleep(Duration::from_millis(400)).await;

        // Act - Get features after 600ms total (past 500ms TTL)
        let result_after = service.get_tenant_features(&ctx, tenant_id).await.unwrap();

        // Assert - Should be expired
        assert!(
            result_after.is_none(),
            "Should be expired after 500ms TTL (sub-second precision preserved)"
        );
    }
}
