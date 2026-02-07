use time::OffsetDateTime;
use uuid::Uuid;

/// Transport-agnostic domain event.
#[derive(Debug, Clone)]
pub enum UserDomainEvent {
    Created { id: Uuid, at: OffsetDateTime },
    Updated { id: Uuid, at: OffsetDateTime },
    Deleted { id: Uuid, at: OffsetDateTime },
}
