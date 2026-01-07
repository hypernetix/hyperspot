use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use user_info_sdk::{Address, City, NewAddress, NewCity, NewUser, User, UserFull, UserPatch};
use utoipa::ToSchema;
use uuid::Uuid;

/// REST DTO for user representation with serde/utoipa
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub display_name: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// REST DTO for creating a new user
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateUserReq {
    /// Optional ID for the user. If not provided, a UUID v7 will be generated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub email: String,
    pub display_name: String,
}

/// REST DTO for updating a user (partial)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct UpdateUserReq {
    pub email: Option<String>,
    pub display_name: Option<String>,
}

/// REST DTO for aggregated user response with related entities
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserFullDto {
    pub user: UserDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<AddressDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<CityDto>,
}

// Conversion implementations between REST DTOs and contract models
impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
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
            id: req.id,
            tenant_id: req.tenant_id,
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

impl From<UserFull> for UserFullDto {
    fn from(user_full: UserFull) -> Self {
        Self {
            user: UserDto::from(user_full.user),
            address: user_full.address.map(AddressDto::from),
            city: user_full.city.map(CityDto::from),
        }
    }
}

// ==================== City DTOs ====================

/// REST DTO for city representation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CityDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub country: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// REST DTO for creating a new city
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCityReq {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub name: String,
    pub country: String,
}

/// REST DTO for updating a city (partial)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct UpdateCityReq {
    pub name: Option<String>,
    pub country: Option<String>,
}

impl From<City> for CityDto {
    fn from(city: City) -> Self {
        Self {
            id: city.id,
            tenant_id: city.tenant_id,
            name: city.name,
            country: city.country,
            created_at: city.created_at,
            updated_at: city.updated_at,
        }
    }
}

impl From<CreateCityReq> for NewCity {
    fn from(req: CreateCityReq) -> Self {
        Self {
            id: req.id,
            tenant_id: req.tenant_id,
            name: req.name,
            country: req.country,
        }
    }
}

impl From<UpdateCityReq> for user_info_sdk::CityPatch {
    fn from(req: UpdateCityReq) -> Self {
        Self {
            name: req.name,
            country: req.country,
        }
    }
}

// ==================== Address DTOs ====================

/// REST DTO for address representation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AddressDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub city_id: Uuid,
    pub street: String,
    pub postal_code: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// REST DTO for creating/upserting an address
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PutAddressReq {
    pub city_id: Uuid,
    pub street: String,
    pub postal_code: String,
}

impl From<Address> for AddressDto {
    fn from(address: Address) -> Self {
        Self {
            id: address.id,
            tenant_id: address.tenant_id,
            user_id: address.user_id,
            city_id: address.city_id,
            street: address.street,
            postal_code: address.postal_code,
            created_at: address.created_at,
            updated_at: address.updated_at,
        }
    }
}

impl PutAddressReq {
    #[must_use]
    pub fn into_new_address(self, user_id: Uuid) -> NewAddress {
        NewAddress {
            id: None,
            tenant_id: Uuid::nil(),
            user_id,
            city_id: self.city_id,
            street: self.street,
            postal_code: self.postal_code,
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
    #[serde(with = "time::serde::rfc3339")]
    pub at: OffsetDateTime,
}

impl From<&crate::domain::events::UserDomainEvent> for UserEvent {
    fn from(e: &crate::domain::events::UserDomainEvent) -> Self {
        use crate::domain::events::UserDomainEvent::{Created, Deleted, Updated};
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
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{UserEvent, Uuid};
    use crate::domain::events::UserDomainEvent;
    use time::OffsetDateTime;

    #[test]
    fn maps_domain_event_to_transport() {
        let at = OffsetDateTime::from_unix_timestamp(1_699_963_200).unwrap();
        let id = Uuid::nil();
        let de = UserDomainEvent::Created { id, at };
        let out = UserEvent::from(&de);
        assert_eq!(out.kind, "created");
        assert_eq!(out.id, id);
        assert_eq!(out.at, at);
    }

    #[test]
    fn maps_all_domain_event_variants() {
        let at = OffsetDateTime::from_unix_timestamp(1_699_963_200).unwrap();
        let id = Uuid::nil();

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

    #[test]
    fn serializes_event_timestamp_with_millis() {
        let input = serde_json::json!({
            "kind": "created",
            "id": Uuid::nil(),
            "at": "2023-11-14T12:00:00.123Z"
        });

        let ev: UserEvent = serde_json::from_value(input).unwrap();
        assert_eq!(ev.at.unix_timestamp(), 1_699_963_200);
        assert_eq!(ev.at.nanosecond(), 123_000_000);
    }
}
