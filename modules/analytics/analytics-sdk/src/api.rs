use async_trait::async_trait;

#[async_trait]
pub trait AnalyticsApi: Send + Sync {
    // Business methods will be added by business features
}
