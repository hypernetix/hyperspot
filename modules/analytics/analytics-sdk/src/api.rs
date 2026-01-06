use crate::errors::AnalyticsResult;
use async_trait::async_trait;
use modkit_security::SecurityCtx;

#[async_trait]
pub trait AnalyticsApi: Send + Sync {
    // Business methods will be added by business features
}
