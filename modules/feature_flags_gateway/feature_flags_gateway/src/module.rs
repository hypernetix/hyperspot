use async_trait::async_trait;
use modkit::{Module, ModuleCtx};
use std::collections::HashMap;
use std::sync::Arc;

use feature_flags_gateway_sdk::FeatureFlag;

#[modkit::module(name = "feature_flags_gateway", capabilities = [])]
pub struct FeatureFlagsGateway;

impl Default for FeatureFlagsGateway {
    fn default() -> Self {
        Self
    }
}

struct StubFeatureFlags;

#[async_trait]
impl feature_flags_gateway_sdk::FeatureFlagsApi for StubFeatureFlags {
    async fn is_enabled(
        &self,
        _sec: &modkit_security::SecurityCtx,
        flag: &str,
    ) -> Result<bool, feature_flags_gateway_sdk::FeatureFlagsError> {
        if flag.trim().is_empty() {
            return Err(
                feature_flags_gateway_sdk::FeatureFlagsError::invalid_feature_flag_id(
                    flag.to_owned(),
                ),
            );
        }
        Ok(flag == FeatureFlag::GLOBAL_BASE)
    }

    async fn are_enabled(
        &self,
        sec: &modkit_security::SecurityCtx,
        flags: &[String],
    ) -> Result<HashMap<String, bool>, feature_flags_gateway_sdk::FeatureFlagsError> {
        let mut result = HashMap::with_capacity(flags.len());
        for flag in flags {
            if flag.trim().is_empty() {
                return Err(
                    feature_flags_gateway_sdk::FeatureFlagsError::invalid_feature_flag_id(
                        flag.clone(),
                    ),
                );
            }
            let enabled = self.is_enabled(sec, flag).await?;
            result.insert(flag.clone(), enabled);
        }

        Ok(result)
    }
}

#[async_trait]
impl Module for FeatureFlagsGateway {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        let api: Arc<dyn feature_flags_gateway_sdk::FeatureFlagsApi> = Arc::new(StubFeatureFlags);
        ctx.client_hub()
            .register::<dyn feature_flags_gateway_sdk::FeatureFlagsApi>(api);
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
