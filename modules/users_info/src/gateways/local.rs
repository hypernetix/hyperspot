use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::contract::{
    client::UsersInfoApi,
    error::UsersInfoError,
    model::{NewUser, User, UserPatch},
};
use crate::domain::service::Service;

/// Local implementation of the UsersInfoApi trait that delegates to the domain service
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
    async fn get_user(&self, id: Uuid) -> Result<User, UsersInfoError> {
        self.service.get_user(id).await.map_err(Into::into)
    }

    async fn list_users(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<User>, UsersInfoError> {
        self.service
            .list_users(limit, offset)
            .await
            .map_err(Into::into)
    }

    async fn create_user(&self, new_user: NewUser) -> Result<User, UsersInfoError> {
        self.service.create_user(new_user).await.map_err(Into::into)
    }

    async fn update_user(&self, id: Uuid, patch: UserPatch) -> Result<User, UsersInfoError> {
        self.service
            .update_user(id, patch)
            .await
            .map_err(Into::into)
    }

    async fn delete_user(&self, id: Uuid) -> Result<(), UsersInfoError> {
        self.service.delete_user(id).await.map_err(Into::into)
    }
}
