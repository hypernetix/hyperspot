//! Service implementation for static licenses plugin.
//!
//! This is a configuration-driven stub that returns configured features for bootstrap/testing.

use license_enforcer_sdk::{
    EnabledGlobalFeatures, LicenseEnforcerError, LicenseFeatureID, global_features,
};
use modkit_security::SecurityContext;

/// Static licenses service.
///
/// Provides configured license data. Returns the base feature plus
/// any additional features specified in configuration.
pub struct Service {
    /// Configured enabled global features (in addition to base feature).
    configured_features: Vec<LicenseFeatureID>,
}

impl Service {
    /// Create a new service with configured features.
    ///
    /// The service will return the base feature plus any features
    /// provided in the configuration.
    #[must_use]
    pub fn new(configured_features: Vec<LicenseFeatureID>) -> Self {
        Self {
            configured_features,
        }
    }

    /// Get enabled global features.
    ///
    /// Returns the base feature plus any configured additional features.
    ///
    /// # Errors
    ///
    /// This function currently never returns an error but is defined to return
    /// a `Result` for consistency with the plugin trait interface.
    #[allow(clippy::unused_async)]
    pub async fn get_enabled_global_features(
        &self,
        _ctx: &SecurityContext,
        _tenant_id: uuid::Uuid,
    ) -> Result<EnabledGlobalFeatures, LicenseEnforcerError> {
        // Return base feature plus configured features
        let mut features = EnabledGlobalFeatures::new();
        features.insert(LicenseFeatureID::from(global_features::BASE));

        // Add configured features
        for feature in &self.configured_features {
            features.insert(feature.clone());
        }

        Ok(features)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_test_context() -> SecurityContext {
        SecurityContext::builder().tenant_id(Uuid::new_v4()).build()
    }

    #[tokio::test]
    async fn test_service_returns_base_feature_with_empty_config() {
        let service = Service::new(vec![]);
        let ctx = make_test_context();
        let tenant_id = ctx.tenant_id();

        let result = service.get_enabled_global_features(&ctx, tenant_id).await;
        assert!(result.is_ok());

        let features = result.unwrap();
        assert_eq!(features.len(), 1, "Should return only base feature");
        assert!(
            features.contains(&LicenseFeatureID::from(global_features::BASE)),
            "Should contain base feature"
        );
    }

    #[tokio::test]
    async fn test_service_returns_base_plus_configured_features() {
        let configured = vec![
            LicenseFeatureID::from("gts.x.core.lic.feat.v1~x.core.global.advanced_analytics.v1"),
            LicenseFeatureID::from("gts.x.core.lic.feat.v1~x.core.global.export.v1"),
        ];
        let service = Service::new(configured);
        let ctx = make_test_context();
        let tenant_id = ctx.tenant_id();

        let result = service.get_enabled_global_features(&ctx, tenant_id).await;
        assert!(result.is_ok());

        let features = result.unwrap();
        assert_eq!(
            features.len(),
            3,
            "Should return base + 2 configured features"
        );
        assert!(
            features.contains(&LicenseFeatureID::from(global_features::BASE)),
            "Should contain base feature"
        );
        assert!(
            features.contains(&LicenseFeatureID::from(
                "gts.x.core.lic.feat.v1~x.core.global.advanced_analytics.v1"
            )),
            "Should contain advanced_analytics feature"
        );
        assert!(
            features.contains(&LicenseFeatureID::from(
                "gts.x.core.lic.feat.v1~x.core.global.export.v1"
            )),
            "Should contain export feature"
        );
    }

    #[tokio::test]
    async fn test_service_handles_duplicate_base_feature() {
        // If base feature is included in config, should not duplicate
        let configured = vec![
            LicenseFeatureID::from(global_features::BASE),
            LicenseFeatureID::from("gts.x.core.lic.feat.v1~x.core.global.test.v1"),
        ];
        let service = Service::new(configured);
        let ctx = make_test_context();
        let tenant_id = ctx.tenant_id();

        let result = service.get_enabled_global_features(&ctx, tenant_id).await;
        assert!(result.is_ok());

        let features = result.unwrap();
        // HashSet should de-duplicate, so we expect 2 unique features
        assert_eq!(features.len(), 2, "Should deduplicate base feature");
        assert!(
            features.contains(&LicenseFeatureID::from(global_features::BASE)),
            "Should contain base feature"
        );
        assert!(
            features.contains(&LicenseFeatureID::from(
                "gts.x.core.lic.feat.v1~x.core.global.test.v1"
            )),
            "Should contain test feature"
        );
    }
}
