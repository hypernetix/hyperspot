use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::contract::model::{NewUser, User, UserPatch};

/// REST DTO for user representation with serde/utoipa
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserDto {
    pub id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// REST DTO for creating a new user
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateUserReq {
    pub email: String,
    pub display_name: String,
}

/// REST DTO for updating a user (partial)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct UpdateUserReq {
    pub email: Option<String>,
    pub display_name: Option<String>,
}

// Conversion implementations between REST DTOs and contract models
impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: Some(user.id),
            tenant_id: user.tenant_id,
            email: user.email,
            display_name: user.display_name,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

impl From<CreateUserReq> for NewUser {
    fn from(req: CreateUserReq) -> Self {
        Self {
            id: None,
            email: req.email,
            display_name: req.display_name,
        }
    }
}

impl From<UpdateUserReq> for UserPatch {
    fn from(req: UpdateUserReq) -> Self {
        Self {
            email: req.email,
            display_name: req.display_name,
        }
    }
}

/// Transport-level SSE payload.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(title = "UserEvent", description = "Server-sent user event")]
pub struct UserEvent {
    pub kind: String,
    pub id: Uuid,
    #[schema(format = "date-time")]
    pub at: DateTime<Utc>,
}

impl From<&crate::domain::events::UserDomainEvent> for UserEvent {
    fn from(e: &crate::domain::events::UserDomainEvent) -> Self {
        use crate::domain::events::UserDomainEvent::*;
        match e {
            Created { id, at } => Self {
                kind: "created".into(),
                id: *id,
                at: *at,
            },
            Updated { id, at } => Self {
                kind: "updated".into(),
                id: *id,
                at: *at,
            },
            Deleted { id, at } => Self {
                kind: "deleted".into(),
                id: *id,
                at: *at,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::events::UserDomainEvent;
    use chrono::TimeZone;

    #[test]
    fn maps_domain_event_to_transport() {
        let at = chrono::Utc
            .with_ymd_and_hms(2023, 11, 14, 12, 0, 0)
            .unwrap();
        let id = uuid::Uuid::nil();
        let de = UserDomainEvent::Created { id, at };
        let out = UserEvent::from(&de);
        assert_eq!(out.kind, "created");
        assert_eq!(out.id, id);
        assert_eq!(out.at, at);
    }

    #[test]
    fn maps_all_domain_event_variants() {
        let at = chrono::Utc
            .with_ymd_and_hms(2023, 11, 14, 12, 0, 0)
            .unwrap();
        let id = uuid::Uuid::nil();

        // Test Created event
        let created = UserDomainEvent::Created { id, at };
        let created_event = UserEvent::from(&created);
        assert_eq!(created_event.kind, "created");
        assert_eq!(created_event.id, id);
        assert_eq!(created_event.at, at);

        // Test Updated event
        let updated = UserDomainEvent::Updated { id, at };
        let updated_event = UserEvent::from(&updated);
        assert_eq!(updated_event.kind, "updated");
        assert_eq!(updated_event.id, id);
        assert_eq!(updated_event.at, at);

        // Test Deleted event
        let deleted = UserDomainEvent::Deleted { id, at };
        let deleted_event = UserEvent::from(&deleted);
        assert_eq!(deleted_event.kind, "deleted");
        assert_eq!(deleted_event.id, id);
        assert_eq!(deleted_event.at, at);
    }
}
