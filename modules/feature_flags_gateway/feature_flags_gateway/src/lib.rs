#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub use feature_flags_gateway_sdk::*;

pub mod module;
pub use module::FeatureFlagsGateway;

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::sync::Arc;

    use modkit::{client_hub::ClientHub, config::ConfigProvider, Module, ModuleCtx};
    use modkit_security::SecurityCtx;
    use tokio_util::sync::CancellationToken;
    use uuid::Uuid;

    use crate::{FeatureFlag, FeatureFlagsApi, FeatureFlagsError, FeatureFlagsGateway};

    #[derive(Default)]
    struct EmptyConfigProvider;

    impl ConfigProvider for EmptyConfigProvider {
        fn get_module_config(&self, _module_name: &str) -> Option<&serde_json::Value> {
            None
        }
    }

    macro_rules! are_enabled_test_case {
        ($name:ident, $flags:expr, $expected:expr $(,)?) => {
            #[tokio::test]
            async fn $name() {
                // Arrange
                let hub = Arc::new(ClientHub::default());
                let ctx = ModuleCtx::new(
                    "feature_flags_gateway",
                    Uuid::new_v4(),
                    Arc::new(EmptyConfigProvider),
                    Arc::clone(&hub),
                    CancellationToken::new(),
                    None,
                );
                let module = FeatureFlagsGateway;
                module.init(&ctx).await.expect("init should succeed");

                let api = ctx
                    .client_hub()
                    .get::<dyn FeatureFlagsApi>()
                    .expect("FeatureFlagsApi should be resolvable from ClientHub");
                let sec = SecurityCtx::root_ctx();

                let flags: Vec<String> = $flags;
                let expected: std::collections::HashMap<String, bool> = $expected;

                // Act
                let batch = api
                    .are_enabled(&sec, &flags)
                    .await
                    .expect("are_enabled should succeed for valid flags");

                // Assert
                assert_eq!(
                    batch, expected,
                    "Batch results must match expected per-flag evaluation"
                );
            }
        };
    }

    #[tokio::test]
    async fn init_registers_feature_flags_api_into_client_hub() {
        // Arrange
        let hub = Arc::new(ClientHub::default());
        let ctx = ModuleCtx::new(
            "feature_flags_gateway",
            Uuid::new_v4(),
            Arc::new(EmptyConfigProvider),
            Arc::clone(&hub),
            CancellationToken::new(),
            None,
        );

        let module = FeatureFlagsGateway;

        // Act
        module.init(&ctx).await.expect("init should succeed");

        // Assert
        assert!(
            ctx.client_hub().get::<dyn FeatureFlagsApi>().is_ok(),
            "FeatureFlagsApi must be resolvable from ClientHub after init"
        );
        assert_eq!(
            hub.len(),
            1,
            "FeatureFlagsGateway must register exactly one client into ClientHub"
        );
    }

    #[tokio::test]
    async fn is_enabled_returns_true_for_global_base() {
        // Arrange
        let hub = Arc::new(ClientHub::default());
        let ctx = ModuleCtx::new(
            "feature_flags_gateway",
            Uuid::new_v4(),
            Arc::new(EmptyConfigProvider),
            Arc::clone(&hub),
            CancellationToken::new(),
            None,
        );
        let module = FeatureFlagsGateway;
        module.init(&ctx).await.expect("init should succeed");

        let api = ctx
            .client_hub()
            .get::<dyn FeatureFlagsApi>()
            .expect("FeatureFlagsApi should be resolvable from ClientHub");
        let sec = SecurityCtx::root_ctx();

        // Act
        let enabled = api
            .is_enabled(&sec, FeatureFlag::GLOBAL_BASE)
            .await
            .expect("is_enabled should succeed for GlobalBase");

        // Assert
        assert!(enabled, "GlobalBase feature flag must be enabled");
    }

    #[tokio::test]
    async fn is_enabled_returns_false_for_other_valid_flags() {
        // Arrange
        let hub = Arc::new(ClientHub::default());
        let ctx = ModuleCtx::new(
            "feature_flags_gateway",
            Uuid::new_v4(),
            Arc::new(EmptyConfigProvider),
            Arc::clone(&hub),
            CancellationToken::new(),
            None,
        );
        let module = FeatureFlagsGateway;
        module.init(&ctx).await.expect("init should succeed");

        let api = ctx
            .client_hub()
            .get::<dyn FeatureFlagsApi>()
            .expect("FeatureFlagsApi should be resolvable from ClientHub");
        let sec = SecurityCtx::root_ctx();

        let other_flag = "gts.x.core.ff.flag.v1~acme.some.flag.v1";

        // Act
        let enabled = api
            .is_enabled(&sec, other_flag)
            .await
            .expect("is_enabled should succeed for a valid non-GlobalBase flag");

        // Assert
        assert!(
            !enabled,
            "Non-GlobalBase flags must be disabled in the stub evaluator"
        );
    }

    #[tokio::test]
    async fn is_enabled_returns_error_for_empty_flag_id() {
        // Arrange
        let hub = Arc::new(ClientHub::default());
        let ctx = ModuleCtx::new(
            "feature_flags_gateway",
            Uuid::new_v4(),
            Arc::new(EmptyConfigProvider),
            Arc::clone(&hub),
            CancellationToken::new(),
            None,
        );
        let module = FeatureFlagsGateway;
        module.init(&ctx).await.expect("init should succeed");

        let api = ctx
            .client_hub()
            .get::<dyn FeatureFlagsApi>()
            .expect("FeatureFlagsApi should be resolvable from ClientHub");
        let sec = SecurityCtx::root_ctx();

        // Act
        let err = api
            .is_enabled(&sec, "")
            .await
            .expect_err("is_enabled must reject empty feature flag identifiers");

        // Assert
        assert_eq!(
            err,
            FeatureFlagsError::InvalidFeatureFlagId {
                value: String::default(),
            },
            "is_enabled must return InvalidFeatureFlagId for empty identifiers"
        );
    }

    are_enabled_test_case!(
        are_enabled_returns_expected_values_for_known_flags,
        vec![
            FeatureFlag::GLOBAL_BASE.to_owned(),
            "gts.x.core.ff.flag.v1~acme.some.flag.v1".to_owned(),
        ],
        std::collections::HashMap::from([
            (FeatureFlag::GLOBAL_BASE.to_owned(), true),
            ("gts.x.core.ff.flag.v1~acme.some.flag.v1".to_owned(), false),
        ]),
    );

    are_enabled_test_case!(
        are_enabled_with_empty_input_returns_empty_map,
        vec![],
        std::collections::HashMap::new(),
    );

    are_enabled_test_case!(
        are_enabled_with_duplicate_flags_deduplicates_in_output_map,
        vec![
            FeatureFlag::GLOBAL_BASE.to_owned(),
            FeatureFlag::GLOBAL_BASE.to_owned(),
        ],
        std::collections::HashMap::from([(FeatureFlag::GLOBAL_BASE.to_owned(), true)]),
    );

    #[tokio::test]
    async fn are_enabled_returns_error_for_whitespace_only_flag_id() {
        // Arrange
        let hub = Arc::new(ClientHub::default());
        let ctx = ModuleCtx::new(
            "feature_flags_gateway",
            Uuid::new_v4(),
            Arc::new(EmptyConfigProvider),
            Arc::clone(&hub),
            CancellationToken::new(),
            None,
        );
        let module = FeatureFlagsGateway;
        module.init(&ctx).await.expect("init should succeed");

        let api = ctx
            .client_hub()
            .get::<dyn FeatureFlagsApi>()
            .expect("FeatureFlagsApi should be resolvable from ClientHub");
        let sec = SecurityCtx::root_ctx();

        let flags = vec![" ".to_owned()];

        // Act
        let err = api
            .are_enabled(&sec, &flags)
            .await
            .expect_err("are_enabled must reject whitespace-only feature flag identifiers");

        // Assert
        assert_eq!(
            err,
            FeatureFlagsError::InvalidFeatureFlagId {
                value: " ".to_owned(),
            },
            "are_enabled must return InvalidFeatureFlagId for whitespace-only identifiers"
        );
    }
}
