//! `UsersInfoApi` trait definition.
//!
//! This trait defines the public API for the `user_info` module.
//! All methods require a `SecurityCtx` for authorization and access control.

use async_trait::async_trait;
use modkit_odata::{ODataQuery, Page};
use modkit_security::SecurityCtx;
use uuid::Uuid;

use crate::errors::UsersInfoError;
use crate::models::{NewUser, UpdateUserRequest, User};

/// Public API trait for the `user_info` module.
///
/// This trait can be consumed by other modules via `ClientHub`:
/// ```ignore
/// let client = hub.get::<dyn UsersInfoApi>()?;
/// let user = client.get_user(&ctx, user_id).await?;
/// ```
///
/// All methods require a `SecurityCtx` for proper authorization and access control.
#[async_trait]
pub trait UsersInfoApi: Send + Sync {
    /// Get a user by ID.
    async fn get_user(&self, ctx: &SecurityCtx, id: Uuid) -> Result<User, UsersInfoError>;

    /// List users with cursor-based pagination.
    async fn list_users(
        &self,
        ctx: &SecurityCtx,
        query: ODataQuery,
    ) -> Result<Page<User>, UsersInfoError>;

    /// Create a new user.
    async fn create_user(
        &self,
        ctx: &SecurityCtx,
        new_user: NewUser,
    ) -> Result<User, UsersInfoError>;

    /// Update a user with partial data.
    async fn update_user(
        &self,
        ctx: &SecurityCtx,
        req: UpdateUserRequest,
    ) -> Result<User, UsersInfoError>;

    /// Delete a user by ID.
    async fn delete_user(&self, ctx: &SecurityCtx, id: Uuid) -> Result<(), UsersInfoError>;
}
