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
