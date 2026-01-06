use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    // Configuration fields will be added by business features
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {}
    }
}
