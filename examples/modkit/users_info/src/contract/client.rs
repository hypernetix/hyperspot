use async_trait::async_trait;
use uuid::Uuid;

use crate::contract::{
    error::UsersInfoError,
    model::{NewUser, User, UserPatch},
};
use modkit_odata::{ODataQuery, Page};

/// Public API trait for the users_info module that other modules can use
#[async_trait]
pub trait UsersInfoApi: Send + Sync {
    /// Get a user by ID
    async fn get_user(&self, id: Uuid) -> Result<User, UsersInfoError>;

    /// List users with cursor-based pagination
    async fn list_users(&self, query: ODataQuery) -> Result<Page<User>, UsersInfoError>;

    /// Create a new user
    async fn create_user(&self, new_user: NewUser) -> Result<User, UsersInfoError>;

    /// Update a user with partial data
    async fn update_user(&self, id: Uuid, patch: UserPatch) -> Result<User, UsersInfoError>;

    /// Delete a user by ID
    async fn delete_user(&self, id: Uuid) -> Result<(), UsersInfoError>;
}
