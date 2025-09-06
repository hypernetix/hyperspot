use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::contract::model::{NewUser, User, UserPatch};

/// REST DTO for user representation with serde/utoipa
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserDto {
    pub id: Uuid,
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

/// REST DTO for user list response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserListDto {
    pub users: Vec<UserDto>,
    pub total: usize,
    pub limit: u32,
    pub offset: u32,
}

/// REST DTO for query parameters
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ListUsersQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

// Conversion implementations between REST DTOs and contract models

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
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
