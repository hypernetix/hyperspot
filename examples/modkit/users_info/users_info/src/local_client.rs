//! Local implementation of `UsersInfoApi`.
//!
//! This client is used for inter-module communication within the same process.
//! It delegates to the domain service and converts errors to SDK error types.

use async_trait::async_trait;
use modkit_odata::{ODataQuery, Page};
use modkit_security::SecurityContext;
use std::sync::Arc;
use uuid::Uuid;

use user_info_sdk::{NewUser, UpdateUserRequest, User, UsersInfoApi, UsersInfoError};

use crate::domain::service::Service;

/// Local implementation of the `UsersInfoApi` trait that delegates to the domain service.
///
/// This client is used for inter-module communication within the same process.
/// It accepts a `SecurityCtx` from the caller and forwards it directly to the domain service,
/// ensuring proper authorization and access control throughout the call chain.
pub struct UsersInfoLocalClient {
    service: Arc<Service>,
}

impl UsersInfoLocalClient {
    /// Create a new local client wrapping the domain service.
    #[must_use]
    pub fn new(service: Arc<Service>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl UsersInfoApi for UsersInfoLocalClient {
    async fn get_user(&self, ctx: &SecurityContext, id: Uuid) -> Result<User, UsersInfoError> {
        self.service.get_user(ctx, id).await.map_err(Into::into)
    }

    async fn list_users(
        &self,
        ctx: &SecurityContext,
        query: ODataQuery,
    ) -> Result<Page<User>, UsersInfoError> {
        self.service
            .list_users_page(ctx, &query)
            .await
            .map_err(|e| {
                // OData errors at this layer are unexpected (query construction errors)
                // Log and convert to internal error
                tracing::error!(error = ?e, "Unexpected OData error in gateway");
                UsersInfoError::internal()
            })
    }

    async fn create_user(
        &self,
        ctx: &SecurityContext,
        new_user: NewUser,
    ) -> Result<User, UsersInfoError> {
        self.service
            .create_user(ctx, new_user)
            .await
            .map_err(Into::into)
    }

    async fn update_user(
        &self,
        ctx: &SecurityContext,
        req: UpdateUserRequest,
    ) -> Result<User, UsersInfoError> {
        self.service
            .update_user(ctx, req.id, req.patch)
            .await
            .map_err(Into::into)
    }

    async fn delete_user(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), UsersInfoError> {
        self.service.delete_user(ctx, id).await.map_err(Into::into)
    }
}
