use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::error::DomainError;

/// Transport-agnostic audit port that encapsulates both external effects:
/// 1) user-access check (GET)
/// 2) user-created notification (POST)
#[async_trait]
pub trait AuditPort: Send + Sync {
    async fn get_user_access(&self, id: Uuid) -> Result<(), DomainError>;
    async fn notify_user_created(&self) -> Result<(), DomainError>;
}
