use thiserror::Error;

/// Errors that can occur during authentication system configuration
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("unknown plugin: {0}")]
    UnknownPlugin(String),

    #[error("invalid auth mode configuration: {0}")]
    InvalidMode(String),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),
}
