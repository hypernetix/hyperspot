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

impl From<modkit_node_info::NodeInfoError> for DomainError {
    fn from(e: modkit_node_info::NodeInfoError) -> Self {
        match e {
            modkit_node_info::NodeInfoError::SysInfoCollectionFailed(msg) => {
                Self::SysInfoCollectionFailed(msg)
            }
            modkit_node_info::NodeInfoError::SysCapCollectionFailed(msg) => {
                Self::SysCapCollectionFailed(msg)
            }
            modkit_node_info::NodeInfoError::HardwareUuidFailed(msg) => {
                Self::Internal(format!("Hardware UUID failed: {msg}"))
            }
            modkit_node_info::NodeInfoError::Internal(msg) => Self::Internal(msg),
        }
    }
}
