//! Service implementation for static licenses plugin.
//!
//! This is a configuration-driven stub that returns configured features for bootstrap/testing.

use gts::GtsID;
use license_enforcer_sdk::{
    EnabledGlobalFeatures, LicenseEnforcerError, global_features, models::LicenseFeatureId,
};
use modkit_security::SecurityContext;

/// Static licenses service.
///
/// Provides configured license data. Returns the base feature plus
/// any additional features specified in configuration.
pub struct Service {
    /// Configured enabled global features (in addition to base feature).
    configured_features: Vec<GtsID>,
}

impl Service {
    /// Create a new service with configured features.
    ///
    /// The service will return the base feature plus any features
    /// provided in the configuration.
    #[must_use]
    pub fn new(configured_features: Vec<GtsID>) -> Self {
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
        let base_feature = &global_features::BaseFeature;
        let mut features = EnabledGlobalFeatures::new();
        features.insert(base_feature.to_gts());

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
    use license_enforcer_sdk::models::LicenseFeatureId;
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
        let base_feature = &global_features::BaseFeature;
        assert_eq!(features.len(), 1, "Should return only base feature");
        assert!(
            features.contains(&base_feature.to_gts()),
            "Should contain base feature"
        );
    }

    #[tokio::test]
    async fn test_service_returns_base_plus_configured_features() {
        let configured = vec![
            global_features::CyberChatFeature.to_gts(),
            global_features::CyberEmployeeAgentsFeature.to_gts(),
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
            features.contains(&global_features::BaseFeature.to_gts()),
            "Should contain base feature"
        );
        assert!(
            features.contains(&global_features::CyberChatFeature.to_gts()),
            "Should contain cyber_chat feature"
        );
        assert!(
            features.contains(&global_features::CyberEmployeeAgentsFeature.to_gts()),
            "Should contain cyber_employee_agents feature"
        );
    }

    #[tokio::test]
    async fn test_service_handles_duplicate_base_feature() {
        // If base feature is included in config, should not duplicate
        let configured = vec![
            global_features::BaseFeature.to_gts(),
            global_features::CyberEmployeeUnitsFeature.to_gts(),
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
            features.contains(&global_features::BaseFeature.to_gts()),
            "Should contain base feature"
        );
        assert!(
            features.contains(&global_features::CyberEmployeeUnitsFeature.to_gts()),
            "Should contain cyber_employee_units feature"
        );
    }
}
