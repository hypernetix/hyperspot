/// Domain-level errors for nodes registry
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Node not found: {0}")]
    NodeNotFound(uuid::Uuid),

    #[error("Failed to collect system information: {0}")]
    SysInfoCollectionFailed(String),

    #[error("Failed to collect system capabilities: {0}")]
    SysCapCollectionFailed(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<anyhow::Error> for DomainError {
    fn from(e: anyhow::Error) -> Self {
        Self::Internal(e.to_string())
    }
}
