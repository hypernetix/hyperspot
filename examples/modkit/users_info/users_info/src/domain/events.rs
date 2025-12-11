use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Transport-agnostic domain event.
#[derive(Debug, Clone)]
pub enum UserDomainEvent {
    Created { id: Uuid, at: DateTime<Utc> },
    Updated { id: Uuid, at: DateTime<Utc> },
    Deleted { id: Uuid, at: DateTime<Utc> },
}
