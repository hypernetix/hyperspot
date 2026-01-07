//! Public models for the `user_info` module.
//!
//! These are transport-agnostic data structures that define the contract
//! between the `user_info` module and its consumers.

use time::OffsetDateTime;
use uuid::Uuid;

/// A user entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

/// Data for creating a new user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewUser {
    pub id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub email: String,
    pub display_name: String,
}

/// Partial update data for a user.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserPatch {
    pub email: Option<String>,
    pub display_name: Option<String>,
}

/// Request to update a user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateUserRequest {
    pub id: Uuid,
    pub patch: UserPatch,
}

/// An address entity (1:1 with users).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub city_id: Uuid,
    pub street: String,
    pub postal_code: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

/// Data for creating a new address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewAddress {
    pub id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub city_id: Uuid,
    pub street: String,
    pub postal_code: String,
}

/// Partial update data for an address.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AddressPatch {
    pub city_id: Option<Uuid>,
    pub street: Option<String>,
    pub postal_code: Option<String>,
}

/// Request to update an address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateAddressRequest {
    pub id: Uuid,
    pub patch: AddressPatch,
}

/// A city entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct City {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub country: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

/// Data for creating a new city.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCity {
    pub id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub name: String,
    pub country: String,
}

/// Partial update data for a city.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CityPatch {
    pub name: Option<String>,
    pub country: Option<String>,
}

/// Request to update a city.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateCityRequest {
    pub id: Uuid,
    pub patch: CityPatch,
}

/// Aggregated user representation with related entities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserFull {
    pub user: User,
    pub address: Option<Address>,
    pub city: Option<City>,
}
