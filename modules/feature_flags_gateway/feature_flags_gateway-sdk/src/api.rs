use std::collections::HashMap;

use async_trait::async_trait;
use modkit_security::SecurityCtx;

use crate::errors::FeatureFlagsError;

#[async_trait]
pub trait FeatureFlagsApi: Send + Sync {
    async fn is_enabled(&self, sec: &SecurityCtx, flag: &str) -> Result<bool, FeatureFlagsError>;

    async fn are_enabled(
        &self,
        sec: &SecurityCtx,
        flags: &[String],
    ) -> Result<HashMap<String, bool>, FeatureFlagsError>;
}
