use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::contract::{
    client::UsersInfoApi,
    error::UsersInfoError,
    model::{NewUser, UpdateUserRequest, User},
};
use crate::domain::service::Service;
use modkit_odata::{ODataQuery, Page};
use modkit_security::SecurityCtx;

/// Local implementation of the UsersInfoApi trait that delegates to the domain service.
///
/// This client is used for inter-module communication within the same process.
/// It accepts a SecurityCtx from the caller and forwards it directly to the domain service,
/// ensuring proper authorization and access control throughout the call chain.
pub struct UsersInfoLocalClient {
    service: Arc<Service>,
}

impl UsersInfoLocalClient {
    pub fn new(service: Arc<Service>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl UsersInfoApi for UsersInfoLocalClient {
    async fn get_user(&self, ctx: &SecurityCtx, id: Uuid) -> Result<User, UsersInfoError> {
        self.service.get_user(ctx, id).await.map_err(Into::into)
    }

    async fn list_users(
        &self,
        ctx: &SecurityCtx,
        query: ODataQuery,
    ) -> Result<Page<User>, UsersInfoError> {
        self.service.list_users_page(ctx, query).await.map_err(|e| {
            // OData errors at this layer are unexpected (query construction errors)
            // Log and convert to internal error
            tracing::error!(error = ?e, "Unexpected OData error in gateway");
            UsersInfoError::internal()
        })
    }

    async fn create_user(
        &self,
        ctx: &SecurityCtx,
        new_user: NewUser,
    ) -> Result<User, UsersInfoError> {
        self.service
            .create_user(ctx, new_user)
            .await
            .map_err(Into::into)
    }

    async fn update_user(
        &self,
        ctx: &SecurityCtx,
        req: UpdateUserRequest,
    ) -> Result<User, UsersInfoError> {
        self.service
            .update_user(ctx, req.id, req.patch)
            .await
            .map_err(Into::into)
    }

    async fn delete_user(&self, ctx: &SecurityCtx, id: Uuid) -> Result<(), UsersInfoError> {
        self.service.delete_user(ctx, id).await.map_err(Into::into)
    }
}
