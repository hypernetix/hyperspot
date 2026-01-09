use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnalyticsError {
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type AnalyticsResult<T> = Result<T, AnalyticsError>;
