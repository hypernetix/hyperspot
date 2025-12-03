/// Errors for the nodes registry module
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NodesRegistryError {
    #[error("Node not found with ID: {0}")]
    NodeNotFound(uuid::Uuid),

    #[error("Failed to collect system information: {0}")]
    SysInfoCollectionFailed(String),

    #[error("Failed to collect system capabilities: {0}")]
    SysCapCollectionFailed(String),

    #[error("Invalid input: {0}")]
    Validation(String),

    #[error("An internal error occurred")]
    Internal,
}

impl From<crate::domain::error::DomainError> for NodesRegistryError {
    fn from(e: crate::domain::error::DomainError) -> Self {
        match e {
            crate::domain::error::DomainError::NodeNotFound(id) => Self::NodeNotFound(id),
            crate::domain::error::DomainError::SysInfoCollectionFailed(msg) => {
                Self::SysInfoCollectionFailed(msg)
            }
            crate::domain::error::DomainError::SysCapCollectionFailed(msg) => {
                Self::SysCapCollectionFailed(msg)
            }
            crate::domain::error::DomainError::InvalidInput(msg) => Self::Validation(msg),
            _ => Self::Internal,
        }
    }
}
