//! Public models for the user_info module.
//!
//! These are transport-agnostic data structures that define the contract
//! between the user_info module and its consumers.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A user entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
