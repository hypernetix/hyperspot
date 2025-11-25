use async_trait::async_trait;
use modkit_security::SecurityCtx;
use uuid::Uuid;

use crate::contract::{
    error::UsersInfoError,
    model::{NewUser, UpdateUserRequest, User},
};
use modkit_odata::{ODataQuery, Page};

/// Public API trait for the users_info module that other modules can use
///
/// All methods require a SecurityCtx for proper authorization and access control.
///
/// For local (in-process) usage, see `gateways::local::UsersInfoLocalClient`.
/// For remote gRPC usage, use `#[generate_clients]` to generate a client that
/// automatically propagates SecurityCtx via gRPC metadata.
#[async_trait]
pub trait UsersInfoApi: Send + Sync {
    /// Get a user by ID
    async fn get_user(&self, ctx: &SecurityCtx, id: Uuid) -> Result<User, UsersInfoError>;

    /// List users with cursor-based pagination
    async fn list_users(
        &self,
        ctx: &SecurityCtx,
        query: ODataQuery,
    ) -> Result<Page<User>, UsersInfoError>;

    /// Create a new user
    async fn create_user(
        &self,
        ctx: &SecurityCtx,
        new_user: NewUser,
    ) -> Result<User, UsersInfoError>;

    /// Update a user with partial data
    async fn update_user(
        &self,
        ctx: &SecurityCtx,
        req: UpdateUserRequest,
    ) -> Result<User, UsersInfoError>;

    /// Delete a user by ID
    async fn delete_user(&self, ctx: &SecurityCtx, id: Uuid) -> Result<(), UsersInfoError>;
}
