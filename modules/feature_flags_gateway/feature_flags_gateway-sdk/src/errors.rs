use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FeatureFlagsError {
    #[error("Invalid feature flag identifier: {value}")]
    InvalidFeatureFlagId { value: String },
}

impl FeatureFlagsError {
    #[must_use]
    pub fn invalid_feature_flag_id(value: impl Into<String>) -> Self {
        Self::InvalidFeatureFlagId {
            value: value.into(),
        }
    }
}
