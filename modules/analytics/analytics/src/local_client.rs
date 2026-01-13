use analytics_sdk::AnalyticsApi;
use async_trait::async_trait;

#[derive(Clone)]
pub struct LocalAnalyticsClient {
    // Client state will be added by business features
}

impl LocalAnalyticsClient {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for LocalAnalyticsClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AnalyticsApi for LocalAnalyticsClient {
    // Method implementations will be added by business features
}
