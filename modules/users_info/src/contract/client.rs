use async_trait::async_trait;
use uuid::Uuid;

use crate::contract::{
    error::UsersInfoError,
    model::{NewUser, User, UserPatch},
};

/// Public API trait for the users_info module that other modules can use
#[async_trait]
pub trait UsersInfoApi: Send + Sync {
    /// Get a user by ID
    async fn get_user(&self, id: Uuid) -> Result<User, UsersInfoError>;

    /// List users with optional pagination
    async fn list_users(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<User>, UsersInfoError>;

    /// Create a new user
    async fn create_user(&self, new_user: NewUser) -> Result<User, UsersInfoError>;

    /// Update a user with partial data
    async fn update_user(&self, id: Uuid, patch: UserPatch) -> Result<User, UsersInfoError>;

    /// Delete a user by ID
    async fn delete_user(&self, id: Uuid) -> Result<(), UsersInfoError>;
}
